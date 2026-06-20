use crate::error::{AppError, Result};
use crate::models::Template;
use crate::repo::{document, project, task};
use rusqlite::{params, Connection, Row};
use serde_json::Value;

const KINDS: [&str; 2] = ["project", "document"];

fn map_row(row: &Row) -> rusqlite::Result<Template> {
    Ok(Template {
        id: row.get("id")?,
        name: row.get("name")?,
        kind: row.get("kind")?,
        payload: row.get("payload")?,
    })
}

pub fn list(conn: &Connection) -> Result<Vec<Template>> {
    let mut stmt = conn.prepare("SELECT * FROM template ORDER BY name, id")?;
    let rows = stmt.query_map([], map_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: i64) -> Result<Template> {
    conn.query_row("SELECT * FROM template WHERE id=?1", params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Db(other),
        })
}

pub fn create(conn: &Connection, name: &str, kind: &str, payload: &str) -> Result<Template> {
    if name.trim().is_empty() {
        return Err(AppError::Invalid("템플릿명은 필수입니다".into()));
    }
    if !KINDS.contains(&kind) {
        return Err(AppError::Invalid(format!("알 수 없는 종류: {kind}")));
    }
    // payload 가 유효한 JSON인지 확인
    serde_json::from_str::<Value>(payload).map_err(|e| AppError::Invalid(format!("payload JSON 오류: {e}")))?;
    conn.execute(
        "INSERT INTO template (name, kind, payload) VALUES (?1,?2,?3)",
        params![name, kind, payload],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute("DELETE FROM template WHERE id=?1", params![id])?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// 프로젝트 템플릿을 사업에 적용 → 프로젝트 + 태스크 + 문서 생성. 새 프로젝트 id 반환.
pub fn apply_project(conn: &Connection, template_id: i64, business_id: i64) -> Result<i64> {
    let t = get(conn, template_id)?;
    if t.kind != "project" {
        return Err(AppError::Invalid("프로젝트 템플릿이 아닙니다".into()));
    }
    let payload: Value = serde_json::from_str(&t.payload).unwrap_or(Value::Null);
    let proj = project::create(conn, business_id, &t.name)?;
    if let Some(tasks) = payload.get("tasks").and_then(|x| x.as_array()) {
        for tk in tasks {
            let title = tk.get("title").and_then(|x| x.as_str()).unwrap_or("태스크");
            let priority = tk.get("priority").and_then(|x| x.as_i64()).unwrap_or(2);
            task::create(conn, business_id, Some(proj.id), title, priority)?;
        }
    }
    if let Some(docs) = payload.get("documents").and_then(|x| x.as_array()) {
        for d in docs {
            let title = d.get("title").and_then(|x| x.as_str()).unwrap_or("제목 없음");
            document::create(conn, business_id, Some(proj.id), title)?;
        }
    }
    Ok(proj.id)
}

/// 문서 템플릿을 적용 → 문서 + 블록 생성. 새 문서 id 반환.
pub fn apply_document(
    conn: &Connection,
    template_id: i64,
    business_id: i64,
    project_id: Option<i64>,
) -> Result<i64> {
    let t = get(conn, template_id)?;
    if t.kind != "document" {
        return Err(AppError::Invalid("문서 템플릿이 아닙니다".into()));
    }
    let payload: Value = serde_json::from_str(&t.payload).unwrap_or(Value::Null);
    let doc = document::create(conn, business_id, project_id, &t.name)?;
    if let Some(blocks) = payload.get("blocks").and_then(|x| x.as_array()) {
        for (i, b) in blocks.iter().enumerate() {
            let btype = b.get("type").and_then(|x| x.as_str()).unwrap_or("paragraph");
            let content = b.get("content").and_then(|x| x.as_str()).unwrap_or("{}");
            document::create_block(conn, doc.id, btype, content, (i + 1) as f64)?;
        }
    }
    Ok(doc.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, repo::business};

    #[test]
    fn create_validates_kind_and_json() {
        let c = db::open_in_memory().unwrap();
        assert!(create(&c, "t", "bogus", "{}").is_err());
        assert!(create(&c, "t", "project", "not json").is_err());
        assert!(create(&c, "", "project", "{}").is_err());
        assert!(create(&c, "ok", "project", "{}").is_ok());
    }

    #[test]
    fn apply_project_creates_tasks_and_docs() {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "사업", "si", None).unwrap();
        let payload = r#"{"tasks":[{"title":"킥오프","priority":3},{"title":"설계"}],"documents":[{"title":"요건"}]}"#;
        let t = create(&c, "표준 프로젝트", "project", payload).unwrap();
        let proj_id = apply_project(&c, t.id, b.id).unwrap();

        let tasks = task::list(&c, b.id, Some(proj_id)).unwrap();
        assert_eq!(tasks.len(), 2);
        assert!(tasks.iter().any(|t| t.title == "킥오프" && t.priority == 3));
        let docs = document::list_by_business(&c, b.id).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].title, "요건");
    }

    #[test]
    fn apply_document_creates_blocks() {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "사업", "si", None).unwrap();
        let payload = r#"{"blocks":[{"type":"heading","content":"{\"text\":\"개요\"}"},{"type":"paragraph","content":"{}"}]}"#;
        let t = create(&c, "회의록", "document", payload).unwrap();
        let doc_id = apply_document(&c, t.id, b.id, None).unwrap();
        let blocks = document::list_blocks(&c, doc_id).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].r#type, "heading");
    }

    #[test]
    fn apply_rejects_wrong_kind() {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "사업", "si", None).unwrap();
        let t = create(&c, "문서템플", "document", "{}").unwrap();
        assert!(apply_project(&c, t.id, b.id).is_err());
    }

    #[test]
    fn list_and_delete() {
        let c = db::open_in_memory().unwrap();
        let t = create(&c, "t", "project", "{}").unwrap();
        assert_eq!(list(&c).unwrap().len(), 1);
        delete(&c, t.id).unwrap();
        assert!(list(&c).unwrap().is_empty());
    }
}
