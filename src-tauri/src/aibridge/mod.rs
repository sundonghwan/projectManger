pub mod headers;
pub mod proxy;
pub mod token;

use crate::error::{AppError, Result};
use crate::terminal::BridgePorts;
use std::sync::Mutex;
use tauri::AppHandle;
use tokio::net::TcpListener;

/// 원격 AI 자격증명 브리지의 실행 상태. 프록시 서버 기동 여부와 확정된 포트 쌍을 보관.
#[derive(Default)]
pub struct AiBridge {
    ports: Mutex<Option<BridgePorts>>,
}

impl AiBridge {
    /// 미기동이면 두 loopback 프록시(anthropic/openai)를 기동하고 포트를 확정해 반환한다.
    /// 이미 기동돼 있으면 저장된 포트를 그대로 돌려준다. `app` 은 프록시가 인증/쿼터 실패
    /// 감지 시 `aibridge://alert` 이벤트를 emit 하는 데 쓰인다.
    pub fn ensure_started(&self, app: &AppHandle) -> Result<BridgePorts> {
        if let Some(p) = *self.ports.lock().unwrap() {
            return Ok(p);
        }
        let (a_listener, a_local) = bind_loopback()?;
        let (o_listener, o_local) = bind_loopback()?;
        let ports = BridgePorts {
            anthropic_remote: 8971,
            anthropic_local: a_local,
            openai_remote: 8972,
            openai_local: o_local,
        };
        tauri::async_runtime::spawn(proxy::serve(a_listener, proxy::anthropic_ctx(app.clone())));
        tauri::async_runtime::spawn(proxy::serve(o_listener, proxy::openai_ctx(app.clone())));
        *self.ports.lock().unwrap() = Some(ports);
        Ok(ports)
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
