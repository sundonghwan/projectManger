use crate::error::{AppError, Result};
use crate::models::{Deliverable, DeliverableVersion};
use rusqlite::{params, Connection, Row};

const STATUSES: [&str; 3] = ["draft", "review", "done"];
const KINDS: [&str; 2] = ["file", "document"];

fn map_deliverable(row: &Row) -> rusqlite::Result<Deliverable> {
    Ok(Deliverable {
        id: row.get("id")?,
        business_id: row.get("business_id")?,
        project_id: row.get("project_id")?,
        title: row.get("title")?,
        kind: row.get("kind")?,
        document_id: row.get("document_id")?,
        file_path: row.get("file_path")?,
        status: row.get("status")?,
        current_version: row.get("current_version")?,
        sort_order: row.get("sort_order")?,
        archived_at: row.get("archived_at")?,
    })
}

fn map_version(row: &Row) -> rusqlite::Result<DeliverableVersion> {
    Ok(DeliverableVersion {
        id: row.get("id")?,
        deliverable_id: row.get("deliverable_id")?,
        version: row.get("version")?,
        file_path: row.get("file_path")?,
        note: row.get("note")?,
        created_at: row.get("created_at")?,
    })
}

pub fn list_by_business(conn: &Connection, business_id: i64) -> Result<Vec<Deliverable>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM deliverable WHERE business_id=?1 AND archived_at IS NULL \
         ORDER BY sort_order, id",
    )?;
    let rows = stmt.query_map(params![business_id], map_deliverable)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: i64) -> Result<Deliverable> {
    conn.query_row("SELECT * FROM deliverable WHERE id=?1", params![id], map_deliverable)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Db(other),
        })
}

/// 산출물 생성. 초기 버전 v1 기록도 함께 남긴다.
pub fn create(
    conn: &Connection,
    business_id: i64,
    project_id: Option<i64>,
    title: &str,
    kind: &str,
) -> Result<Deliverable> {
    if title.trim().is_empty() {
        return Err(AppError::Invalid("산출물명은 비어 있을 수 없습니다".into()));
    }
    if !KINDS.contains(&kind) {
        return Err(AppError::Invalid(format!("알 수 없는 종류: {kind}")));
    }
    if let Some(pid) = project_id {
        let ok: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM project WHERE id=?1 AND business_id=?2)",
            params![pid, business_id],
            |r| r.get(0),
        )?;
        if !ok {
            return Err(AppError::Invalid("프로젝트가 해당 사업 소속이 아닙니다".into()));
        }
    }
    let next: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order),0)+1 FROM deliverable WHERE business_id=?1",
        params![business_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO deliverable (business_id, project_id, title, kind, sort_order) \
         VALUES (?1,?2,?3,?4,?5)",
        params![business_id, project_id, title, kind, next],
    )?;
    let id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO deliverable_version (deliverable_id, version, note) VALUES (?1, 1, '최초 생성')",
        params![id],
    )?;
    get(conn, id)
}

pub fn update_status(conn: &Connection, id: i64, status: &str) -> Result<Deliverable> {
    if !STATUSES.contains(&status) {
        return Err(AppError::Invalid(format!("알 수 없는 상태: {status}")));
    }
    let n = conn.execute(
        "UPDATE deliverable SET status=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, status],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

/// 새 버전 기록 (current_version + 1).
pub fn add_version(
    conn: &Connection,
    id: i64,
    note: Option<&str>,
    file_path: Option<&str>,
) -> Result<Deliverable> {
    let current: i64 = conn
        .query_row("SELECT current_version FROM deliverable WHERE id=?1", params![id], |r| r.get(0))
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Db(other),
        })?;
    let next = current + 1;
    conn.execute(
        "INSERT INTO deliverable_version (deliverable_id, version, note, file_path) \
         VALUES (?1, ?2, ?3, ?4)",
        params![id, next, note, file_path],
    )?;
    conn.execute(
        "UPDATE deliverable SET current_version=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') \
         WHERE id=?1",
        params![id, next],
    )?;
    get(conn, id)
}

pub fn list_versions(conn: &Connection, deliverable_id: i64) -> Result<Vec<DeliverableVersion>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM deliverable_version WHERE deliverable_id=?1 ORDER BY version DESC",
    )?;
    let rows = stmt.query_map(params![deliverable_id], map_version)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn archive(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute(
        "UPDATE deliverable SET archived_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
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
    use crate::{db, repo::business, repo::project};

    fn setup() -> (Connection, i64, i64) {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "사업", "si", None).unwrap();
        let p = project::create(&c, b.id, "프로젝트").unwrap();
        (c, b.id, p.id)
    }

    #[test]
    fn create_defaults_and_initial_version() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, "제안서", "document").unwrap();
        assert_eq!(d.status, "draft");
        assert_eq!(d.current_version, 1);
        let versions = list_versions(&c, d.id).unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version, 1);
    }

    #[test]
    fn create_validates_kind_and_title_and_project() {
        let (c, biz, _) = setup();
        assert!(create(&c, biz, None, " ", "file").is_err());
        assert!(create(&c, biz, None, "x", "bogus").is_err());
        let other = business::create(&c, "다른", "ops", None).unwrap();
        let op = project::create(&c, other.id, "P").unwrap();
        assert!(create(&c, biz, Some(op.id), "x", "file").is_err());
    }

    #[test]
    fn add_version_increments_and_records() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, "산출물", "file").unwrap();
        let d2 = add_version(&c, d.id, Some("피드백 반영"), None).unwrap();
        assert_eq!(d2.current_version, 2);
        let versions = list_versions(&c, d.id).unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, 2); // 최신 먼저
        assert_eq!(versions[0].note.as_deref(), Some("피드백 반영"));
    }

    #[test]
    fn update_status_validates() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, "x", "file").unwrap();
        assert_eq!(update_status(&c, d.id, "review").unwrap().status, "review");
        assert!(update_status(&c, d.id, "bogus").is_err());
    }

    #[test]
    fn archive_hides_from_list() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, "x", "file").unwrap();
        archive(&c, d.id).unwrap();
        assert!(list_by_business(&c, biz).unwrap().is_empty());
    }

    #[test]
    fn deleting_deliverable_cascades_versions() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, "x", "file").unwrap();
        add_version(&c, d.id, None, None).unwrap();
        c.execute("DELETE FROM deliverable WHERE id=?1", params![d.id]).unwrap();
        let count: i64 = c
            .query_row("SELECT count(*) FROM deliverable_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
