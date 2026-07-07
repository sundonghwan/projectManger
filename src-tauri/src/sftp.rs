// SFTP 파일 브라우징 — 시스템 sftp 를 배치 모드로 실행해 디렉터리를 나열.
// 키 기반(BatchMode) 인증 전제. ls -l 출력 파싱이 핵심 로직(테스트 대상).
use crate::error::{AppError, Result};
use crate::store::model::Server;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: i64,
}

/// `ls -l` 한 줄을 파싱. 엔트리가 아니면(프롬프트/헤더/.//..) None.
pub fn parse_ls_line(line: &str) -> Option<SftpEntry> {
    let line = line.trim();
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }
    let perms = parts[0];
    let first = perms.chars().next()?;
    if !matches!(first, 'd' | '-' | 'l') {
        return None;
    }
    let size = parts[4].parse::<i64>().ok()?;
    let name = parts[8..].join(" ");
    if name == "." || name == ".." || name.is_empty() {
        return None;
    }
    Some(SftpEntry {
        name,
        is_dir: first == 'd',
        size,
    })
}

pub fn parse_listing(out: &str) -> Vec<SftpEntry> {
    out.lines().filter_map(parse_ls_line).collect()
}

pub fn build_sftp_args(server: &Server, known_hosts: Option<&str>) -> Vec<String> {
    let mut args = vec![
        "-b".to_string(),
        "-".to_string(),
        "-P".to_string(),
        server.port.to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=yes".to_string(),
        "-o".to_string(),
        "BatchMode=yes".to_string(),
    ];
    if let Some(kh) = known_hosts {
        args.push("-o".to_string());
        args.push(format!(
            "UserKnownHostsFile={}",
            crate::hostkey::escape_config_value(kh)
        ));
    }
    if let Some(k) = &server.key_path {
        if !k.trim().is_empty() {
            args.push("-i".to_string());
            args.push(k.clone());
        }
    }
    args.push(format!("{}@{}", server.username, server.host));
    args
}

/// sftp batch 명령(`ls -l "<path>"`) 문맥 주입 방지(CWE-78 관련): 따옴표·백슬래시·개행·
/// 기타 제어문자를 제거한다. '"' 만 막으면 '\' 로 인용 구조를 교란할 수 있어 '\\' 도 제거.
/// 결과가 비면 "." 로 대체.
pub fn sanitize_remote_path(path: &str) -> String {
    let safe: String = path
        .chars()
        .filter(|c| *c != '"' && *c != '\\' && !c.is_control())
        .collect();
    if safe.trim().is_empty() {
        ".".to_string()
    } else {
        safe
    }
}

/// 원격 디렉터리 나열. (키 기반 인증 필요 — 실 서버 대상 검증 요)
pub fn list(server: &Server, path: &str, known_hosts: Option<&str>) -> Result<Vec<SftpEntry>> {
    let safe = sanitize_remote_path(path);

    let mut child = Command::new("sftp")
        .args(build_sftp_args(server, known_hosts))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Invalid(format!("sftp 실행 실패: {e}")))?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| AppError::Invalid("sftp stdin 오류".into()))?;
        let batch = format!("ls -l \"{safe}\"\nquit\n");
        stdin
            .write_all(batch.as_bytes())
            .map_err(|e| AppError::Invalid(format!("sftp 입력 실패: {e}")))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|e| AppError::Invalid(format!("sftp 종료 오류: {e}")))?;
    if !output.status.success() && output.stdout.is_empty() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Invalid(format!("sftp 실패: {}", err.trim())));
    }
    Ok(parse_listing(&String::from_utf8_lossy(&output.stdout)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server() -> Server {
        Server {
            id: "1".into(),
            business_id: "b1".into(),
            project_id: None,
            name: "s".into(),
            host: "h".into(),
            port: 2222,
            username: "u".into(),
            auth_type: "key".into(),
            key_path: Some("/k".into()),
            secret_ref: None,
            last_used_at: None,
            archived_at: None,
            created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-01T00:00:00.000Z".into(),
        }
    }

    #[test]
    fn parses_dir_and_file() {
        let d = parse_ls_line("drwxr-xr-x 2 u g 4096 Jun 20 10:00 src").unwrap();
        assert_eq!(d.name, "src");
        assert!(d.is_dir);
        let f = parse_ls_line("-rw-r--r-- 1 u g 123 Jun 20 10:00 README.md").unwrap();
        assert_eq!(f.name, "README.md");
        assert!(!f.is_dir);
        assert_eq!(f.size, 123);
    }

    #[test]
    fn skips_prompts_dots_and_garbage() {
        assert!(parse_ls_line("sftp> ls -l").is_none());
        assert!(parse_ls_line("drwxr-xr-x 2 u g 4096 Jun 20 10:00 .").is_none());
        assert!(parse_ls_line("-rw-r--r-- 1 u g 0 Jun 20 10:00 ..").is_none());
        assert!(parse_ls_line("").is_none());
        assert!(parse_ls_line("random text").is_none());
    }

    #[test]
    fn parses_names_with_spaces() {
        let e = parse_ls_line("-rw-r--r-- 1 u g 10 Jun 20 10:00 my file.txt").unwrap();
        assert_eq!(e.name, "my file.txt");
    }

    #[test]
    fn parse_listing_filters() {
        let out = "sftp> ls -l /home\ndrwxr-xr-x 2 u g 4096 Jun 20 10:00 docs\n-rw-r--r-- 1 u g 5 Jun 20 10:00 a.txt\n";
        let entries = parse_listing(out);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn sanitize_strips_quote_backslash_control_and_defaults() {
        assert_eq!(sanitize_remote_path("/var/www"), "/var/www");
        // 따옴표·백슬래시·개행 제거
        assert_eq!(sanitize_remote_path("a\"b\\c\nd"), "abcd");
        // 제어문자 제거
        assert_eq!(sanitize_remote_path("x\u{7}y"), "xy");
        // 빈/공백 → "."
        assert_eq!(sanitize_remote_path("   "), ".");
    }

    #[test]
    fn sftp_args_have_batch_port_key() {
        let a = build_sftp_args(&server(), Some("/tmp/kh"));
        assert!(a.contains(&"BatchMode=yes".to_string()));
        assert!(a.contains(&"StrictHostKeyChecking=yes".to_string()));
        assert!(a.contains(&"UserKnownHostsFile=/tmp/kh".to_string()));
        assert!(a.contains(&"2222".to_string()));
        assert!(a.contains(&"u@h".to_string()));
        assert!(a.contains(&"/k".to_string()));
    }

    #[test]
    fn sftp_args_escape_known_hosts_path_for_openssh_config_parser() {
        let a = build_sftp_args(
            &server(),
            Some("/Users/polaris/Library/Application Support/com.app/known_hosts"),
        );
        assert!(a.contains(
            &"UserKnownHostsFile=/Users/polaris/Library/Application\\ Support/com.app/known_hosts"
                .to_string()
        ));
    }
}
