//! `cswap` (claude-swap) 연동 — 선택적 편의 기능. 원격 AI 자격증명 브리지 본체는
//! `cswap` 유무와 무관하게 동작해야 하며, 이 모듈은 사용자가 `cswap` 을 설치해 둔
//! 경우에만 계정 목록 조회/전환 UI를 제공하기 위한 것이다.
//!
//! `cswap --list --json` 의 실제 출력 스키마(검증됨):
//! ```json
//! {
//!   "schemaVersion": 1,
//!   "activeAccountNumber": 1,
//!   "accounts": [
//!     {
//!       "number": 1,
//!       "email": "user@example.com",
//!       "active": true,
//!       "usageStatus": "ok",
//!       "usage": {
//!         "fiveHour": { "pct": 11.0, ... },
//!         "sevenDay": { "pct": 15.0, ... }
//!       }
//!     }
//!   ]
//! }
//! ```
//! `token_state` 필드는 존재하지 않는다 — 초안 브리핑의 추정 스키마는 사용하지 않는다.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// cswap 계정 하나의 요약 상태. 프론트 표시에 필요한 필드만 추출해 보관한다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub number: u32,
    pub email: String,
    pub active: bool,
    pub usage_status: String,
    pub five_hour_pct: f64,
    pub seven_day_pct: f64,
}

/// `cswap --version` 이 성공 종료하면 설치된 것으로 간주한다.
pub fn available() -> bool {
    Command::new("cswap")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `cswap --list --json` 표준출력을 파싱해 계정 목록을 반환한다.
/// `usage`/`usageStatus` 등 일부 필드가 없어도 panic 하지 않고 기본값(0.0 / 빈 문자열)으로
/// 채운다. `accounts` 배열 자체가 없으면 에러.
pub fn parse_list(json: &str) -> Result<Vec<Account>> {
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| AppError::Invalid(format!("cswap json 파싱: {e}")))?;
    let arr = v
        .get("accounts")
        .and_then(|a| a.as_array())
        .ok_or_else(|| AppError::Invalid("cswap json: accounts 없음".into()))?;

    Ok(arr
        .iter()
        .filter_map(|a| {
            let number = a.get("number")?.as_u64()? as u32;
            let email = a.get("email")?.as_str()?.to_string();
            let active = a.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
            let usage_status = a
                .get("usageStatus")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let five_hour_pct = a
                .get("usage")
                .and_then(|u| u.get("fiveHour"))
                .and_then(|f| f.get("pct"))
                .and_then(|p| p.as_f64())
                .unwrap_or(0.0);
            let seven_day_pct = a
                .get("usage")
                .and_then(|u| u.get("sevenDay"))
                .and_then(|s| s.get("pct"))
                .and_then(|p| p.as_f64())
                .unwrap_or(0.0);
            Some(Account {
                number,
                email,
                active,
                usage_status,
                five_hour_pct,
                seven_day_pct,
            })
        })
        .collect())
}

/// `cswap --list --json` 을 실행해 계정 목록을 조회한다 (`--token-status` 는 실제 스키마에
/// 존재하지 않으므로 사용하지 않는다).
pub fn list() -> Result<Vec<Account>> {
    let out = Command::new("cswap")
        .args(["--list", "--json"])
        .output()
        .map_err(|e| AppError::Invalid(format!("cswap 실행 실패: {e}")))?;
    parse_list(&String::from_utf8_lossy(&out.stdout))
}

/// `cswap --switch-to <target>` 으로 활성 계정을 전환한다.
pub fn switch_to(target: &str) -> Result<()> {
    let out = Command::new("cswap")
        .args(["--switch-to", target])
        .output()
        .map_err(|e| AppError::Invalid(format!("cswap 전환 실패: {e}")))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(AppError::Invalid(format!(
            "cswap 전환 실패: {}",
            String::from_utf8_lossy(&out.stderr)
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
        "schemaVersion": 1,
        "activeAccountNumber": 1,
        "accounts": [
            {
                "number": 1,
                "email": "a@x.com",
                "organizationName": "Acme",
                "organizationUuid": "uuid-1",
                "isOrganization": true,
                "active": true,
                "usageStatus": "ok",
                "usage": {
                    "fiveHour": { "pct": 11.0, "resetsAt": "t1", "countdown": "0m", "clock": "13:40" },
                    "sevenDay": { "pct": 15.0, "resetsAt": "t2", "countdown": "4d 4h", "clock": "c" },
                    "scoped": [ { "pct": 2.0, "name": "Fable" } ]
                },
                "usageFetchedAt": "t3",
                "usageAgeSeconds": 42
            },
            {
                "number": 2,
                "email": "b@x.com",
                "organizationName": null,
                "organizationUuid": null,
                "isOrganization": false,
                "active": false,
                "usageStatus": "warning",
                "usage": {
                    "fiveHour": { "pct": 80.0, "resetsAt": "t4", "countdown": "1h", "clock": "14:00" },
                    "sevenDay": { "pct": 60.0, "resetsAt": "t5", "countdown": "1d", "clock": "c2" }
                },
                "usageFetchedAt": "t6",
                "usageAgeSeconds": 10
            }
        ]
    }"#;

    #[test]
    fn parses_real_cswap_list_json_schema() {
        let v = parse_list(FIXTURE).unwrap();
        assert_eq!(v.len(), 2);

        assert_eq!(v[0].number, 1);
        assert_eq!(v[0].email, "a@x.com");
        assert!(v[0].active);
        assert_eq!(v[0].usage_status, "ok");
        assert_eq!(v[0].five_hour_pct, 11.0);
        assert_eq!(v[0].seven_day_pct, 15.0);

        assert_eq!(v[1].number, 2);
        assert_eq!(v[1].email, "b@x.com");
        assert!(!v[1].active);
        assert_eq!(v[1].usage_status, "warning");
        assert_eq!(v[1].five_hour_pct, 80.0);
        assert_eq!(v[1].seven_day_pct, 60.0);
    }

    #[test]
    fn missing_accounts_key_is_error() {
        let err = parse_list(r#"{"schemaVersion":1}"#).unwrap_err();
        assert!(err.to_string().contains("accounts"));
    }

    #[test]
    fn missing_usage_fields_default_gracefully() {
        let j = r#"{"accounts":[{"number":3,"email":"c@x.com","active":false}]}"#;
        let v = parse_list(j).unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].usage_status, "");
        assert_eq!(v[0].five_hour_pct, 0.0);
        assert_eq!(v[0].seven_day_pct, 0.0);
    }

    #[test]
    fn account_missing_required_number_or_email_is_skipped_not_panicking() {
        let j = r#"{"accounts":[{"email":"no-number@x.com"},{"number":9}]}"#;
        let v = parse_list(j).unwrap();
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn invalid_json_is_error_not_panic() {
        let err = parse_list("not json").unwrap_err();
        assert!(matches!(err, AppError::Invalid(_)));
    }
}
