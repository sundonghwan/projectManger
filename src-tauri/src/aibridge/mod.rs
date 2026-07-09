pub mod cswap;
pub mod headers;
pub mod proxy;
pub mod token;

use crate::error::{AppError, Result};
use crate::terminal::BridgePorts;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::LazyLock;
use std::sync::Mutex;
use tauri::AppHandle;
use tokio::net::TcpListener;

/// 브리지 전역에서 공유하는 HTTP 클라이언트(커넥션 풀/TLS 세션 재사용).
pub static HTTP: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

/// 원격 AI 자격증명 브리지의 실행 상태. 로컬 프록시 리슨 포트는 프로세스당 하나만 기동해
/// 공유하고, 원격(리버스 포워드) 포트는 세션(`ensure_started` 호출)마다 새로 발급한다.
#[derive(Default)]
pub struct AiBridge {
    /// (anthropic_local, openai_local) — 프록시 리슨 포트(공유, 최초 1회 기동).
    local: Mutex<Option<(u16, u16)>>,
    next_remote: AtomicU16,
}

impl AiBridge {
    /// 로컬 프록시를 (필요 시) 기동하고, 이번 세션 전용 원격 포트 쌍을 발급해 반환한다.
    /// `app` 은 프록시가 인증/쿼터 실패 감지 시 `aibridge://alert` 이벤트를 emit 하는 데 쓰인다.
    pub fn ensure_started(&self, app: &AppHandle) -> Result<BridgePorts> {
        let (a_local, o_local) = self.ensure_proxies(app)?;
        // 세션마다 고유 원격 포트 → 같은 호스트로 동시에 여러 브리지 세션을 열어도
        // `-R` 리버스 포워드 포트가 충돌하지 않는다.
        // %1000 으로 8971..=9970 범위에 묶는다 — 500개 초과 동시 세션이면 값이 겹칠 수 있으나
        // 개인 사용 규모에서는 무해하다.
        let step = self.next_remote.fetch_add(2, Ordering::Relaxed);
        let anthropic_remote = 8971 + (step % 1000);
        let openai_remote = anthropic_remote + 1;
        Ok(BridgePorts {
            anthropic_remote,
            anthropic_local: a_local,
            openai_remote,
            openai_local: o_local,
        })
    }

    /// 미기동이면 두 loopback 프록시(anthropic/openai)를 기동하고 로컬 포트를 확정해 반환한다.
    /// 이미 기동돼 있으면 저장된 로컬 포트를 그대로 돌려준다(프록시는 프로세스당 1개만 필요).
    fn ensure_proxies(&self, app: &AppHandle) -> Result<(u16, u16)> {
        let mut guard = self.local.lock().unwrap();
        if let Some(p) = *guard {
            return Ok(p);
        }
        let (a_listener, a_local) = bind_loopback()?;
        let (o_listener, o_local) = bind_loopback()?;
        tauri::async_runtime::spawn(proxy::serve(a_listener, proxy::anthropic_ctx(app.clone())));
        tauri::async_runtime::spawn(proxy::serve(o_listener, proxy::openai_ctx(app.clone())));
        *guard = Some((a_local, o_local));
        Ok((a_local, o_local))
    }
}

/// `127.0.0.1:0` 으로 바인딩해 OS 가 배정한 로컬 포트를 확정한다 (loopback 전용).
fn bind_loopback() -> Result<(TcpListener, u16)> {
    tauri::async_runtime::block_on(async {
        let l = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| AppError::Invalid(format!("bind: {e}")))?;
        let port = l
            .local_addr()
            .map_err(|e| AppError::Invalid(format!("local_addr: {e}")))?
            .port();
        Ok((l, port))
    })
}
