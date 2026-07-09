//! 업스트림으로 나가기 직전, 원격이 보낸 더미 인증을 실제 OAuth 토큰으로 교체한다.
use crate::error::{AppError, Result};
use http::{HeaderMap, HeaderValue};

const ANTHROPIC_OAUTH_BETA: &str = "oauth-2025-04-20";

/// 토큰/계정에서 파생된 값을 HTTP 헤더 값으로 변환한다. 손상된 refresh/키체인에서 온
/// 헤더 부적합 바이트가 프록시 핸들러를 패닉시키지 않도록 실패를 Err 로 흘린다.
fn header_value(v: &str) -> Result<HeaderValue> {
    HeaderValue::from_str(v).map_err(|e| AppError::Invalid(format!("헤더 값 부적합: {e}")))
}

pub fn apply_anthropic_auth(headers: &mut HeaderMap, access_token: &str) -> Result<()> {
    headers.remove("authorization");
    headers.remove("x-api-key");
    headers.insert("authorization", header_value(&format!("Bearer {access_token}"))?);
    let merged = match headers.get("anthropic-beta").and_then(|v| v.to_str().ok()) {
        Some(existing) if existing.split(',').any(|p| p.trim() == ANTHROPIC_OAUTH_BETA) => {
            existing.to_string()
        }
        Some(existing) if !existing.is_empty() => format!("{existing},{ANTHROPIC_OAUTH_BETA}"),
        _ => ANTHROPIC_OAUTH_BETA.to_string(),
    };
    headers.insert("anthropic-beta", header_value(&merged)?);
    Ok(())
}

pub fn apply_openai_auth(headers: &mut HeaderMap, access_token: &str, account_id: Option<&str>) -> Result<()> {
    headers.remove("authorization");
    headers.remove("x-api-key");
    headers.insert("authorization", header_value(&format!("Bearer {access_token}"))?);
    if let Some(acct) = account_id {
        headers.insert("chatgpt-account-id", header_value(acct)?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderMap;

    #[test]
    fn anthropic_overwrites_auth_and_sets_beta() {
        let mut h = HeaderMap::new();
        h.insert("authorization", "Bearer bridge".parse().unwrap());
        h.insert("x-api-key", "leak".parse().unwrap());
        apply_anthropic_auth(&mut h, "real-token").unwrap();
        assert_eq!(h.get("authorization").unwrap(), "Bearer real-token");
        assert!(h.get("x-api-key").is_none());
        assert!(h.get("anthropic-beta").unwrap().to_str().unwrap().contains("oauth-2025-04-20"));
    }

    #[test]
    fn anthropic_merges_existing_beta_without_dup() {
        let mut h = HeaderMap::new();
        h.insert("anthropic-beta", "messages-2023-12-15".parse().unwrap());
        apply_anthropic_auth(&mut h, "t").unwrap();
        let beta = h.get("anthropic-beta").unwrap().to_str().unwrap().to_string();
        assert!(beta.contains("messages-2023-12-15"));
        assert!(beta.contains("oauth-2025-04-20"));
        apply_anthropic_auth(&mut h, "t").unwrap(); // 두 번 적용해도 중복 없음
        assert_eq!(h.get("anthropic-beta").unwrap().to_str().unwrap().matches("oauth-2025-04-20").count(), 1);
    }

    #[test]
    fn openai_sets_bearer_and_account() {
        let mut h = HeaderMap::new();
        h.insert("authorization", "Bearer bridge".parse().unwrap());
        apply_openai_auth(&mut h, "tok", Some("acct_123")).unwrap();
        assert_eq!(h.get("authorization").unwrap(), "Bearer tok");
        assert_eq!(h.get("chatgpt-account-id").unwrap(), "acct_123");
    }

    #[test]
    fn anthropic_malformed_token_returns_err_not_panic() {
        // 손상된 refresh/키체인에서 온 헤더 부적합 바이트(개행/제어문자)는
        // 패닉이 아니라 Err 로 전파되어야 한다.
        let mut h = HeaderMap::new();
        assert!(apply_anthropic_auth(&mut h, "tok\nen").is_err());
        assert!(apply_anthropic_auth(&mut h, "tok\u{7f}").is_err());
    }

    #[test]
    fn openai_malformed_token_or_account_returns_err_not_panic() {
        let mut h = HeaderMap::new();
        assert!(apply_openai_auth(&mut h, "tok\nen", None).is_err());
        assert!(apply_openai_auth(&mut h, "tok", Some("acct\u{7f}")).is_err());
    }
}
