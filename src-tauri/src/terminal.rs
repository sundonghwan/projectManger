// SSH 터미널 — 시스템 ssh 를 PTY로 실행하고 입출력을 프론트(xterm.js)와 스트리밍.
// 키 기반 인증을 기본으로 하고, 비밀번호 인증은 OS 키체인의 값을 사용한다.
use crate::error::{AppError, Result};
use crate::store::model::Server;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

/// 원격 AI 자격증명 브리지의 SSH 리버스 포워드 포트 쌍.
/// `anthropic_*`/`openai_*` 각각 (원격 포트, 로컬 포트) — 로컬에서 대기 중인 프록시로
/// 원격 셸의 요청을 되돌려 보낸다 (Task 6에서 실제 값 주입).
#[derive(Clone, Copy, Debug)]
pub struct BridgePorts {
    pub anthropic_remote: u16,
    pub anthropic_local: u16,
    pub openai_remote: u16,
    pub openai_local: u16,
}

/// ssh 실행 인자 구성 (순수 함수 — 테스트 대상).
/// 보안(CWE-295): 호스트 키를 엄격 검증(`StrictHostKeyChecking=yes`)하고, 앱 전용
/// known_hosts 파일을 사용한다. 처음 보는 호스트는 상위 계층(지문 확인 UI)에서 먼저
/// 신뢰 등록한 뒤 접속하므로, 여기서 `yes` 로 두어도 정상 접속된다.
/// `bridge` 가 `Some` 이면 리버스 포워드(`-R`)와 원격 셸 실행 시 주입할 env 변수를
/// 담은 원격 명령을 추가한다. `None` 이면 기존 동작과 완전히 동일하다.
pub fn build_ssh_args(
    server: &Server,
    known_hosts: Option<&str>,
    bridge: Option<&BridgePorts>,
) -> Vec<String> {
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
    if let Some(b) = bridge {
        args.push("-R".to_string());
        args.push(format!(
            "127.0.0.1:{}:127.0.0.1:{}",
            b.anthropic_remote, b.anthropic_local
        ));
        args.push("-R".to_string());
        args.push(format!(
            "127.0.0.1:{}:127.0.0.1:{}",
            b.openai_remote, b.openai_local
        ));
    }
    args.push(format!("{}@{}", server.username, server.host));
    if let Some(b) = bridge {
        args.push(format!(
            "command -v claude >/dev/null 2>&1 || echo '[bridge] 원격에 claude 미설치'; \
             command -v codex >/dev/null 2>&1 || echo '[bridge] 원격에 codex 미설치'; \
             ANTHROPIC_BASE_URL=http://127.0.0.1:{a} OPENAI_BASE_URL=http://127.0.0.1:{o} ANTHROPIC_AUTH_TOKEN=bridge exec ${{SHELL:-/bin/sh}} -l",
            a = b.anthropic_remote, o = b.openai_remote
        ));
    }
    args
}

fn build_ssh_command(
    server: &Server,
    known_hosts: Option<&str>,
    bridge: Option<&BridgePorts>,
) -> CommandBuilder {
    let mut cmd = CommandBuilder::new("ssh");
    cmd.env("TERM", "xterm-256color");
    for a in build_ssh_args(server, known_hosts, bridge) {
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

/// 로컬 로그인 셸 실행 구성 (순수 함수 — 테스트 대상).
/// `claude login` / `cswap` 등을 로컬에서 직접 실행할 수 있도록, 원격 브리지와
/// 무관하게 사용자의 기본 셸을 로그인 셸(`-l`)로 띄운다.
pub fn build_local_command() -> CommandBuilder {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let mut cmd = CommandBuilder::new(&shell);
    cmd.arg("-l");
    cmd.env("TERM", "xterm-256color");
    cmd
}

/// PTY 기동 + 리더 스레드 + 세션 등록 — `connect`(ssh)와 `connect_local`이 공유하는
/// 하부 로직. 차이는 오직 실행할 `CommandBuilder` 뿐이다.
fn spawn_session(app: &AppHandle, manager: &TerminalManager, id: &str, cmd: CommandBuilder) -> Result<()> {
    let pty = native_pty_system();
    let pair = pty
        .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| AppError::Invalid(format!("PTY 생성 실패: {e}")))?;

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| AppError::Invalid(format!("셸 실행 실패: {e}")))?;
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| AppError::Invalid(format!("PTY 리더 오류: {e}")))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| AppError::Invalid(format!("PTY 라이터 오류: {e}")))?;

    let id = id.to_string();
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

