//! OAuth 토큰 모델과 refresh 판단. 저장소 I/O 는 어댑터(별도 태스크)에서.
use crate::error::{AppError, Result};
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
}
