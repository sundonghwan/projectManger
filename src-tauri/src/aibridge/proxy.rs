//! 로컬 loopback 프록시. 원격 CLI 가 SSH 리버스 터널을 통해 이 포트로 요청을 보내면,
//! 더미 인증 헤더를 실제 OAuth 토큰으로 교체한 뒤 실제 업스트림 API 로 스트리밍 중계한다.
//! 요청/응답 body 는 버퍼링 없이 스트림으로 흘려보내 SSE 가 그대로 통과한다.
use crate::aibridge::{headers, token::ClaudeTokenSource};
use crate::error::{AppError, Result};
use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    routing::any,
    Router,
};
use tokio::net::TcpListener;

const ANTHROPIC_UPSTREAM: &str = "https://api.anthropic.com";
const OPENAI_UPSTREAM: &str = "https://chatgpt.com/backend-api/codex";

#[derive(Clone, Copy)]
pub enum Provider {
    Anthropic,
    OpenAi,
}

#[derive(Clone, Copy)]
pub struct Ctx {
    provider: Provider,
    upstream: &'static str,
}

pub fn anthropic_ctx() -> Ctx {
    Ctx {
        provider: Provider::Anthropic,
        upstream: ANTHROPIC_UPSTREAM,
    }
}

pub fn openai_ctx() -> Ctx {
    Ctx {
        provider: Provider::OpenAi,
        upstream: OPENAI_UPSTREAM,
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
            headers::apply_anthropic_auth(&mut headers, &tok);
        }
        Provider::OpenAi => {
            // Task 8: OpenAI(codex) 토큰 소스 + headers::apply_openai_auth 주입 예정.
            // 지금은 no-op — 더미 인증이 그대로 통과하므로 실제 호출은 실패한다(의도된 스텁).
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