pub fn connect(
    app: &AppHandle,
    manager: &TerminalManager,
    server: &Server,
    bridge: Option<&BridgePorts>,
) -> Result<()> {
    let known_hosts = crate::hostkey::known_hosts_path(app).map(|p| p.to_string_lossy().to_string());
    let cmd = build_ssh_command(server, known_hosts.as_deref(), bridge);
    spawn_session(app, manager, &server.id, cmd)
}

/// 로컬 셸 세션 기동 — 원격 SSH 없이 `claude login`/`cswap` 등을 로컬에서 바로
/// 실행할 수 있게 한다. `connect`와 동일한 `sessions` 맵에 등록되므로
/// write/resize/disconnect 는 기존 커맨드가 id 기준으로 그대로 동작한다.
pub fn connect_local(app: &AppHandle, manager: &TerminalManager, session_id: &str) -> Result<()> {
    spawn_session(app, manager, session_id, build_local_command())
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
            ai_bridge: false,
            archived_at: None,
            created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-01T00:00:00.000Z".into(),
        }
    }

    #[test]
    fn ssh_args_include_port_user_host_and_key() {
        let args = build_ssh_args(&server(), None, None);
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
        let args = build_ssh_args(&s, None, None);
        assert!(!args.contains(&"-i".to_string()));
    }

    #[test]
    fn ssh_args_strict_checking_and_known_hosts() {
        let args = build_ssh_args(&server(), Some("/tmp/kh"), None);
        assert!(args.contains(&"StrictHostKeyChecking=yes".to_string()));
        assert!(args.contains(&"UserKnownHostsFile=/tmp/kh".to_string()));
        assert!(!args.contains(&"StrictHostKeyChecking=accept-new".to_string()));
    }

    #[test]
    fn ssh_args_escape_known_hosts_path_for_openssh_config_parser() {
        let args = build_ssh_args(
            &server(),
            Some("/Users/polaris/Library/Application Support/com.app/known_hosts"),
            None,
        );
        assert!(args.contains(
            &"UserKnownHostsFile=/Users/polaris/Library/Application\\ Support/com.app/known_hosts"
                .to_string()
        ));
    }

    #[test]
    fn ssh_command_sets_interactive_terminal_type() {
        let cmd = build_ssh_command(&server(), Some("/tmp/kh"), None);
        assert_eq!(
            cmd.get_env("TERM").and_then(|v| v.to_str()),
            Some("xterm-256color")
        );
    }

    fn ports() -> BridgePorts {
        BridgePorts { anthropic_remote: 8971, anthropic_local: 51001, openai_remote: 8972, openai_local: 51002 }
    }

    #[test]
    fn bridge_adds_reverse_forwards_and_env_command() {
        let args = build_ssh_args(&server(), None, Some(&ports()));
        assert!(args.iter().any(|a| a == "-R"));
        assert!(args.contains(&"127.0.0.1:8971:127.0.0.1:51001".to_string()));
        assert!(args.contains(&"127.0.0.1:8972:127.0.0.1:51002".to_string()));
        let cmd = args.last().unwrap();
        assert!(cmd.contains("ANTHROPIC_BASE_URL=http://127.0.0.1:8971"));
        assert!(cmd.contains("OPENAI_BASE_URL=http://127.0.0.1:8972"));
        assert!(cmd.contains("ANTHROPIC_AUTH_TOKEN=bridge"));
        assert!(cmd.contains("exec"));
    }

    #[test]
    fn bridge_command_warns_when_cli_missing() {
        let args = build_ssh_args(&server(), None, Some(&ports()));
        let cmd = args.last().unwrap();
        assert!(cmd.contains("command -v claude"));
        assert!(cmd.contains("command -v codex"));
    }

    #[test]
    fn no_bridge_keeps_args_unchanged() {
        let args = build_ssh_args(&server(), None, None);
        assert!(!args.iter().any(|a| a == "-R"));
        assert_eq!(args.last().unwrap(), "deploy@example.com");
    }

    #[test]
    fn local_command_uses_login_shell() {
        let cmd = build_local_command();
        assert_eq!(cmd.get_env("TERM").and_then(|v| v.to_str()), Some("xterm-256color"));
        // 인자에 "-l" 로그인 셸 플래그 포함
        assert!(cmd.get_argv().iter().any(|a| a == "-l"));
    }
}
