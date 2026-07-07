// SSH 터미널 — 시스템 ssh 를 PTY로 실행하고 입출력을 프론트(xterm.js)와 스트리밍.
// 키 기반 인증을 기본으로 하고, 비밀번호 인증은 OS 키체인의 값을 사용한다.
use crate::error::{AppError, Result};
use crate::store::model::Server;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

/// ssh 실행 인자 구성 (순수 함수 — 테스트 대상).
/// 보안(CWE-295): 호스트 키를 엄격 검증(`StrictHostKeyChecking=yes`)하고, 앱 전용
/// known_hosts 파일을 사용한다. 처음 보는 호스트는 상위 계층(지문 확인 UI)에서 먼저
/// 신뢰 등록한 뒤 접속하므로, 여기서 `yes` 로 두어도 정상 접속된다.
pub fn build_ssh_args(server: &Server, known_hosts: Option<&str>) -> Vec<String> {
    let mut args = vec![
        "-tt".to_string(),
        "-p".to_string(),
        server.port.to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=yes".to_string(),
    ];
    if let Some(kh) = known_hosts {
        args.push("-o".to_string());
        args.push(format!(
            "UserKnownHostsFile={}",
            crate::hostkey::escape_config_value(kh)
        ));
    }
    if let Some(key) = &server.key_path {
        if !key.trim().is_empty() {
            args.push("-i".to_string());
            args.push(key.clone());
        }
    }
    args.push(format!("{}@{}", server.username, server.host));
    args
}

fn build_ssh_command(server: &Server, known_hosts: Option<&str>) -> CommandBuilder {
    let mut cmd = CommandBuilder::new("ssh");
    cmd.env("TERM", "xterm-256color");
    for a in build_ssh_args(server, known_hosts) {
        cmd.arg(a);
    }
    cmd
}

struct Session {
    writer: Box<dyn Write + Send>,
    master: Box<dyn portable_pty::MasterPty + Send>,
}

#[derive(Default)]
pub struct TerminalManager {
    sessions: Mutex<HashMap<String, Session>>,
}

pub fn connect(app: &AppHandle, manager: &TerminalManager, server: &Server) -> Result<()> {
    let pty = native_pty_system();
    let pair = pty
        .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| AppError::Invalid(format!("PTY 생성 실패: {e}")))?;

    let known_hosts = crate::hostkey::known_hosts_path(app).map(|p| p.to_string_lossy().to_string());
    let cmd = build_ssh_command(server, known_hosts.as_deref());

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| AppError::Invalid(format!("ssh 실행 실패: {e}")))?;
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| AppError::Invalid(format!("PTY 리더 오류: {e}")))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| AppError::Invalid(format!("PTY 라이터 오류: {e}")))?;

    let id = server.id.clone();
    let id2 = id.clone();
    let app2 = app.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app2.emit(&format!("terminal://data/{id2}"), chunk);
                }
                Err(_) => break,
            }
        }
        let _ = child.wait();
        let _ = app2.emit(&format!("terminal://exit/{id2}"), ());
    });

    manager
        .sessions
        .lock()
        .unwrap()
        .insert(id, Session { writer, master: pair.master });
    Ok(())
}

pub fn write(manager: &TerminalManager, id: &str, data: &str) -> Result<()> {
    let mut sessions = manager.sessions.lock().unwrap();
    let s = sessions.get_mut(id).ok_or(AppError::NotFound)?;
    s.writer
        .write_all(data.as_bytes())
        .map_err(|e| AppError::Invalid(format!("쓰기 실패: {e}")))?;
    s.writer.flush().ok();
    Ok(())
}

pub fn resize(manager: &TerminalManager, id: &str, rows: u16, cols: u16) -> Result<()> {
    let sessions = manager.sessions.lock().unwrap();
    let s = sessions.get(id).ok_or(AppError::NotFound)?;
    s.master
        .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| AppError::Invalid(format!("리사이즈 실패: {e}")))
}

pub fn disconnect(manager: &TerminalManager, id: &str) -> Result<()> {
    manager.sessions.lock().unwrap().remove(id);
    Ok(())
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
            host: "example.com".into(),
            port: 2222,
            username: "deploy".into(),
            auth_type: "key".into(),
            key_path: Some("/home/u/.ssh/id_ed25519".into()),
            secret_ref: None,
            last_used_at: None,
            archived_at: None,
            created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-01T00:00:00.000Z".into(),
        }
    }

    #[test]
    fn ssh_args_include_port_user_host_and_key() {
        let args = build_ssh_args(&server(), None);
        assert!(args.contains(&"-p".to_string()));
        assert!(args.contains(&"2222".to_string()));
        assert!(args.contains(&"deploy@example.com".to_string()));
        assert!(args.contains(&"-i".to_string()));
        assert!(args.contains(&"/home/u/.ssh/id_ed25519".to_string()));
    }

    #[test]
    fn ssh_args_omit_key_when_absent() {
        let mut s = server();
        s.key_path = None;
        let args = build_ssh_args(&s, None);
        assert!(!args.contains(&"-i".to_string()));
    }

    #[test]
    fn ssh_args_strict_checking_and_known_hosts() {
        let args = build_ssh_args(&server(), Some("/tmp/kh"));
        assert!(args.contains(&"StrictHostKeyChecking=yes".to_string()));
        assert!(args.contains(&"UserKnownHostsFile=/tmp/kh".to_string()));
        assert!(!args.contains(&"StrictHostKeyChecking=accept-new".to_string()));
    }

    #[test]
    fn ssh_args_escape_known_hosts_path_for_openssh_config_parser() {
        let args = build_ssh_args(
            &server(),
            Some("/Users/polaris/Library/Application Support/com.app/known_hosts"),
        );
        assert!(args.contains(
            &"UserKnownHostsFile=/Users/polaris/Library/Application\\ Support/com.app/known_hosts"
                .to_string()
        ));
    }

    #[test]
    fn ssh_command_sets_interactive_terminal_type() {
        let cmd = build_ssh_command(&server(), Some("/tmp/kh"));
        assert_eq!(
            cmd.get_env("TERM").and_then(|v| v.to_str()),
            Some("xterm-256color")
        );
    }
}
