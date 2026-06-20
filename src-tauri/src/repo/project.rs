use crate::error::{AppError, Result};
use crate::models::Project;
use rusqlite::{params, Connection, Row};

fn map_row(row: &Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get("id")?,
        business_id: row.get("business_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        status: row.get("status")?,
        start_date: row.get("start_date")?,
        due_date: row.get("due_date")?,
        sort_order: row.get("sort_order")?,
        archived_at: row.get("archived_at")?,
    })
}

/// 특정 사업의 보관되지 않은 프로젝트를 sort_order 순으로 조회.
pub fn list_by_business(conn: &Connection, business_id: i64) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM project WHERE business_id = ?1 AND archived_at IS NULL \
         ORDER BY sort_order, id",
    )?;
    let rows = stmt.query_map(params![business_id], map_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: i64) -> Result<Project> {
    conn.query_row("SELECT * FROM project WHERE id = ?1", params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Db(other),
        })
}

/// 새 프로젝트 생성. 소속 사업이 존재하지 않으면 무결성 오류.
pub fn create(conn: &Connection, business_id: i64, name: &str) -> Result<Project> {
    if name.trim().is_empty() {
        return Err(AppError::Invalid("프로젝트명은 비어 있을 수 없습니다".into()));
    }
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM business WHERE id = ?1 AND archived_at IS NULL)",
        params![business_id],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(AppError::Invalid("존재하지 않는 사업입니다".into()));
    }
    let next: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM project WHERE business_id = ?1",
        params![business_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO project (business_id, name, sort_order) VALUES (?1, ?2, ?3)",
        params![business_id, name, next],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(
    conn: &Connection,
    id: i64,
    name: &str,
    status: &str,
    description: Option<&str>,
    due_date: Option<&str>,
) -> Result<Project> {
    let n = conn.execute(
        "UPDATE project SET name=?2, status=?3, description=?4, due_date=?5, \
         updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, name, status, description, due_date],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

pub fn archive(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute(
        "UPDATE project SET archived_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
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
    fn create_then_list_by_business() {
        let (c, biz) = setup();
        let p = create(&c, biz, "프로젝트 1").unwrap();
        assert_eq!(p.business_id, biz);
        assert_eq!(p.status, "active");
        let list = list_by_business(&c, biz).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, p.id);
    }

    #[test]
    fn create_rejects_unknown_business() {
        let (c, _) = setup();
        assert!(matches!(create(&c, 999, "x"), Err(AppError::Invalid(_))));
    }

    #[test]
    fn create_rejects_empty_name() {
        let (c, biz) = setup();
        assert!(create(&c, biz, "   ").is_err());
    }

    #[test]
    fn list_only_returns_own_business() {
        let (c, biz1) = setup();
        let biz2 = business::create(&c, "사업2", "ops", None).unwrap();
        create(&c, biz1, "P1").unwrap();
        create(&c, biz2.id, "P2").unwrap();
        assert_eq!(list_by_business(&c, biz1).unwrap().len(), 1);
        assert_eq!(list_by_business(&c, biz2.id).unwrap().len(), 1);
    }

    #[test]
    fn archive_hides_from_list() {
        let (c, biz) = setup();
        let p = create(&c, biz, "P").unwrap();
        archive(&c, p.id).unwrap();
        assert!(list_by_business(&c, biz).unwrap().is_empty());
    }

    #[test]
    fn update_changes_fields() {
        let (c, biz) = setup();
        let p = create(&c, biz, "원래").unwrap();
        let u = update(&c, p.id, "변경", "done", Some("설명"), Some("2026-07-30")).unwrap();
        assert_eq!(u.name, "변경");
        assert_eq!(u.status, "done");
        assert_eq!(u.due_date.as_deref(), Some("2026-07-30"));
    }

    #[test]
    fn archiving_business_cascades_to_projects_via_fk_on_delete() {
        // 참고: 보관은 소프트삭제라 CASCADE 아님. 여기선 실제 삭제 시 CASCADE 확인.
        let (c, biz) = setup();
        create(&c, biz, "P").unwrap();
        c.execute("DELETE FROM business WHERE id = ?1", params![biz]).unwrap();
        let count: i64 = c
            .query_row("SELECT count(*) FROM project", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
