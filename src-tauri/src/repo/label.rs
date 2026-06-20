use crate::error::{AppError, Result};
use crate::models::{Label, TaskLabel};
use rusqlite::{params, Connection, Row};

fn map_label(row: &Row) -> rusqlite::Result<Label> {
    Ok(Label {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
    })
}

pub fn list(conn: &Connection) -> Result<Vec<Label>> {
    let mut stmt = conn.prepare("SELECT * FROM label ORDER BY name")?;
    let rows = stmt.query_map([], map_label)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// 라벨 생성. 같은 이름이 있으면 기존 것을 반환(멱등).
pub fn create(conn: &Connection, name: &str, color: Option<&str>) -> Result<Label> {
    if name.trim().is_empty() {
        return Err(AppError::Invalid("라벨명은 비어 있을 수 없습니다".into()));
    }
    if let Ok(existing) =
        conn.query_row("SELECT * FROM label WHERE name=?1", params![name], map_label)
    {
        return Ok(existing);
    }
    conn.execute(
        "INSERT INTO label (name, color) VALUES (?1, ?2)",
        params![name, color],
    )?;
    conn.query_row(
        "SELECT * FROM label WHERE id=?1",
        params![conn.last_insert_rowid()],
        map_label,
    )
    .map_err(AppError::Db)
}

pub fn assign(conn: &Connection, task_id: i64, label_id: i64) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO task_label (task_id, label_id) VALUES (?1, ?2)",
        params![task_id, label_id],
    )?;
    Ok(())
}

pub fn unassign(conn: &Connection, task_id: i64, label_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM task_label WHERE task_id=?1 AND label_id=?2",
        params![task_id, label_id],
    )?;
    Ok(())
}

/// 한 사업의 모든 태스크-라벨 매핑을 일괄 조회 (프론트에서 taskId→labels 맵 구성).
pub fn map_for_business(conn: &Connection, business_id: i64) -> Result<Vec<TaskLabel>> {
    let mut stmt = conn.prepare(
        "SELECT tl.task_id AS task_id, l.id AS label_id, l.name AS name, l.color AS color \
         FROM task_label tl \
         JOIN label l ON l.id = tl.label_id \
         JOIN task t ON t.id = tl.task_id \
         WHERE t.business_id = ?1",
    )?;
    let rows = stmt.query_map(params![business_id], |row| {
        Ok(TaskLabel {
            task_id: row.get("task_id")?,
            label_id: row.get("label_id")?,
            name: row.get("name")?,
            color: row.get("color")?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, repo::business, repo::task};

    fn setup() -> (Connection, i64, i64) {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "사업", "si", None).unwrap();
        let t = task::create(&c, b.id, None, "태스크", 2).unwrap();
        (c, b.id, t.id)
    }

    #[test]
    fn create_is_idempotent_by_name() {
        let (c, _, _) = setup();
        let a = create(&c, "백엔드", Some("#3b82f6")).unwrap();
        let b = create(&c, "백엔드", None).unwrap();
        assert_eq!(a.id, b.id);
        assert_eq!(list(&c).unwrap().len(), 1);
    }

    #[test]
    fn create_rejects_empty() {
        let (c, _, _) = setup();
        assert!(create(&c, "  ", None).is_err());
    }

    #[test]
    fn assign_and_map() {
        let (c, biz, task_id) = setup();
        let l1 = create(&c, "백엔드", Some("#3b82f6")).unwrap();
        let l2 = create(&c, "긴급", Some("#ef4444")).unwrap();
        assign(&c, task_id, l1.id).unwrap();
        assign(&c, task_id, l2.id).unwrap();
        assign(&c, task_id, l1.id).unwrap(); // 중복 무시
        let map = map_for_business(&c, biz).unwrap();
        assert_eq!(map.len(), 2);
        assert!(map.iter().all(|m| m.task_id == task_id));
    }

    #[test]
    fn unassign_removes() {
        let (c, biz, task_id) = setup();
        let l = create(&c, "백엔드", None).unwrap();
        assign(&c, task_id, l.id).unwrap();
        unassign(&c, task_id, l.id).unwrap();
        assert!(map_for_business(&c, biz).unwrap().is_empty());
    }

    #[test]
    fn deleting_task_cascades_assignment() {
        let (c, biz, task_id) = setup();
        let l = create(&c, "x", None).unwrap();
        assign(&c, task_id, l.id).unwrap();
        c.execute("DELETE FROM task WHERE id=?1", params![task_id]).unwrap();
        assert!(map_for_business(&c, biz).unwrap().is_empty());
        // 라벨 자체는 남음
        assert_eq!(list(&c).unwrap().len(), 1);
    }
}
