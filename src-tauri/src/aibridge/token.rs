//! OAuth 토큰 모델과 refresh 판단. 저장소 I/O 는 어댑터(별도 태스크)에서.
use crate::error::{AppError, Result};
use keyring::Entry;
use serde_json::Value;
use std::sync::LazyLock;
use tokio::sync::Mutex;

/// per-provider 단일 비행(single-flight) 락. 동시 만료 요청이 각자 refresh 하며
/// refresh-token 회전으로 서로를 무효화 + 키체인/auth.json 되쓰기를 덮어써 로컬 CLI
/// 로그인을 손상시키는 레이스를 막는다. 이중 검사(double-checked) 패턴으로 대기자는
/// 앞선 refresh 결과를 재사용한다.
static CLAUDE_REFRESH_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
static CODEX_REFRESH_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

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

/// 원본 키체인 JSON 을 보존한 채 claudeAiOauth 의 토큰 3개 필드만 갱신한다.
pub fn merge_claude_refresh(raw: &str, refreshed: &OauthToken) -> Result<String> {
    let mut root: Value = serde_json::from_str(raw)
        .map_err(|e| AppError::Invalid(format!("claude 토큰 병합 파싱: {e}")))?;
    let obj = root.get_mut("claudeAiOauth").and_then(|v| v.as_object_mut())
        .ok_or_else(|| AppError::Invalid("claudeAiOauth 객체 없음".into()))?;
    obj.insert("accessToken".into(), serde_json::json!(refreshed.access_token));
    obj.insert("refreshToken".into(), serde_json::json!(refreshed.refresh_token));
    obj.insert("expiresAt".into(), serde_json::json!(refreshed.expires_at_ms));
    Ok(root.to_string())
}

/// 원본 auth.json 을 보존한 채 tokens.access_token / tokens.refresh_token 만 갱신한다.
/// (auth_mode, OPENAI_API_KEY, id_token, account_id, last_refresh 등은 그대로 유지)
pub fn merge_codex_refresh(raw: &str, refreshed: &OauthToken) -> Result<String> {
    let mut root: Value = serde_json::from_str(raw)
        .map_err(|e| AppError::Invalid(format!("codex 토큰 병합 파싱: {e}")))?;
    let obj = root.get_mut("tokens").and_then(|v| v.as_object_mut())
        .ok_or_else(|| AppError::Invalid("tokens 객체 없음".into()))?;
    obj.insert("access_token".into(), serde_json::json!(refreshed.access_token));
    obj.insert("refresh_token".into(), serde_json::json!(refreshed.refresh_token));
    Ok(root.to_string())
}

/// base64url(패딩 有/無 모두 허용) 디코드. 표준 base64 문자셋(`+`/`/`) 은 다루지 않는다(JWT 는 url-safe 사용).
fn b64url_decode(s: &str) -> Result<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let s = s.trim_end_matches('=');
    let mut out = Vec::with_capacity(s.len() * 3 / 4 + 3);
    let mut buf = 0u32;
    let mut bits = 0u32;
    for c in s.bytes() {
        let idx = ALPHABET.iter().position(|&a| a == c)
            .ok_or_else(|| AppError::Invalid("base64url 디코드 실패: 잘못된 문자".into()))?;
        buf = (buf << 6) | idx as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
}

fn jwt_payload_json(jwt: &str) -> Result<Value> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() < 2 {
        return Err(AppError::Invalid("JWT 형식 오류: 세그먼트 부족".into()));
    }
    let raw = b64url_decode(parts[1])?;
    serde_json::from_slice(&raw).map_err(|e| AppError::Invalid(format!("JWT payload 파싱: {e}")))
}

/// JWT 의 `exp` claim(유닉스 초)을 밀리초로 환산해 반환한다.
pub fn jwt_exp_ms(jwt: &str) -> Result<i64> {
    let payload = jwt_payload_json(jwt)?;
    let exp = payload.get("exp").and_then(|x| x.as_i64())
        .ok_or_else(|| AppError::Invalid("JWT 에 exp claim 없음".into()))?;
    Ok(exp * 1000)
}

/// JWT 의 `client_id` claim 을 반환한다 (refresh 요청 조립용).
pub fn jwt_client_id(jwt: &str) -> Result<String> {
    let payload = jwt_payload_json(jwt)?;
    s(&payload, "client_id")
}

