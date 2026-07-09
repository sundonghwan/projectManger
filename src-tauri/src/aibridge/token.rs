//! OAuth 토큰 모델과 refresh 판단. 저장소 I/O 는 어댑터(별도 태스크)에서.
use crate::error::{AppError, Result};
use keyring::Entry;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct OauthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at_ms: i64,
    pub account_id: Option<String>,
}

pub fn needs_refresh(expires_at_ms: i64, now_ms: i64, skew_ms: i64) -> bool {
    now_ms + skew_ms >= expires_at_ms
}

fn s(v: &Value, k: &str) -> Result<String> {
    v.get(k).and_then(|x| x.as_str()).map(str::to_string)
        .ok_or_else(|| AppError::Invalid(format!("토큰 필드 누락: {k}")))
}

pub fn parse_claude_keychain(json: &str) -> Result<OauthToken> {
    let v: Value = serde_json::from_str(json).map_err(|e| AppError::Invalid(format!("claude 토큰 파싱: {e}")))?;
    let o = v.get("claudeAiOauth").ok_or_else(|| AppError::Invalid("claudeAiOauth 없음".into()))?;
    Ok(OauthToken {
        access_token: s(o, "accessToken")?,
        refresh_token: s(o, "refreshToken")?,
        expires_at_ms: o.get("expiresAt").and_then(|x| x.as_i64()).unwrap_or(0),
        account_id: None,
    })
}

pub fn parse_codex_authfile(json: &str) -> Result<OauthToken> {
    let v: Value = serde_json::from_str(json).map_err(|e| AppError::Invalid(format!("codex 토큰 파싱: {e}")))?;
    let t = v.get("tokens").ok_or_else(|| AppError::Invalid("tokens 없음".into()))?;
    Ok(OauthToken {
        access_token: s(t, "access_token")?,
        refresh_token: s(t, "refresh_token")?,
        expires_at_ms: t.get("expiresAt").and_then(|x| x.as_i64()).unwrap_or(0),
        account_id: t.get("account_id").and_then(|x| x.as_str()).map(str::to_string),
    })
}

const CLAUDE_KEYCHAIN_SERVICE: &str = "Claude Code-credentials";
const REFRESH_SKEW_MS: i64 = 60_000;

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub struct ClaudeTokenSource;

impl ClaudeTokenSource {
    fn entry() -> Result<Entry> {
        // 키체인 account 이름은 실제 저장 시 사용된 값으로 확인 필요(대개 사용자명/고정값).
        Entry::new(CLAUDE_KEYCHAIN_SERVICE, "default")
            .map_err(|e| AppError::Invalid(format!("키체인 오류: {e}")))
    }

    pub async fn access_token(&self) -> Result<String> {
        let raw = Self::entry()?
            .get_password()
            .map_err(|e| AppError::Invalid(format!("claude 자격증명 로드 실패: {e}")))?;
        let tok = parse_claude_keychain(&raw)?;
        if !needs_refresh(tok.expires_at_ms, now_ms(), REFRESH_SKEW_MS) {
            return Ok(tok.access_token);
        }
        let refreshed = refresh_claude(&tok.refresh_token).await?;
        // 키체인 되쓰기 (로컬 세션 동기 유지)
        let write = serde_json::json!({"claudeAiOauth": {
            "accessToken": refreshed.access_token,
            "refreshToken": refreshed.refresh_token,
            "expiresAt": refreshed.expires_at_ms,
        }});
        Self::entry()?.set_password(&write.to_string())
            .map_err(|e| AppError::Invalid(format!("claude 자격증명 저장 실패: {e}")))?;
        Ok(refreshed.access_token)
    }
}

async fn refresh_claude(refresh_token: &str) -> Result<OauthToken> {
    let body = serde_json::json!({
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
        "client_id": "9d1c250a-e61b-44d9-88ed-5944d1962f5e"
    });
    let resp = reqwest::Client::new()
        .post("https://console.anthropic.com/v1/oauth/token")
        .json(&body).send().await
        .map_err(|e| AppError::Invalid(format!("claude refresh 요청 실패: {e}")))?;
    let v: Value = resp.json().await
        .map_err(|e| AppError::Invalid(format!("claude refresh 응답 파싱: {e}")))?;
    let expires_in = v.get("expires_in").and_then(|x| x.as_i64()).unwrap_or(3600);
    Ok(OauthToken {
        access_token: s(&v, "access_token")?,
        refresh_token: v.get("refresh_token").and_then(|x| x.as_str())
            .map(str::to_string).unwrap_or_else(|| refresh_token.to_string()),
        expires_at_ms: now_ms() + expires_in * 1000,
        account_id: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_when_within_skew() {
        assert!(needs_refresh(1_000, 900, 200));   // 900+200 >= 1000
        assert!(!needs_refresh(1_000, 700, 200));  // 700+200 < 1000
    }

    #[test]
    fn parses_claude_keychain_json() {
        let j = r#"{"claudeAiOauth":{"accessToken":"a","refreshToken":"r","expiresAt":1720000000000}}"#;
        let t = parse_claude_keychain(j).unwrap();
        assert_eq!(t.access_token, "a");
        assert_eq!(t.refresh_token, "r");
        assert_eq!(t.expires_at_ms, 1720000000000);
    }

    #[test]
    fn parses_codex_authfile_json() {
        let j = r#"{"tokens":{"access_token":"a","refresh_token":"r","account_id":"acct_1"}}"#;
        let t = parse_codex_authfile(j).unwrap();
        assert_eq!(t.access_token, "a");
        assert_eq!(t.account_id.as_deref(), Some("acct_1"));
    }

    #[test]
    fn valid_token_skips_refresh() {
        // 만료가 충분히 미래면 needs_refresh=false → 네트워크 불필요
        let future = 4_000_000_000_000;
        assert!(!needs_refresh(future, now_ms(), 60_000));
    }
}
