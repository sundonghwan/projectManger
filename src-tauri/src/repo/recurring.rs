use crate::error::{AppError, Result};
use crate::models::RecurringTask;
use crate::repo::task;
use rusqlite::{params, Connection, Row};

fn map_row(row: &Row) -> rusqlite::Result<RecurringTask> {
    Ok(RecurringTask {
        id: row.get("id")?,
        business_id: row.get("business_id")?,
        project_id: row.get("project_id")?,
        title: row.get("title")?,
        priority: row.get("priority")?,
        interval_days: row.get("interval_days")?,
        next_run: row.get("next_run")?,
        active: row.get("active")?,
    })
}

pub fn list_by_business(conn: &Connection, business_id: i64) -> Result<Vec<RecurringTask>> {
    let mut stmt = conn
        .prepare("SELECT * FROM recurring_task WHERE business_id=?1 ORDER BY next_run, id")?;
    let rows = stmt.query_map(params![business_id], map_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn create(
    conn: &Connection,
    business_id: i64,
    project_id: Option<i64>,
    title: &str,
    priority: i64,
    interval_days: i64,
    next_run: &str,
) -> Result<RecurringTask> {
    if title.trim().is_empty() {
        return Err(AppError::Invalid("제목은 필수입니다".into()));
    }
    if interval_days < 1 {
        return Err(AppError::Invalid("반복 간격은 1일 이상이어야 합니다".into()));
    }
    conn.execute(
        "INSERT INTO recurring_task (business_id, project_id, title, priority, interval_days, next_run) \
         VALUES (?1,?2,?3,?4,?5,?6)",
        params![business_id, project_id, title, priority, interval_days, next_run],
    )?;
    conn.query_row(
        "SELECT * FROM recurring_task WHERE id=?1",
        params![conn.last_insert_rowid()],
        map_row,
    )
    .map_err(AppError::Db)
}

pub fn set_active(conn: &Connection, id: i64, active: bool) -> Result<()> {
    let n = conn.execute(
        "UPDATE recurring_task SET active=?2 WHERE id=?1",
        params![id, active as i64],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute("DELETE FROM recurring_task WHERE id=?1", params![id])?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// today(YYYY-MM-DD) 기준으로 도래한 활성 반복 항목에 대해 태스크를 생성하고
/// next_run 을 interval_days 만큼 전진. 생성한 태스크 수 반환.
pub fn generate_due(conn: &Connection, today: &str) -> Result<usize> {
    let due: Vec<RecurringTask> = {
        let mut stmt = conn.prepare(
            "SELECT * FROM recurring_task WHERE active=1 AND next_run <= ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![today], map_row)?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        v
    };

    let mut created = 0usize;
    for r in &due {
        task::create(conn, r.business_id, r.project_id, &r.title, r.priority)?;
        // next_run = date(next_run, '+interval days')
        let new_next: String = conn.query_row(
            "SELECT date(?1, ?2)",
            params![r.next_run, format!("+{} days", r.interval_days)],
            |row| row.get(0),
        )?;
        conn.execute(
            "UPDATE recurring_task SET next_run=?2 WHERE id=?1",
            params![r.id, new_next],
        )?;
        created += 1;
    }
    Ok(created)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, repo::business};

    fn biz(c: &Connection) -> i64 {
        business::create(c, "사업", "si", None).unwrap().id
    }

    #[test]
    fn create_validates() {
        let c = db::open_in_memory().unwrap();
        let b = biz(&c);
        assert!(create(&c, b, None, "", 2, 7, "2026-07-01").is_err());
        assert!(create(&c, b, None, "주간보고", 2, 0, "2026-07-01").is_err());
        assert!(create(&c, b, None, "주간보고", 2, 7, "2026-07-01").is_ok());
    }

    #[test]
    fn generate_due_creates_task_and_advances() {
        let c = db::open_in_memory().unwrap();
        let b = biz(&c);
        create(&c, b, None, "주간보고", 2, 7, "2026-06-20").unwrap();
        // 미도래 항목은 건드리지 않음
        create(&c, b, None, "미래", 2, 7, "2026-12-31").unwrap();

        let made = generate_due(&c, "2026-06-20").unwrap();
        assert_eq!(made, 1);
        let tasks = task::list(&c, b, None).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "주간보고");

        // next_run 이 7일 뒤로 전진
        let items = list_by_business(&c, b).unwrap();
        let weekly = items.iter().find(|i| i.title == "주간보고").unwrap();
        assert_eq!(weekly.next_run, "2026-06-27");
    }

    #[test]
    fn inactive_not_generated() {
        let c = db::open_in_memory().unwrap();
        let b = biz(&c);
        let r = create(&c, b, None, "보고", 2, 1, "2026-06-20").unwrap();
        set_active(&c, r.id, false).unwrap();
        assert_eq!(generate_due(&c, "2026-06-20").unwrap(), 0);
    }

    #[test]
    fn delete_works() {
        let c = db::open_in_memory().unwrap();
        let b = biz(&c);
        let r = create(&c, b, None, "x", 2, 1, "2026-06-20").unwrap();
        delete(&c, r.id).unwrap();
        assert!(list_by_business(&c, b).unwrap().is_empty());
    }
}
