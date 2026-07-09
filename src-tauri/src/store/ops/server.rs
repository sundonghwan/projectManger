use crate::error::{AppError, Result};
use crate::store::ids::{new_id, now};
use crate::store::local::LocalStore;
use crate::store::model::Server;

const AUTH_TYPES: [&str; 3] = ["key", "password", "agent"];

/// SSH 인자 주입(CWE-88) 방지: host/username 이 '-'로 시작하면 ssh/sftp 가 옵션으로
/// 해석(-oProxyCommand=… 류)하여 임의 명령 실행으로 이어질 수 있다. '@'·공백·제어문자도 거부.
fn validate_ssh_field(value: &str, field: &str) -> Result<()> {
    let t = value.trim();
    if t.starts_with('-') {
        return Err(AppError::Invalid(format!("{field}는 '-'로 시작할 수 없습니다")));
    }
    if t.chars().any(|c| c.is_control() || c == '@' || c == ' ' || c == '\t') {
        return Err(AppError::Invalid(format!(
            "{field}에 허용되지 않는 문자가 있습니다(@·공백·제어문자)"
        )));
    }
    Ok(())
}

/// 키 파일 경로는 공백/@ 를 포함할 수 있으나, '-' 로 시작하면 ssh 옵션으로 오인되므로 거부.
fn validate_key_path(key_path: Option<&str>) -> Result<()> {
    if let Some(k) = key_path {
        if k.trim().starts_with('-') {
            return Err(AppError::Invalid("키 파일 경로는 '-'로 시작할 수 없습니다".into()));
        }
    }
    Ok(())
}

pub fn list_by_business(local: &LocalStore, business_id: &str) -> Result<Vec<Server>> {
    let mut out: Vec<Server> = local
        .servers
        .list()
        .into_iter()
        .filter(|s| s.business_id == business_id && s.archived_at.is_none())
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
    Ok(out)
}

pub fn get(local: &LocalStore, id: &str) -> Result<Server> {
    local.servers.get(id).cloned().ok_or(AppError::NotFound)
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    local: &mut LocalStore,
    business_id: &str,
    project_id: Option<&str>,
    name: &str,
    host: &str,
    port: i64,
    username: &str,
    auth_type: &str,
    key_path: Option<&str>,
    ai_bridge: bool,
) -> Result<Server> {
    if name.trim().is_empty() || host.trim().is_empty() || username.trim().is_empty() {
        return Err(AppError::Invalid("이름·호스트·사용자는 필수입니다".into()));
    }
    if !AUTH_TYPES.contains(&auth_type) {
        return Err(AppError::Invalid(format!("알 수 없는 인증 방식: {auth_type}")));
    }
    validate_ssh_field(host, "호스트")?;
    validate_ssh_field(username, "사용자")?;
    validate_key_path(key_path)?;
    let ts = now();
    let s = Server {
        id: new_id(),
        business_id: business_id.to_string(),
        project_id: project_id.map(|s| s.to_string()),
        name: name.to_string(),
        host: host.to_string(),
        port,
        username: username.to_string(),
        auth_type: auth_type.to_string(),
        key_path: key_path.map(|s| s.to_string()),
        secret_ref: None,
        last_used_at: None,
        ai_bridge,
        archived_at: None,
        created_at: ts.clone(),
        updated_at: ts,
    };
    local.servers.put(s.clone())?;
    Ok(s)
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    local: &mut LocalStore,
    id: &str,
    name: &str,
    host: &str,
    port: i64,
    username: &str,
    auth_type: &str,
    key_path: Option<&str>,
    ai_bridge: bool,
) -> Result<Server> {
    if name.trim().is_empty() || host.trim().is_empty() || username.trim().is_empty() {
        return Err(AppError::Invalid("이름·호스트·사용자는 필수입니다".into()));
    }
    if !AUTH_TYPES.contains(&auth_type) {
        return Err(AppError::Invalid(format!("알 수 없는 인증 방식: {auth_type}")));
    }
    validate_ssh_field(host, "호스트")?;
    validate_ssh_field(username, "사용자")?;
    validate_key_path(key_path)?;
    let mut s = get(local, id)?;
    s.name = name.to_string();
    s.host = host.to_string();
    s.port = port;
    s.username = username.to_string();
    s.auth_type = auth_type.to_string();
    s.key_path = key_path.map(|x| x.to_string());
    s.ai_bridge = ai_bridge;
    s.updated_at = now();
    local.servers.put(s.clone())?;
    Ok(s)
}