const CLAUDE_KEYCHAIN_SERVICE: &str = "Claude Code-credentials";
const REFRESH_SKEW_MS: i64 = 60_000;
/// UNVERIFIED (user must confirm via real codex traffic): refresh 엔드포인트.
const CODEX_REFRESH_URL: &str = "https://auth.openai.com/oauth/token";

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

    fn load(&self) -> Result<(String, OauthToken)> {
        let raw = Self::entry()?
            .get_password()
            .map_err(|e| AppError::Invalid(format!("claude 자격증명 로드 실패: {e}")))?;
        let tok = parse_claude_keychain(&raw)?;
        Ok((raw, tok))
    }

    pub async fn access_token(&self) -> Result<String> {
        // 1) fast path: 락 없이 로드 후 신선하면 즉시 반환
        let (_, tok) = self.load()?;
        if !needs_refresh(tok.expires_at_ms, now_ms(), REFRESH_SKEW_MS) {
            return Ok(tok.access_token);
        }
        // 2) refresh 필요 — 단일 비행 락 획득 후 이중 검사(다른 태스크가 방금 refresh 했을 수 있음)
        let _g = CLAUDE_REFRESH_LOCK.lock().await;
        let (raw, tok) = self.load()?;
        if !needs_refresh(tok.expires_at_ms, now_ms(), REFRESH_SKEW_MS) {
            return Ok(tok.access_token);
        }
        let refreshed = refresh_claude(&tok.refresh_token).await?;
        // 키체인 되쓰기 (로컬 세션 동기 유지) — 원본 필드 보존, 토큰 3개만 갱신
        let write = merge_claude_refresh(&raw, &refreshed)?;
        Self::entry()?.set_password(&write)
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

/// `~/.codex/auth.json` 기반 Codex(ChatGPT OAuth) 토큰 소스.
/// 이 파일에는 `expiresAt` 필드가 없다 — 만료는 `tokens.access_token` (JWT) 의 `exp` claim 에서 계산한다.
pub struct CodexTokenSource;

impl CodexTokenSource {
    fn auth_path() -> Result<std::path::PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| AppError::Invalid("HOME 환경변수 없음".into()))?;
        Ok(std::path::PathBuf::from(home).join(".codex").join("auth.json"))
    }

    fn read_raw() -> Result<String> {
        let path = Self::auth_path()?;
        std::fs::read_to_string(&path)
            .map_err(|e| AppError::Invalid(format!("codex 자격증명 로드 실패: {e}")))
    }

    fn write_raw(raw: &str) -> Result<()> {
        let path = Self::auth_path()?;
        std::fs::write(&path, raw)
            .map_err(|e| AppError::Invalid(format!("codex 자격증명 저장 실패: {e}")))
    }

    fn load() -> Result<(String, OauthToken, i64)> {
        let raw = Self::read_raw()?;
        let tok = parse_codex_authfile(&raw)?;
        let exp_ms = jwt_exp_ms(&tok.access_token)?;
        Ok((raw, tok, exp_ms))
    }

    /// 유효한 access_token 과 (있다면) account_id 를 반환한다. 필요 시 refresh 후 파일에 되쓴다.
    pub async fn access_and_account(&self) -> Result<(String, Option<String>)> {
        // 1) fast path: 락 없이 로드 후 신선하면 즉시 반환
        let (_, tok, exp_ms) = Self::load()?;
        if !needs_refresh(exp_ms, now_ms(), REFRESH_SKEW_MS) {
            return Ok((tok.access_token, tok.account_id));
        }
        // 2) refresh 필요 — 단일 비행 락 획득 후 이중 검사(다른 태스크가 방금 refresh 했을 수 있음)
        let _g = CODEX_REFRESH_LOCK.lock().await;
        let (raw, tok, exp_ms) = Self::load()?;
        if !needs_refresh(exp_ms, now_ms(), REFRESH_SKEW_MS) {
            return Ok((tok.access_token, tok.account_id));
        }
        let client_id = jwt_client_id(&tok.access_token)?;
        let refreshed = refresh_codex(&tok.refresh_token, &client_id).await?;
        let write = merge_codex_refresh(&raw, &refreshed)?;
        Self::write_raw(&write)?;
        Ok((refreshed.access_token, tok.account_id))
    }
}

