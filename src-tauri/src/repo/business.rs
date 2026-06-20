use crate::error::{AppError, Result};
use crate::models::Business;
use rusqlite::{params, Connection, Row};

fn map_row(row: &Row) -> rusqlite::Result<Business> {
    Ok(Business {
        id: row.get("id")?,
        name: row.get("name")?,
        r#type: row.get("type")?,
        color: row.get("color")?,
        description: row.get("description")?,
        status: row.get("status")?,
        sort_order: row.get("sort_order")?,
        archived_at: row.get("archived_at")?,
    })
}

/// 보관되지 않은 사업을 sort_order 순으로 조회.
pub fn list(conn: &Connection) -> Result<Vec<Business>> {
    let mut stmt = conn
        .prepare("SELECT * FROM business WHERE archived_at IS NULL ORDER BY sort_order, id")?;
    let rows = stmt.query_map([], map_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: i64) -> Result<Business> {
    conn.query_row("SELECT * FROM business WHERE id = ?1", params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Db(other),
        })
}

/// 새 사업 생성. sort_order 는 현재 최대값 + 1 로 맨 뒤에 배치.
pub fn create(conn: &Connection, name: &str, type_: &str, color: Option<&str>) -> Result<Business> {
    if name.trim().is_empty() {
        return Err(AppError::Invalid("사업명은 비어 있을 수 없습니다".into()));
    }
    let next: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM business",
        [],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO business (name, type, color, sort_order) VALUES (?1, ?2, ?3, ?4)",
        params![name, type_, color, next],
    )?;
    get(conn, conn.last_insert_rowid())
}

/// 사업 정보 갱신.
pub fn update(
    conn: &Connection,
    id: i64,
    name: &str,
    type_: &str,
    status: &str,
    color: Option<&str>,
    description: Option<&str>,
) -> Result<Business> {
    let n = conn.execute(
        "UPDATE business SET name=?2, type=?3, status=?4, color=?5, description=?6, \
         updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, name, type_, status, color, description],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

/// 이름만 변경.
pub fn rename(conn: &Connection, id: i64, name: &str) -> Result<Business> {
    if name.trim().is_empty() {
        return Err(AppError::Invalid("사업명은 비어 있을 수 없습니다".into()));
    }
    let n = conn.execute(
        "UPDATE business SET name=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, name],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

/// 소프트 삭제(보관).
pub fn archive(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute(
        "UPDATE business SET archived_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
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
    use crate::db;

    fn conn() -> Connection {
        db::open_in_memory().unwrap()
    }

    #[test]
    fn create_then_list_returns_it() {
        let c = conn();
        let b = create(&c, "SI사업 A", "si", Some("#3b82f6")).unwrap();
        assert_eq!(b.name, "SI사업 A");
        assert_eq!(b.r#type, "si");
        assert_eq!(b.status, "active");
        let all = list(&c).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, b.id);
    }

    #[test]
    fn create_rejects_empty_name() {
        let c = conn();
        assert!(create(&c, "  ", "si", None).is_err());
    }

    #[test]
    fn list_is_ordered_by_sort_order() {
        let c = conn();
        let a = create(&c, "첫째", "si", None).unwrap();
        let b = create(&c, "둘째", "internal", None).unwrap();
        assert!(a.sort_order < b.sort_order);
        let all = list(&c).unwrap();
        assert_eq!(all[0].name, "첫째");
        assert_eq!(all[1].name, "둘째");
    }

    #[test]
    fn update_changes_fields() {
        let c = conn();
        let b = create(&c, "원래", "si", None).unwrap();
        let u = update(&c, b.id, "변경", "ops", "done", Some("#fff"), Some("설명")).unwrap();
        assert_eq!(u.name, "변경");
        assert_eq!(u.r#type, "ops");
        assert_eq!(u.status, "done");
        assert_eq!(u.description.as_deref(), Some("설명"));
    }

    #[test]
    fn archive_hides_from_list() {
        let c = conn();
        let b = create(&c, "보관대상", "si", None).unwrap();
        archive(&c, b.id).unwrap();
        assert!(list(&c).unwrap().is_empty());
        // get 으로는 여전히 조회됨(보관 상태)
        assert!(get(&c, b.id).unwrap().archived_at.is_some());
    }

    #[test]
    fn rename_changes_only_name() {
        let c = conn();
        let b = create(&c, "원래", "si", None).unwrap();
        let r = rename(&c, b.id, "새이름").unwrap();
        assert_eq!(r.name, "새이름");
        assert_eq!(r.r#type, "si");
        assert!(rename(&c, b.id, "  ").is_err());
    }

    #[test]
    fn get_missing_returns_not_found() {
        let c = conn();
        assert!(matches!(get(&c, 999), Err(AppError::NotFound)));
    }

    #[test]
    fn update_missing_returns_not_found() {
        let c = conn();
        assert!(matches!(
            update(&c, 999, "x", "si", "active", None, None),
            Err(AppError::NotFound)
        ));
    }
}