/// 키체인 참조 키만 기록(실제 비밀값은 저장하지 않음).
pub fn set_secret_ref(local: &mut LocalStore, id: &str, secret_ref: Option<&str>) -> Result<()> {
    let mut s = get(local, id)?;
    s.secret_ref = secret_ref.map(|x| x.to_string());
    s.updated_at = now();
    local.servers.put(s)?;
    Ok(())
}

pub fn touch_last_used(local: &mut LocalStore, id: &str) -> Result<()> {
    let mut s = get(local, id)?;
    let ts = now();
    s.last_used_at = Some(ts.clone());
    s.updated_at = ts;
    local.servers.put(s)?;
    Ok(())
}

pub fn archive(local: &mut LocalStore, id: &str) -> Result<()> {
    let mut s = get(local, id)?;
    let ts = now();
    s.archived_at = Some(ts.clone());
    s.updated_at = ts;
    local.servers.put(s)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::local::LocalStore;

    fn ls() -> LocalStore {
        LocalStore::open(std::env::temp_dir().join(format!("ops_srv_{}", new_id()))).unwrap()
    }

    #[test]
    fn create_defaults_no_secret() {
        let mut s = ls();
        let srv = create(&mut s, "b1", None, "스테이징", "10.0.0.5", 22, "deploy", "key", Some("~/.ssh/id"), false).unwrap();
        assert_eq!(srv.host, "10.0.0.5");
        assert_eq!(srv.auth_type, "key");
        assert!(srv.secret_ref.is_none());
        assert!(!srv.ai_bridge);
        assert_eq!(list_by_business(&s, "b1").unwrap().len(), 1);
    }

    #[test]
    fn create_validates_required_and_auth() {
        let mut s = ls();
        assert!(create(&mut s, "b1", None, "", "h", 22, "u", "key", None, false).is_err());
        assert!(create(&mut s, "b1", None, "n", "h", 22, "u", "bogus", None, false).is_err());
    }

    #[test]
    fn create_rejects_ssh_arg_injection() {
        let mut s = ls();
        assert!(create(&mut s, "b1", None, "n", "h", 22, "-oProxyCommand=touch /tmp/x", "key", None, false).is_err());
        assert!(create(&mut s, "b1", None, "n", "-Ldanger", 22, "u", "key", None, false).is_err());
        assert!(create(&mut s, "b1", None, "n", "h", 22, "u name", "key", None, false).is_err());
        assert!(create(&mut s, "b1", None, "n", "h@evil", 22, "u", "key", None, false).is_err());
        assert!(create(&mut s, "b1", None, "n", "h", 22, "u", "key", Some("-i/evil"), false).is_err());
        assert!(create(&mut s, "b1", None, "n", "example.com", 22, "deploy", "key", Some("/home/u/.ssh/id_ed25519"), false).is_ok());
    }

    #[test]
    fn set_secret_ref_and_update_archive() {
        let mut s = ls();
        let srv = create(&mut s, "b1", None, "n", "h", 22, "u", "password", None, false).unwrap();
        set_secret_ref(&mut s, &srv.id, Some("ssh/conn-x")).unwrap();
        assert_eq!(get(&s, &srv.id).unwrap().secret_ref.as_deref(), Some("ssh/conn-x"));
        let u = update(&mut s, &srv.id, "새이름", "h2", 2222, "u2", "agent", None, true).unwrap();
        assert_eq!(u.name, "새이름");
        assert_eq!(u.port, 2222);
        assert!(u.ai_bridge);
        touch_last_used(&mut s, &srv.id).unwrap();
        assert!(get(&s, &srv.id).unwrap().last_used_at.is_some());
        archive(&mut s, &srv.id).unwrap();
        assert!(list_by_business(&s, "b1").unwrap().is_empty());
    }

    #[test]
    fn update_rejects_injection_and_missing() {
        let mut s = ls();
        let srv = create(&mut s, "b1", None, "n", "h", 22, "u", "key", None, false).unwrap();
        assert!(update(&mut s, &srv.id, "n", "h", 22, "-oProxyCommand=x", "key", None, false).is_err());
        assert!(matches!(get(&s, "nope"), Err(AppError::NotFound)));
    }
}
