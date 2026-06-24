// SSH 호스트 키 신뢰 관리 — 앱 전용 known_hosts 로 MITM(CWE-295) 방어.
// 처음 보는 호스트는 ssh-keyscan 으로 공개키를 가져와 지문을 사용자에게 보여주고,
// 수락받은 키만 known_hosts 에 추가한다(TOFU 의 첫-연결 위험 제거).
use crate::error::{AppError, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// 앱 데이터 폴더의 known_hosts 경로. (`<appData>/known_hosts`)
pub fn known_hosts_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    use tauri::Manager;
    let dir = app.path().app_data_dir().ok()?;
    std::fs::create_dir_all(&dir).ok();
    Some(dir.join("known_hosts"))
}

/// ssh-keygen -F 의 호스트 스펙. 표준 포트가 아니면 `[host]:port` 형식.
fn host_spec(host: &str, port: i64) -> String {
    if port == 22 {
        host.to_string()
    } else {
        format!("[{host}]:{port}")
    }
}

/// 해당 호스트가 이미 신뢰(known_hosts 등록)되어 있는지.
pub fn is_known(known_hosts: &Path, host: &str, port: i64) -> bool {
    if !known_hosts.exists() {
        return false;
    }
    Command::new("ssh-keygen")
        .arg("-F")
        .arg(host_spec(host, port))
        .arg("-f")
        .arg(known_hosts)
        .output()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false)
}

/// ssh-keyscan 으로 호스트 공개키를 가져오고 지문을 계산해 반환. (네트워크 필요)
/// 반환: (사용자에게 보여줄 지문 문자열, known_hosts 에 추가할 키 라인)
pub fn scan(host: &str, port: i64) -> Result<(String, String)> {
    let out = Command::new("ssh-keyscan")
        .arg("-T")
        .arg("5")
        .arg("-p")
        .arg(port.to_string())
        .arg(host)
        .output()
        .map_err(|e| AppError::Invalid(format!("ssh-keyscan 실행 실패: {e}")))?;
    let raw = String::from_utf8_lossy(&out.stdout);
    let keys: String = raw
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') && !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if keys.is_empty() {
        return Err(AppError::Invalid(
            "호스트 키를 가져오지 못했습니다(호스트/포트/네트워크 확인)".into(),
        ));
    }
    let fp = fingerprint(&keys)?;
    Ok((fp, keys))
}

/// 키 라인들의 지문(ssh-keygen -lf -)을 계산.
fn fingerprint(keys: &str) -> Result<String> {
    let mut child = Command::new("ssh-keygen")
        .arg("-lf")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Invalid(format!("ssh-keygen 실행 실패: {e}")))?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| AppError::Invalid("ssh-keygen stdin 오류".into()))?
        .write_all(keys.as_bytes())
        .map_err(|e| AppError::Invalid(format!("ssh-keygen 입력 실패: {e}")))?;
    let out = child
        .wait_with_output()
        .map_err(|e| AppError::Invalid(format!("ssh-keygen 종료 오류: {e}")))?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// 사용자가 수락한 키 라인을 known_hosts 에 추가.
pub fn trust(known_hosts: &Path, key_lines: &str) -> Result<()> {
    if let Some(parent) = known_hosts.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(known_hosts)
        .map_err(|e| AppError::Invalid(format!("known_hosts 쓰기 실패: {e}")))?;
    writeln!(f, "{}", key_lines.trim_end())
        .map_err(|e| AppError::Invalid(format!("known_hosts 쓰기 실패: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_spec_uses_bracket_for_nonstandard_port() {
        assert_eq!(host_spec("h", 22), "h");
        assert_eq!(host_spec("h", 2222), "[h]:2222");
    }

    #[test]
    fn trust_appends_and_is_known_reflects_file_state() {
        let dir = std::env::temp_dir().join(format!("kh_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let kh = dir.join("known_hosts");
        // 빈 상태 → 미신뢰
        assert!(!is_known(&kh, "example.com", 22));
        // 실제 형식의 더미 키 라인 추가
        let line = "example.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIdummykeyfortest0000000000000000000000";
        trust(&kh, line).unwrap();
        let content = std::fs::read_to_string(&kh).unwrap();
        assert!(content.contains("example.com ssh-ed25519"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
