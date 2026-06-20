use crate::error::Result;
use crate::models::SearchHit;
use rusqlite::{params, Connection};

/// 사업/프로젝트/태스크/문서 제목을 부분일치 검색. 보관 항목 제외.
pub fn search(conn: &Connection, query: &str) -> Result<Vec<SearchHit>> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let like = format!("%{q}%");
    let mut out: Vec<SearchHit> = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT id, name FROM business WHERE archived_at IS NULL AND name LIKE ?1 \
         ORDER BY name LIMIT 20",
    )?;
    for row in stmt.query_map(params![like], |r| {
        let id: i64 = r.get(0)?;
        Ok(SearchHit {
            kind: "business".into(),
            id,
            title: r.get(1)?,
            business_id: id,
            project_id: None,
        })
    })? {
        out.push(row?);
    }

    let mut stmt = conn.prepare(
        "SELECT id, business_id, name FROM project WHERE archived_at IS NULL AND name LIKE ?1 \
         ORDER BY name LIMIT 20",
    )?;
    for row in stmt.query_map(params![like], |r| {
        let id: i64 = r.get(0)?;
        Ok(SearchHit {
            kind: "project".into(),
            id,
            title: r.get(2)?,
            business_id: r.get(1)?,
            project_id: Some(id),
        })
    })? {
        out.push(row?);
    }

    let mut stmt = conn.prepare(
        "SELECT id, business_id, project_id, title FROM task \
         WHERE archived_at IS NULL AND title LIKE ?1 ORDER BY title LIMIT 30",
    )?;
    for row in stmt.query_map(params![like], |r| {
        Ok(SearchHit {
            kind: "task".into(),
            id: r.get(0)?,
            business_id: r.get(1)?,
            project_id: r.get(2)?,
            title: r.get(3)?,
        })
    })? {
        out.push(row?);
    }

    let mut stmt = conn.prepare(
        "SELECT id, business_id, project_id, title FROM document \
         WHERE archived_at IS NULL AND title LIKE ?1 ORDER BY title LIMIT 20",
    )?;
    for row in stmt.query_map(params![like], |r| {
        Ok(SearchHit {
            kind: "document".into(),
            id: r.get(0)?,
            business_id: r.get(1)?,
            project_id: r.get(2)?,
            title: r.get(3)?,
        })
    })? {
        out.push(row?);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::repo::{business, document, project, task};

    #[test]
    fn empty_query_returns_nothing() {
        let c = db::open_in_memory().unwrap();
        business::create(&c, "사업", "si", None).unwrap();
        assert!(search(&c, "  ").unwrap().is_empty());
    }

    #[test]
    fn finds_across_kinds() {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "알파 사업", "si", None).unwrap();
        let p = project::create(&c, b.id, "알파 프로젝트").unwrap();
        task::create(&c, b.id, Some(p.id), "알파 태스크", 2).unwrap();
        document::create(&c, b.id, None, "알파 문서").unwrap();

        let hits = search(&c, "알파").unwrap();
        let kinds: Vec<&str> = hits.iter().map(|h| h.kind.as_str()).collect();
        assert!(kinds.contains(&"business"));
        assert!(kinds.contains(&"project"));
        assert!(kinds.contains(&"task"));
        assert!(kinds.contains(&"document"));
        assert_eq!(hits.len(), 4);
    }

    #[test]
    fn excludes_archived_and_non_matching() {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "찾을것", "si", None).unwrap();
        business::create(&c, "다른것", "si", None).unwrap();
        let t = task::create(&c, b.id, None, "보관될태스크", 2).unwrap();
        task::archive(&c, t.id).unwrap();

        let hits = search(&c, "찾을").unwrap();
        assert_eq!(hits.len(), 1);
        assert!(search(&c, "보관될").unwrap().is_empty());
    }
}
