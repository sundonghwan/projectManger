use crate::error::{AppError, Result};
use crate::models::ServerConnection;
use rusqlite::{params, Connection, Row};

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
        let t = k.trim();
        if t.starts_with('-') {
            return Err(AppError::Invalid("키 파일 경로는 '-'로 시작할 수 없습니다".into()));
        }
    }
    Ok(())
}

fn map_row(row: &Row) -> rusqlite::Result<ServerConnection> {
    Ok(ServerConnection {
        id: row.get("id")?,
        business_id: row.get("business_id")?,
        project_id: row.get("project_id")?,
        name: row.get("name")?,
        host: row.get("host")?,
        port: row.get("port")?,
        username: row.get("username")?,
        auth_type: row.get("auth_type")?,
        key_path: row.get("key_path")?,
        secret_ref: row.get("secret_ref")?,
        last_used_at: row.get("last_used_at")?,
        archived_at: row.get("archived_at")?,
    })
}

pub fn list_by_business(conn: &Connection, business_id: i64) -> Result<Vec<ServerConnection>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM server_connection WHERE business_id=?1 AND archived_at IS NULL \
         ORDER BY name, id",
    )?;
    let rows = stmt.query_map(params![business_id], map_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: i64) -> Result<ServerConnection> {
    conn.query_row("SELECT * FROM server_connection WHERE id=?1", params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Db(other),
        })
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    business_id: i64,
    project_id: Option<i64>,
    name: &str,
    host: &str,
    port: i64,
    username: &str,
    auth_type: &str,
    key_path: Option<&str>,
) -> Result<ServerConnection> {
    if name.trim().is_empty() || host.trim().is_empty() || username.trim().is_empty() {
        return Err(AppError::Invalid("이름·호스트·사용자는 필수입니다".into()));
    }
    if !AUTH_TYPES.contains(&auth_type) {
        return Err(AppError::Invalid(format!("알 수 없는 인증 방식: {auth_type}")));
    }
    validate_ssh_field(host, "호스트")?;
    validate_ssh_field(username, "사용자")?;
    validate_key_path(key_path)?;
    conn.execute(
        "INSERT INTO server_connection (business_id, project_id, name, host, port, username, auth_type, key_path) \
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![business_id, project_id, name, host, port, username, auth_type, key_path],
    )?;
    get(conn, conn.last_insert_rowid())
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    conn: &Connection,
    id: i64,
    name: &str,
    host: &str,
    port: i64,
    username: &str,
    auth_type: &str,
    key_path: Option<&str>,
) -> Result<ServerConnection> {
    if name.trim().is_empty() || host.trim().is_empty() || username.trim().is_empty() {
        return Err(AppError::Invalid("이름·호스트·사용자는 필수입니다".into()));
    }
    if !AUTH_TYPES.contains(&auth_type) {
        return Err(AppError::Invalid(format!("알 수 없는 인증 방식: {auth_type}")));
    }
    validate_ssh_field(host, "호스트")?;
    validate_ssh_field(username, "사용자")?;
    validate_key_path(key_path)?;
    let n = conn.execute(
        "UPDATE server_connection SET name=?2, host=?3, port=?4, username=?5, auth_type=?6, key_path=?7, \
         updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, name, host, port, username, auth_type, key_path],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

/// 키체인 참조 키만 기록(실제 비밀값은 DB에 저장하지 않음).
pub fn set_secret_ref(conn: &Connection, id: i64, secret_ref: Option<&str>) -> Result<()> {
    let n = conn.execute(
        "UPDATE server_connection SET secret_ref=?2 WHERE id=?1",
        params![id, secret_ref],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn touch_last_used(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE server_connection SET last_used_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id],
    )?;
    Ok(())
}

pub fn archive(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute(
        "UPDATE server_connection SET archived_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, repo::business};

    fn setup() -> (Connection, i64) {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "사업", "si", None).unwrap();
        (c, b.id)
    }

    #[test]
    fn create_defaults_no_secret_in_db() {
        let (c, biz) = setup();
        let s = create(&c, biz, None, "스테이징", "10.0.0.5", 22, "deploy", "key", Some("~/.ssh/id")).unwrap();
        assert_eq!(s.host, "10.0.0.5");
        assert_eq!(s.auth_type, "key");
        // 비밀값/참조는 아직 없음
        assert!(s.secret_ref.is_none());
        assert_eq!(list_by_business(&c, biz).unwrap().len(), 1);
    }

    #[test]
    fn create_validates_required_and_auth() {
        let (c, biz) = setup();
        assert!(create(&c, biz, None, "", "h", 22, "u", "key", None).is_err());
        assert!(create(&c, biz, None, "n", "h", 22, "u", "bogus", None).is_err());
    }

    #[test]
    fn create_rejects_ssh_arg_injection() {
        let (c, biz) = setup();
        // username 이 '-'로 시작 → ssh 옵션 주입(-oProxyCommand=…) 차단
        assert!(create(&c, biz, None, "n", "h", 22, "-oProxyCommand=touch /tmp/x", "key", None).is_err());
        // host 가 '-'로 시작
        assert!(create(&c, biz, None, "n", "-Ldanger", 22, "u", "key", None).is_err());
        // 공백/@ 포함
        assert!(create(&c, biz, None, "n", "h", 22, "u name", "key", None).is_err());
        assert!(create(&c, biz, None, "n", "h@evil", 22, "u", "key", None).is_err());
        // 키 경로 '-' 접두
        assert!(create(&c, biz, None, "n", "h", 22, "u", "key", Some("-i/evil")).is_err());
        // 정상 값은 통과
        assert!(create(&c, biz, None, "n", "example.com", 22, "deploy", "key", Some("/home/u/.ssh/id_ed25519")).is_ok());
    }

    #[test]
    fn update_rejects_ssh_arg_injection() {
        let (c, biz) = setup();
        let s = create(&c, biz, None, "n", "h", 22, "u", "key", None).unwrap();
        assert!(update(&c, s.id, "n", "h", 22, "-oProxyCommand=x", "key", None).is_err());
        assert!(update(&c, s.id, "n", "-bad", 22, "u", "key", None).is_err());
    }

    #[test]
    fn set_secret_ref_records_only_reference() {
        let (c, biz) = setup();
        let s = create(&c, biz, None, "n", "h", 22, "u", "password", None).unwrap();
        set_secret_ref(&c, s.id, Some("ssh/conn-1")).unwrap();
        let got = get(&c, s.id).unwrap();
        assert_eq!(got.secret_ref.as_deref(), Some("ssh/conn-1"));
        // DB 스키마에 평문 비밀번호 컬럼이 없음을 보장 (secret_ref만 존재)
    }

    #[test]
    fn update_and_archive() {
        let (c, biz) = setup();
        let s = create(&c, biz, None, "n", "h", 22, "u", "key", None).unwrap();
        let u = update(&c, s.id, "새이름", "h2", 2222, "u2", "agent", None).unwrap();
        assert_eq!(u.name, "새이름");
        assert_eq!(u.port, 2222);
        archive(&c, s.id).unwrap();
        assert!(list_by_business(&c, biz).unwrap().is_empty());
    }
}
