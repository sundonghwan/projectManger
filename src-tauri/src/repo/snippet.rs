use crate::error::{AppError, Result};
use crate::models::CommandSnippet;
use rusqlite::{params, Connection, Row};

fn map_row(row: &Row) -> rusqlite::Result<CommandSnippet> {
    Ok(CommandSnippet {
        id: row.get("id")?,
        server_connection_id: row.get("server_connection_id")?,
        name: row.get("name")?,
        command: row.get("command")?,
        sort_order: row.get("sort_order")?,
    })
}

pub fn list_by_server(conn: &Connection, server_id: i64) -> Result<Vec<CommandSnippet>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM command_snippet WHERE server_connection_id=?1 ORDER BY sort_order, id",
    )?;
    let rows = stmt.query_map(params![server_id], map_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn create(conn: &Connection, server_id: i64, name: &str, command: &str) -> Result<CommandSnippet> {
    if name.trim().is_empty() || command.trim().is_empty() {
        return Err(AppError::Invalid("이름과 명령은 필수입니다".into()));
    }
    let next: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order),0)+1 FROM command_snippet WHERE server_connection_id=?1",
        params![server_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO command_snippet (server_connection_id, name, command, sort_order) \
         VALUES (?1,?2,?3,?4)",
        params![server_id, name, command, next],
    )?;
    conn.query_row(
        "SELECT * FROM command_snippet WHERE id=?1",
        params![conn.last_insert_rowid()],
        map_row,
    )
    .map_err(AppError::Db)
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute("DELETE FROM command_snippet WHERE id=?1", params![id])?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, repo::business, repo::server};

    fn setup() -> (Connection, i64) {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "사업", "si", None).unwrap();
        let s = server::create(&c, b.id, None, "srv", "h", 22, "u", "key", None).unwrap();
        (c, s.id)
    }

    #[test]
    fn create_list_delete() {
        let (c, srv) = setup();
        let s = create(&c, srv, "배포", "./deploy.sh").unwrap();
        assert_eq!(list_by_server(&c, srv).unwrap().len(), 1);
        delete(&c, s.id).unwrap();
        assert!(list_by_server(&c, srv).unwrap().is_empty());
    }

    #[test]
    fn create_validates() {
        let (c, srv) = setup();
        assert!(create(&c, srv, "", "x").is_err());
        assert!(create(&c, srv, "n", "  ").is_err());
    }

    #[test]
    fn deleting_server_cascades_snippets() {
        let (c, srv) = setup();
        create(&c, srv, "n", "cmd").unwrap();
        c.execute("DELETE FROM server_connection WHERE id=?1", params![srv]).unwrap();
        let count: i64 = c
            .query_row("SELECT count(*) FROM command_snippet", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
