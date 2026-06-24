use crate::error::{AppError, Result};
use crate::models::TrashItem;
use rusqlite::{params, Connection};

/// kind → 테이블명 (고정 매핑으로 SQL 주입 방지).
fn table_for(kind: &str) -> Result<&'static str> {
    match kind {
        "business" => Ok("business"),
        "project" => Ok("project"),
        "task" => Ok("task"),
        "document" => Ok("document"),
        "deliverable" => Ok("deliverable"),
        "memo" => Ok("memo"),
        other => Err(AppError::Invalid(format!("알 수 없는 종류: {other}"))),
    }
}

/// 보관된(archived_at IS NOT NULL) 항목을 모두 조회. business/project/document 는 name/title, task 는 title.
pub fn list_archived(conn: &Connection) -> Result<Vec<TrashItem>> {
    let sql = "
        SELECT 'business' AS kind, id, name AS title, archived_at, NULL AS file_size FROM business WHERE archived_at IS NOT NULL
        UNION ALL
        SELECT 'project', id, name, archived_at, NULL FROM project WHERE archived_at IS NOT NULL
        UNION ALL
        SELECT 'task', id, title, archived_at, NULL FROM task WHERE archived_at IS NOT NULL
        UNION ALL
        SELECT 'document', id, title, archived_at, NULL FROM document WHERE archived_at IS NOT NULL
        UNION ALL
        SELECT 'deliverable', id, title, archived_at, file_size FROM deliverable WHERE archived_at IS NOT NULL
        UNION ALL
        SELECT 'memo', id, CASE WHEN title='' THEN '(제목 없음)' ELSE title END, archived_at, NULL FROM memo WHERE archived_at IS NOT NULL
        ORDER BY archived_at DESC";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |r| {
        Ok(TrashItem {
            kind: r.get(0)?,
            id: r.get(1)?,
            title: r.get(2)?,
            archived_at: r.get(3)?,
            file_size: r.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// 보관 해제(복구).
pub fn restore(conn: &Connection, kind: &str, id: i64) -> Result<()> {
    let table = table_for(kind)?;
    let n = conn.execute(
        &format!("UPDATE {table} SET archived_at = NULL WHERE id = ?1"),
        params![id],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// 영구 삭제.
pub fn purge(conn: &Connection, kind: &str, id: i64) -> Result<()> {
    let table = table_for(kind)?;
    let n = conn.execute(&format!("DELETE FROM {table} WHERE id = ?1"), params![id])?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::repo::{business, task};

    #[test]
    fn lists_only_archived() {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "보관사업", "si", None).unwrap();
        business::create(&c, "활성사업", "si", None).unwrap();
        business::archive(&c, b.id).unwrap();
        let items = list_archived(&c).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, "business");
        assert_eq!(items[0].title, "보관사업");
    }

    #[test]
    fn restore_brings_back() {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "사업", "si", None).unwrap();
        let t = task::create(&c, b.id, None, "태스크", 2).unwrap();
        task::archive(&c, t.id).unwrap();
        assert_eq!(list_archived(&c).unwrap().len(), 1);
        restore(&c, "task", t.id).unwrap();
        assert!(list_archived(&c).unwrap().is_empty());
        assert!(task::get(&c, t.id).unwrap().archived_at.is_none());
    }

    #[test]
    fn purge_deletes_permanently() {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "사업", "si", None).unwrap();
        let t = task::create(&c, b.id, None, "태스크", 2).unwrap();
        task::archive(&c, t.id).unwrap();
        purge(&c, "task", t.id).unwrap();
        assert!(task::get(&c, t.id).is_err());
        assert!(list_archived(&c).unwrap().is_empty());
    }

    #[test]
    fn rejects_unknown_kind() {
        let c = db::open_in_memory().unwrap();
        assert!(restore(&c, "bogus", 1).is_err());
        assert!(purge(&c, "bogus", 1).is_err());
    }
}
