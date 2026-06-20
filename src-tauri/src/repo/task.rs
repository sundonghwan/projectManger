use crate::error::{AppError, Result};
use crate::models::Task;
use rusqlite::{params, Connection, Row};

const STATUSES: [&str; 4] = ["todo", "doing", "review", "done"];

fn map_row(row: &Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get("id")?,
        business_id: row.get("business_id")?,
        project_id: row.get("project_id")?,
        parent_task_id: row.get("parent_task_id")?,
        title: row.get("title")?,
        description: row.get("description")?,
        status: row.get("status")?,
        priority: row.get("priority")?,
        due_date: row.get("due_date")?,
        sort_order: row.get("sort_order")?,
        completed_at: row.get("completed_at")?,
        archived_at: row.get("archived_at")?,
    })
}

/// 사업(선택적으로 프로젝트)의 보관되지 않은 태스크를 status·sort_order 순으로 조회.
/// project_id=None 이면 사업 전체(직속+프로젝트 소속 모두).
pub fn list(conn: &Connection, business_id: i64, project_id: Option<i64>) -> Result<Vec<Task>> {
    let mut out = Vec::new();
    if let Some(pid) = project_id {
        let mut stmt = conn.prepare(
            "SELECT * FROM task WHERE business_id=?1 AND project_id=?2 AND archived_at IS NULL \
             ORDER BY status, sort_order, id",
        )?;
        let rows = stmt.query_map(params![business_id, pid], map_row)?;
        for r in rows {
            out.push(r?);
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT * FROM task WHERE business_id=?1 AND archived_at IS NULL \
             ORDER BY status, sort_order, id",
        )?;
        let rows = stmt.query_map(params![business_id], map_row)?;
        for r in rows {
            out.push(r?);
        }
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: i64) -> Result<Task> {
    conn.query_row("SELECT * FROM task WHERE id = ?1", params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Db(other),
        })
}

/// 새 태스크 생성. business 필수, project_id 가 있으면 그 사업 소속이어야 함.
pub fn create(
    conn: &Connection,
    business_id: i64,
    project_id: Option<i64>,
    title: &str,
    priority: i64,
) -> Result<Task> {
    if title.trim().is_empty() {
        return Err(AppError::Invalid("태스크 제목은 비어 있을 수 없습니다".into()));
    }
    if !(0..=4).contains(&priority) {
        return Err(AppError::Invalid("우선순위는 0~4".into()));
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
        "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM task WHERE business_id=?1 AND status='todo'",
        params![business_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO task (business_id, project_id, title, priority, sort_order) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![business_id, project_id, title, priority, next],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(
    conn: &Connection,
    id: i64,
    title: &str,
    priority: i64,
    due_date: Option<&str>,
    description: Option<&str>,
) -> Result<Task> {
    let n = conn.execute(
        "UPDATE task SET title=?2, priority=?3, due_date=?4, description=?5, \
         updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, title, priority, due_date, description],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

/// 칸반 드래그: 상태/정렬 변경. status='done' 이면 completed_at 설정, 아니면 해제.
pub fn move_task(conn: &Connection, id: i64, status: &str, sort_order: f64) -> Result<Task> {
    if !STATUSES.contains(&status) {
        return Err(AppError::Invalid(format!("알 수 없는 상태: {status}")));
    }
    let n = conn.execute(
        "UPDATE task SET status=?2, sort_order=?3, \
         completed_at = CASE WHEN ?2='done' THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END, \
         updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, status, sort_order],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

pub fn archive(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute(
        "UPDATE task SET archived_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
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
    fn create_defaults_to_todo() {
        let (c, biz, _) = setup();
        let t = create(&c, biz, None, "로그인 API", 3).unwrap();
        assert_eq!(t.status, "todo");
        assert_eq!(t.priority, 3);
        assert!(t.completed_at.is_none());
    }

    #[test]
    fn create_rejects_empty_title_and_bad_priority() {
        let (c, biz, _) = setup();
        assert!(create(&c, biz, None, "  ", 2).is_err());
        assert!(create(&c, biz, None, "x", 9).is_err());
    }

    #[test]
    fn create_rejects_project_of_other_business() {
        let (c, biz, _) = setup();
        let other = business::create(&c, "다른사업", "ops", None).unwrap();
        let other_proj = project::create(&c, other.id, "P").unwrap();
        // biz 의 태스크인데 다른 사업의 프로젝트를 지정 → 거부
        assert!(create(&c, biz, Some(other_proj.id), "x", 2).is_err());
    }

    #[test]
    fn list_filters_by_project() {
        let (c, biz, proj) = setup();
        create(&c, biz, Some(proj), "프로젝트 소속", 2).unwrap();
        create(&c, biz, None, "사업 직속", 2).unwrap();
        assert_eq!(list(&c, biz, None).unwrap().len(), 2);
        assert_eq!(list(&c, biz, Some(proj)).unwrap().len(), 1);
    }

    #[test]
    fn move_to_done_sets_completed_at() {
        let (c, biz, _) = setup();
        let t = create(&c, biz, None, "x", 2).unwrap();
        let moved = move_task(&c, t.id, "done", 5.0).unwrap();
        assert_eq!(moved.status, "done");
        assert!(moved.completed_at.is_some());
        assert_eq!(moved.sort_order, 5.0);
        // 다시 doing 으로 옮기면 completed_at 해제
        let back = move_task(&c, t.id, "doing", 2.0).unwrap();
        assert!(back.completed_at.is_none());
    }

    #[test]
    fn move_rejects_unknown_status() {
        let (c, biz, _) = setup();
        let t = create(&c, biz, None, "x", 2).unwrap();
        assert!(matches!(move_task(&c, t.id, "bogus", 1.0), Err(AppError::Invalid(_))));
    }

    #[test]
    fn archive_hides_from_list() {
        let (c, biz, _) = setup();
        let t = create(&c, biz, None, "x", 2).unwrap();
        archive(&c, t.id).unwrap();
        assert!(list(&c, biz, None).unwrap().is_empty());
    }

    #[test]
    fn update_changes_fields() {
        let (c, biz, _) = setup();
        let t = create(&c, biz, None, "원래", 2).unwrap();
        let u = update(&c, t.id, "변경", 4, Some("2026-07-01"), Some("메모")).unwrap();
        assert_eq!(u.title, "변경");
        assert_eq!(u.priority, 4);
        assert_eq!(u.due_date.as_deref(), Some("2026-07-01"));
    }
}
