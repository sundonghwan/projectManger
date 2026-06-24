use crate::error::Result;
use rusqlite::Connection;
use std::path::Path;

/// 순서대로 적용할 마이그레이션. PRAGMA user_version 으로 적용 개수를 추적한다.
const MIGRATIONS: &[&str] = &[
    include_str!("../migrations/0001_init.sql"),
    include_str!("../migrations/0002_templates_recurring.sql"),
    include_str!("../migrations/0003_deliverable_files.sql"),
    include_str!("../migrations/0004_document_body.sql"),
    include_str!("../migrations/0005_folders.sql"),
    include_str!("../migrations/0006_memo.sql"),
];

/// 아직 적용되지 않은 마이그레이션을 순서대로 실행. 두 번 호출해도 안전(멱등).
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    let applied: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let mut v = applied as usize;
    while v < MIGRATIONS.len() {
        conn.execute_batch(MIGRATIONS[v])?;
        v += 1;
    }
    conn.execute_batch(&format!("PRAGMA user_version = {};", MIGRATIONS.len()))?;
    Ok(())
}

/// 파일 DB를 열고 마이그레이션을 적용.
pub fn open_at(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    migrate(&conn)?;
    Ok(conn)
}

/// 테스트용 인메모리 DB (마이그레이션 적용 완료 상태로 반환).
#[cfg(test)]
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    migrate(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_creates_core_tables() {
        let conn = open_in_memory().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' \
                 AND name IN ('business','project','task','document','deliverable','server_connection')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 6);
    }

    #[test]
    fn migrate_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
        let v: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(v, MIGRATIONS.len() as i64);
    }

    #[test]
    fn foreign_keys_are_enforced() {
        let conn = open_in_memory().unwrap();
        // 존재하지 않는 business_id 로 project 삽입 → FK 위반으로 실패해야 함
        let res = conn.execute(
            "INSERT INTO project (business_id, name) VALUES (999, 'x')",
            [],
        );
        assert!(res.is_err());
    }
}
