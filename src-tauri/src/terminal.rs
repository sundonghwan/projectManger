// SSH 터미널 — 시스템 ssh 를 PTY로 실행하고 입출력을 프론트(xterm.js)와 스트리밍.
// 키 기반 인증을 기본으로 하고, 비밀번호 인증은 OS 키체인의 값을 사용한다.
use crate::error::{AppError, Result};
use crate::models::ServerConnection;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

/// ssh 실행 인자 구성 (순수 함수 — 테스트 대상).
pub fn build_ssh_args(server: &ServerConnection) -> Vec<String> {
    let mut args = vec![
        "-tt".to_string(),
        "-p".to_string(),
        server.port.to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
    ];
    if let Some(key) = &server.key_path {
        if !key.trim().is_empty() {
            args.push("-i".to_string());
            args.push(key.clone());
        }
    }
    args.push(format!("{}@{}", server.username, server.host));
    args
}

struct Session {
    writer: Box<dyn Write + Send>,
    master: Box<dyn portable_pty::MasterPty + Send>,
}

#[derive(Default)]
pub struct TerminalManager {
    sessions: Mutex<HashMap<i64, Session>>,
}

pub fn connect(app: &AppHandle, manager: &TerminalManager, server: &ServerConnection) -> Result<()> {
    let pty = native_pty_system();
    let pair = pty
        .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| AppError::Invalid(format!("PTY 생성 실패: {e}")))?;

    let mut cmd = CommandBuilder::new("ssh");
    for a in build_ssh_args(server) {
        cmd.arg(a);
    }

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

    let id = server.id;
    let app2 = app.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app2.emit(&format!("terminal://data/{id}"), chunk);
                }
                Err(_) => break,
            }
        }
        let _ = child.wait();
        let _ = app2.emit(&format!("terminal://exit/{id}"), ());
    });

    manager
        .sessions
        .lock()
        .unwrap()
        .insert(id, Session { writer, master: pair.master });
    Ok(())
}

pub fn write(manager: &TerminalManager, id: i64, data: &str) -> Result<()> {
    let mut sessions = manager.sessions.lock().unwrap();
    let s = sessions.get_mut(&id).ok_or(AppError::NotFound)?;
    s.writer
        .write_all(data.as_bytes())
        .map_err(|e| AppError::Invalid(format!("쓰기 실패: {e}")))?;
    s.writer.flush().ok();
    Ok(())
}

pub fn resize(manager: &TerminalManager, id: i64, rows: u16, cols: u16) -> Result<()> {
    let sessions = manager.sessions.lock().unwrap();
    let s = sessions.get(&id).ok_or(AppError::NotFound)?;
    s.master
        .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| AppError::Invalid(format!("리사이즈 실패: {e}")))
}

pub fn disconnect(manager: &TerminalManager, id: i64) -> Result<()> {
    manager.sessions.lock().unwrap().remove(&id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server() -> ServerConnection {
        ServerConnection {
            id: 1,
            business_id: 1,
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
        }
    }

    #[test]
    fn ssh_args_include_port_user_host_and_key() {
        let args = build_ssh_args(&server());
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
        let args = build_ssh_args(&s);
        assert!(!args.contains(&"-i".to_string()));
    }
}
