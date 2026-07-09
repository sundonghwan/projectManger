//! 로컬 loopback 프록시. 원격 CLI 가 SSH 리버스 터널을 통해 이 포트로 요청을 보내면,
//! 더미 인증 헤더를 실제 OAuth 토큰으로 교체한 뒤 실제 업스트림 API 로 스트리밍 중계한다.
//! 요청/응답 body 는 버퍼링 없이 스트림으로 흘려보내 SSE 가 그대로 통과한다.
use crate::aibridge::{
    headers,
    token::{ClaudeTokenSource, CodexTokenSource},
};
use crate::error::{AppError, Result};
use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    routing::any,
    Router,
};
use tauri::{AppHandle, Emitter};
use tokio::net::TcpListener;

const ANTHROPIC_UPSTREAM: &str = "https://api.anthropic.com";
const OPENAI_UPSTREAM: &str = "https://chatgpt.com/backend-api/codex";

#[derive(Clone, Copy)]
pub enum Provider {
    Anthropic,
    OpenAi,
}

impl Provider {
    fn as_str(self) -> &'static str {
        match self {
            Provider::Anthropic => "anthropic",
            Provider::OpenAi => "openai",
        }
    }
}

/// axum `State` 로 공유되는 프록시 컨텍스트. `AppHandle` 은 `Copy` 가 아니므로 `Ctx` 도
/// `Clone` 만 구현한다(핸들러마다 저비용 클론 — 내부적으로 Arc 기반).
#[derive(Clone)]
pub struct Ctx {
    provider: Provider,
    upstream: &'static str,
    app: AppHandle,
}

pub fn anthropic_ctx(app: AppHandle) -> Ctx {
    Ctx {
        provider: Provider::Anthropic,
        upstream: ANTHROPIC_UPSTREAM,
        app,
    }
}

pub fn openai_ctx(app: AppHandle) -> Ctx {
    Ctx {
        provider: Provider::OpenAi,
        upstream: OPENAI_UPSTREAM,
        app,
    }
}

/// loopback 리스너에서 axum 서버를 구동한다. catch-all 라우트로 모든 method/path 를 받는다.
pub async fn serve(listener: TcpListener, ctx: Ctx) -> Result<()> {
    let app = Router::new()
        .route("/", any(handle))
        .fallback(any(handle))
        .with_state(ctx);
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Invalid(format!("proxy serve: {e}")))
}

async fn handle(State(ctx): State<Ctx>, req: Request<Body>) -> Response<Body> {
    match forward(ctx, req).await {
        Ok(r) => r,
        Err(e) => Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .body(Body::from(format!("bridge error: {e}")))
            .expect("static error response"),
    }
}

async fn forward(ctx: Ctx, req: Request<Body>) -> Result<Response<Body>> {
    let (parts, body) = req.into_parts();
    let path_q = parts
        .uri
        .path_and_query()
        .map(|p| p.as_str())
        .unwrap_or("/");
    let url = format!("{}{}", ctx.upstream, path_q);

    let mut headers = parts.headers.clone();
    headers.remove(http::header::HOST);
    match ctx.provider {
        Provider::Anthropic => {
            let tok = ClaudeTokenSource.access_token().await?;
            headers::apply_anthropic_auth(&mut headers, &tok)?;
        }
        Provider::OpenAi => {
            let (tok, acct) = CodexTokenSource.access_and_account().await?;
            headers::apply_openai_auth(&mut headers, &tok, acct.as_deref())?;
        }
    }

    // 요청 body 를 스트림 그대로 업스트림으로 전달 (버퍼링 금지).
    let req_stream = body.into_data_stream();
    let client = reqwest::Client::new();
    let up = client
        .request(parts.method, &url)
        .headers(headers)
        .body(reqwest::Body::wrap_stream(req_stream))
        .send()
        .await
        .map_err(|e| AppError::Invalid(format!("upstream: {e}")))?;

    // 인증/쿼터 실패를 감지해 프론트에 알린다. 응답은 그대로 통과시키므로(아래) emit 실패는
    // 무시한다 — 토큰/헤더 값은 로깅하지 않는다.
    let kind = match up.status().as_u16() {
        401 | 403 => Some("auth"),
        429 => Some("quota"),
        _ => None,
    };
    if let Some(kind) = kind {
        let _ = ctx.app.emit(
            "aibridge://alert",
            serde_json::json!({ "provider": ctx.provider.as_str(), "kind": kind }),
        );
    }

    // 응답 status/headers 복사 후 body 는 스트림으로 흘려보낸다 (SSE 통과).
    let mut builder = Response::builder().status(up.status());
    for (k, v) in up.headers() {
        builder = builder.header(k, v);
    }
    let out = Body::from_stream(up.bytes_stream());
    builder
        .body(out)
        .map_err(|e| AppError::Invalid(format!("resp build: {e}")))
}
