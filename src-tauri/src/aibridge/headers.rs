//! 업스트림으로 나가기 직전, 원격이 보낸 더미 인증을 실제 OAuth 토큰으로 교체한다.
use http::HeaderMap;

const ANTHROPIC_OAUTH_BETA: &str = "oauth-2025-04-20";

pub fn apply_anthropic_auth(headers: &mut HeaderMap, access_token: &str) {
    headers.remove("authorization");
    headers.remove("x-api-key");
    headers.insert(
        "authorization",
        format!("Bearer {access_token}").parse().expect("valid header"),
    );
    let merged = match headers.get("anthropic-beta").and_then(|v| v.to_str().ok()) {
        Some(existing) if existing.split(',').any(|p| p.trim() == ANTHROPIC_OAUTH_BETA) => {
            existing.to_string()
        }
        Some(existing) if !existing.is_empty() => format!("{existing},{ANTHROPIC_OAUTH_BETA}"),
        _ => ANTHROPIC_OAUTH_BETA.to_string(),
    };
    headers.insert("anthropic-beta", merged.parse().expect("valid header"));
}

pub fn apply_openai_auth(headers: &mut HeaderMap, access_token: &str, account_id: Option<&str>) {
    headers.remove("authorization");
    headers.remove("x-api-key");
    headers.insert(
        "authorization",
        format!("Bearer {access_token}").parse().expect("valid header"),
    );
    if let Some(acct) = account_id {
        headers.insert("chatgpt-account-id", acct.parse().expect("valid header"));
    }
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
        apply_anthropic_auth(&mut h, "real-token");
        assert_eq!(h.get("authorization").unwrap(), "Bearer real-token");
        assert!(h.get("x-api-key").is_none());
        assert!(h.get("anthropic-beta").unwrap().to_str().unwrap().contains("oauth-2025-04-20"));
    }

    #[test]
    fn anthropic_merges_existing_beta_without_dup() {
        let mut h = HeaderMap::new();
        h.insert("anthropic-beta", "messages-2023-12-15".parse().unwrap());
        apply_anthropic_auth(&mut h, "t");
        let beta = h.get("anthropic-beta").unwrap().to_str().unwrap().to_string();
        assert!(beta.contains("messages-2023-12-15"));
        assert!(beta.contains("oauth-2025-04-20"));
        apply_anthropic_auth(&mut h, "t"); // 두 번 적용해도 중복 없음
        assert_eq!(h.get("anthropic-beta").unwrap().to_str().unwrap().matches("oauth-2025-04-20").count(), 1);
    }

    #[test]
    fn openai_sets_bearer_and_account() {
        let mut h = HeaderMap::new();
        h.insert("authorization", "Bearer bridge".parse().unwrap());
        apply_openai_auth(&mut h, "tok", Some("acct_123"));
        assert_eq!(h.get("authorization").unwrap(), "Bearer tok");
        assert_eq!(h.get("chatgpt-account-id").unwrap(), "acct_123");
    }
}
