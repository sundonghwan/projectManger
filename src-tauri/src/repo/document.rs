use crate::error::{AppError, Result};
use crate::models::{Block, Document};
use rusqlite::{params, Connection, Row};

fn map_doc(row: &Row) -> rusqlite::Result<Document> {
    Ok(Document {
        id: row.get("id")?,
        business_id: row.get("business_id")?,
        project_id: row.get("project_id")?,
        title: row.get("title")?,
        icon: row.get("icon")?,
        body: row.get("body")?,
        sort_order: row.get("sort_order")?,
        archived_at: row.get("archived_at")?,
        created_at: row.get("created_at")?,
    })
}

fn map_block(row: &Row) -> rusqlite::Result<Block> {
    Ok(Block {
        id: row.get("id")?,
        document_id: row.get("document_id")?,
        parent_block_id: row.get("parent_block_id")?,
        r#type: row.get("type")?,
        content: row.get("content")?,
        sort_order: row.get("sort_order")?,
    })
}

/// 사업의 모든(직속+프로젝트 소속) 문서를 조회. 트리 배치는 프론트가 project_id로 결정.
pub fn list_by_business(conn: &Connection, business_id: i64) -> Result<Vec<Document>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM document WHERE business_id=?1 AND archived_at IS NULL \
         ORDER BY sort_order, id",
    )?;
    let rows = stmt.query_map(params![business_id], map_doc)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: i64) -> Result<Document> {
    conn.query_row("SELECT * FROM document WHERE id=?1", params![id], map_doc)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Db(other),
        })
}

/// 문서 생성. project_id 가 있으면 사업 소속이어야 함.
pub fn create(
    conn: &Connection,
    business_id: i64,
    project_id: Option<i64>,
    title: &str,
) -> Result<Document> {
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
    let title = if title.trim().is_empty() { "제목 없음" } else { title };
    let next: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order),0)+1 FROM document WHERE business_id=?1",
        params![business_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO document (business_id, project_id, title, sort_order) VALUES (?1,?2,?3,?4)",
        params![business_id, project_id, title, next],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn rename(conn: &Connection, id: i64, title: &str) -> Result<Document> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::Invalid("문서 제목은 비어 있을 수 없습니다".into()));
    }
    let n = conn.execute(
        "UPDATE document SET title=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, title],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

/// 문서 본문(마크다운) 저장.
pub fn set_body(conn: &Connection, id: i64, body: &str) -> Result<()> {
    let n = conn.execute(
        "UPDATE document SET body=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, body],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn archive(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute(
        "UPDATE document SET archived_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

// ---- 블록 ----

pub fn list_blocks(conn: &Connection, document_id: i64) -> Result<Vec<Block>> {
    let mut stmt = conn
        .prepare("SELECT * FROM block WHERE document_id=?1 ORDER BY sort_order, id")?;
    let rows = stmt.query_map(params![document_id], map_block)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get_block(conn: &Connection, id: i64) -> Result<Block> {
    conn.query_row("SELECT * FROM block WHERE id=?1", params![id], map_block)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Db(other),
        })
}

pub fn create_block(
    conn: &Connection,
    document_id: i64,
    type_: &str,
    content: &str,
    sort_order: f64,
) -> Result<Block> {
    conn.execute(
        "INSERT INTO block (document_id, type, content, sort_order) VALUES (?1,?2,?3,?4)",
        params![document_id, type_, content, sort_order],
    )?;
    get_block(conn, conn.last_insert_rowid())
}

pub fn update_block(conn: &Connection, id: i64, type_: &str, content: &str) -> Result<Block> {
    let n = conn.execute(
        "UPDATE block SET type=?2, content=?3, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') \
         WHERE id=?1",
        params![id, type_, content],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get_block(conn, id)
}

pub fn delete_block(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute("DELETE FROM block WHERE id=?1", params![id])?;
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
    fn create_doc_direct_and_in_project() {
        let (c, biz, proj) = setup();
        let d1 = create(&c, biz, None, "사업 직속 문서").unwrap();
        let d2 = create(&c, biz, Some(proj), "프로젝트 문서").unwrap();
        assert_eq!(d1.project_id, None);
        assert_eq!(d2.project_id, Some(proj));
        assert_eq!(list_by_business(&c, biz).unwrap().len(), 2);
    }

    #[test]
    fn create_doc_blank_title_defaults() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, "   ").unwrap();
        assert_eq!(d.title, "제목 없음");
    }

    #[test]
    fn create_doc_rejects_foreign_project() {
        let (c, biz, _) = setup();
        let other = business::create(&c, "다른", "ops", None).unwrap();
        let op = project::create(&c, other.id, "P").unwrap();
        assert!(create(&c, biz, Some(op.id), "x").is_err());
    }

    #[test]
    fn rename_and_archive() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, "원래").unwrap();
        assert_eq!(rename(&c, d.id, "새이름").unwrap().title, "새이름");
        archive(&c, d.id).unwrap();
        assert!(list_by_business(&c, biz).unwrap().is_empty());
    }

    #[test]
    fn blocks_crud_and_order() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, "문서").unwrap();
        let b1 = create_block(&c, d.id, "heading", "{\"text\":\"제목\"}", 1.0).unwrap();
        let _b2 = create_block(&c, d.id, "paragraph", "{\"text\":\"본문\"}", 2.0).unwrap();
        let blocks = list_blocks(&c, d.id).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].r#type, "heading");

        let updated = update_block(&c, b1.id, "heading", "{\"text\":\"수정됨\"}").unwrap();
        assert!(updated.content.contains("수정됨"));

        delete_block(&c, b1.id).unwrap();
        assert_eq!(list_blocks(&c, d.id).unwrap().len(), 1);
    }

    #[test]
    fn deleting_document_cascades_blocks() {
        let (c, biz, _) = setup();
        let d = create(&c, biz, None, "문서").unwrap();
        create_block(&c, d.id, "paragraph", "{}", 1.0).unwrap();
        c.execute("DELETE FROM document WHERE id=?1", params![d.id]).unwrap();
        let count: i64 = c.query_row("SELECT count(*) FROM block", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }
}
