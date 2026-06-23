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
        folder_id: row.get("folder_id")?,
        title: row.get("title")?,
        kind: row.get("kind")?,
        document_id: row.get("document_id")?,
        file_path: row.get("file_path")?,
        file_size: row.get("file_size")?,
        original_name: row.get("original_name")?,
        status: row.get("status")?,
        current_version: row.get("current_version")?,
        sort_order: row.get("sort_order")?,
        archived_at: row.get("archived_at")?,
        created_at: row.get("created_at")?,
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
    folder_id: Option<i64>,
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
    crate::repo::folder::ensure_owns(conn, folder_id, business_id, "deliverable")?;
    let next: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order),0)+1 FROM deliverable WHERE business_id=?1",
        params![business_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO deliverable (business_id, project_id, folder_id, title, kind, sort_order) \
         VALUES (?1,?2,?3,?4,?5,?6)",
        params![business_id, project_id, folder_id, title, kind, next],
    )?;
    let id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO deliverable_version (deliverable_id, version, note) VALUES (?1, 1, '최초 생성')",
        params![id],
    )?;
    get(conn, id)
}

/// 파일 산출물 생성(업로드 모델). 버전 행은 남기지 않는다. file_path 는 복사 후 set_file_path 로 채운다.
pub fn create_file(
    conn: &Connection,
    business_id: i64,
    project_id: Option<i64>,
    folder_id: Option<i64>,
    title: &str,
    original_name: &str,
    file_size: i64,
) -> Result<Deliverable> {
    if title.trim().is_empty() {
        return Err(AppError::Invalid("산출물명은 비어 있을 수 없습니다".into()));
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
    crate::repo::folder::ensure_owns(conn, folder_id, business_id, "deliverable")?;
    let next: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order),0)+1 FROM deliverable WHERE business_id=?1",
        params![business_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO deliverable (business_id, project_id, folder_id, title, kind, original_name, file_size, sort_order) \
         VALUES (?1,?2,?3,?4,'file',?5,?6,?7)",
        params![business_id, project_id, folder_id, title, original_name, file_size, next],
    )?;
    get(conn, conn.last_insert_rowid())
}

/// 산출물을 폴더로 이동(또는 folder_id=None 으로 미분류). 같은 사업의 산출물 폴더여야 함.
pub fn set_folder(conn: &Connection, id: i64, folder_id: Option<i64>) -> Result<Deliverable> {
    let d = get(conn, id)?;
    crate::repo::folder::ensure_owns(conn, folder_id, d.business_id, "deliverable")?;
    conn.execute(
        "UPDATE deliverable SET folder_id=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, folder_id],
    )?;
    get(conn, id)
}

/// 복사 완료된 파일의 절대 경로를 기록.
pub fn set_file_path(conn: &Connection, id: i64, file_path: &str) -> Result<()> {
    let n = conn.execute(
        "UPDATE deliverable SET file_path=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, file_path],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// 표시명(title) 수정. 원본 파일명(original_name)은 바뀌지 않는다.
pub fn rename(conn: &Connection, id: i64, title: &str) -> Result<Deliverable> {
    if title.trim().is_empty() {
        return Err(AppError::Invalid("산출물명은 비어 있을 수 없습니다".into()));
    }
    let n = conn.execute(
        "UPDATE deliverable SET title=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, title.trim()],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

/// 산출물 삭제(영구) 직전, 물리 파일 정리를 위해 file_path 를 조회.
pub fn file_path_of(conn: &Connection, id: i64) -> Result<Option<String>> {
    conn.query_row("SELECT file_path FROM deliverable WHERE id=?1", params![id], |r| r.get(0))
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Db(other),
        })
}

/// 산출물 행 삭제(영구). 물리 파일 제거는 호출자(명령 계층)가 담당.
pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute("DELETE FROM deliverable WHERE id=?1", params![id])?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
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
        let d = create(&c, biz, None, None, "제안서", "document").unwrap();
        assert_eq!(d.status, "draft");
        assert_eq!(d.current_version, 1);
        let versions = list_versions(&c, d.id).unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version, 1);
    }

    #[test]
    fn create_validates_kind_and_title_and_project() {
        let (c, biz, _) = setup();
        assert!(create(&c, biz, None, None, " ", "file").is_err());
        assert!(create(&c, biz, None, None, "x", "bogus").is_err());
        let other = business::create(&c, "다른", "ops", None).unwrap();
        let op = project::create(&c, other.id, "P").unwrap();
        assert!(create(&c, biz, Some(op.id), None, "x", "file").is_err());
    }

    #[test]
    fn add_version_increments_and_records() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, None, "산출물", "file").unwrap();
        let d2 = add_version(&c, d.id, Some("피드백 반영"), None).unwrap();
        assert_eq!(d2.current_version, 2);
        let versions = list_versions(&c, d.id).unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, 2); // 최신 먼저
        assert_eq!(versions[0].note.as_deref(), Some("피드백 반영"));
    }

    #[test]
    fn create_file_sets_fields_without_version() {
        let (c, biz, _) = setup();
        let d = create_file(&c, biz, None, None, "보고서.pdf", "보고서.pdf", 2048).unwrap();
        assert_eq!(d.kind, "file");
        assert_eq!(d.status, "draft");
        assert_eq!(d.original_name.as_deref(), Some("보고서.pdf"));
        assert_eq!(d.file_size, Some(2048));
        assert!(d.file_path.is_none());
        // 업로드 모델은 버전 행을 만들지 않는다
        assert!(list_versions(&c, d.id).unwrap().is_empty());
        // 경로 기록
        set_file_path(&c, d.id, "/tmp/x/보고서.pdf").unwrap();
        assert_eq!(get(&c, d.id).unwrap().file_path.as_deref(), Some("/tmp/x/보고서.pdf"));
    }

    #[test]
    fn rename_changes_title_only() {
        let (c, biz, _) = setup();
        let d = create_file(&c, biz, None, None, "a.txt", "a.txt", 1).unwrap();
        let r = rename(&c, d.id, "최종본").unwrap();
        assert_eq!(r.title, "최종본");
        assert_eq!(r.original_name.as_deref(), Some("a.txt")); // 원본명 불변
        assert!(rename(&c, d.id, "  ").is_err());
    }

    #[test]
    fn delete_removes_row() {
        let (c, biz, _) = setup();
        let d = create_file(&c, biz, None, None, "a.txt", "a.txt", 1).unwrap();
        delete(&c, d.id).unwrap();
        assert!(get(&c, d.id).is_err());
    }

    #[test]
    fn update_status_validates() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, None, "x", "file").unwrap();
        assert_eq!(update_status(&c, d.id, "review").unwrap().status, "review");
        assert!(update_status(&c, d.id, "bogus").is_err());
    }

    #[test]
    fn archive_hides_from_list() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, None, "x", "file").unwrap();
        archive(&c, d.id).unwrap();
        assert!(list_by_business(&c, biz).unwrap().is_empty());
    }

    #[test]
    fn deleting_deliverable_cascades_versions() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, None, "x", "file").unwrap();
        add_version(&c, d.id, None, None).unwrap();
        c.execute("DELETE FROM deliverable WHERE id=?1", params![d.id]).unwrap();
        let count: i64 = c
            .query_row("SELECT count(*) FROM deliverable_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