/// UNVERIFIED (user must confirm via real codex traffic): 엔드포인트/파라미터가 실제 Codex
/// OAuth refresh 흐름과 일치하는지 실측 검증 전. 라이브 호출은 이 태스크 범위에서 실행하지 않는다.
async fn refresh_codex(refresh_token: &str, client_id: &str) -> Result<OauthToken> {
    let body = serde_json::json!({
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
        "client_id": client_id,
    });
    let resp = reqwest::Client::new()
        .post(CODEX_REFRESH_URL)
        .json(&body).send().await
        .map_err(|e| AppError::Invalid(format!("codex refresh 요청 실패: {e}")))?;
    let v: Value = resp.json().await
        .map_err(|e| AppError::Invalid(format!("codex refresh 응답 파싱: {e}")))?;
    let access_token = s(&v, "access_token")?;
    let expires_at_ms = jwt_exp_ms(&access_token).unwrap_or_else(|_| now_ms() + 3_600_000);
    Ok(OauthToken {
        access_token,
        refresh_token: v.get("refresh_token").and_then(|x| x.as_str())
            .map(str::to_string).unwrap_or_else(|| refresh_token.to_string()),
        expires_at_ms,
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

    #[test]
    fn merge_preserves_extra_fields_and_updates_tokens() {
        let raw = r#"{"claudeAiOauth":{"accessToken":"old","refreshToken":"oldr","expiresAt":1,"subscriptionType":"max","scopes":["a"]}}"#;
        let refreshed = OauthToken { access_token: "new".into(), refresh_token: "newr".into(), expires_at_ms: 999, account_id: None };
        let out = merge_claude_refresh(raw, &refreshed).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let o = &v["claudeAiOauth"];
        assert_eq!(o["accessToken"], "new");
        assert_eq!(o["refreshToken"], "newr");
        assert_eq!(o["expiresAt"], 999);
        assert_eq!(o["subscriptionType"], "max");   // extra field preserved
        assert_eq!(o["scopes"][0], "a");            // extra field preserved
    }

    /// base64url(no padding) 인코더 — 테스트용 JWT 조립 전용 (프로덕션 디코더와는 별개).
    fn b64url_encode(bytes: &[u8]) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = *chunk.get(1).unwrap_or(&0) as u32;
            let b2 = *chunk.get(2).unwrap_or(&0) as u32;
            let n = (b0 << 16) | (b1 << 8) | b2;
            let c0 = ALPHABET[((n >> 18) & 0x3F) as usize] as char;
            let c1 = ALPHABET[((n >> 12) & 0x3F) as usize] as char;
            let c2 = ALPHABET[((n >> 6) & 0x3F) as usize] as char;
            let c3 = ALPHABET[(n & 0x3F) as usize] as char;
            out.push(c0);
            out.push(c1);
            if chunk.len() > 1 { out.push(c2); }
            if chunk.len() > 2 { out.push(c3); }
        }
        out
    }

    fn make_test_jwt(payload_json: &str) -> String {
        let header = b64url_encode(br#"{"alg":"none"}"#);
        let payload = b64url_encode(payload_json.as_bytes());
        format!("{header}.{payload}.sig")
    }

    #[test]
    fn jwt_exp_ms_extracts_and_scales_to_millis() {
        let jwt = make_test_jwt(r#"{"exp":9999999999,"client_id":"x"}"#);
        let ms = jwt_exp_ms(&jwt).unwrap();
        assert_eq!(ms, 9_999_999_999 * 1000);
    }

    #[test]
    fn jwt_exp_ms_errors_on_malformed_jwt() {
        assert!(jwt_exp_ms("not-a-jwt").is_err());
        assert!(jwt_exp_ms("a.b").is_err()); // 세그먼트 부족
        let bad_payload = format!("h.{}.s", "not-base64url!!!");
        assert!(jwt_exp_ms(&bad_payload).is_err());
    }

    #[test]
    fn jwt_client_id_extracts_claim() {
        let jwt = make_test_jwt(r#"{"exp":1,"client_id":"abc-123"}"#);
        assert_eq!(jwt_client_id(&jwt).unwrap(), "abc-123");
    }

    #[test]
    fn merge_codex_preserves_extra_fields_and_updates_tokens() {
        let raw = r#"{"auth_mode":"chatgpt","OPENAI_API_KEY":null,"tokens":{"id_token":"idtok","access_token":"old","refresh_token":"oldr","account_id":"acct-xyz"},"last_refresh":"2026-01-01T00:00:00Z"}"#;
        let refreshed = OauthToken { access_token: "new".into(), refresh_token: "newr".into(), expires_at_ms: 999, account_id: None };
        let out = merge_codex_refresh(raw, &refreshed).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["tokens"]["access_token"], "new");
        assert_eq!(v["tokens"]["refresh_token"], "newr");
        assert_eq!(v["tokens"]["id_token"], "idtok");        // extra field preserved
        assert_eq!(v["tokens"]["account_id"], "acct-xyz");   // extra field preserved
        assert_eq!(v["auth_mode"], "chatgpt");               // top-level extra preserved
        assert_eq!(v["last_refresh"], "2026-01-01T00:00:00Z"); // top-level extra preserved
        assert!(v["OPENAI_API_KEY"].is_null());
    }
}
