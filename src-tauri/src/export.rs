use crate::error::Result;
use crate::repo;
use rusqlite::Connection;
use serde_json::{json, Value};

/// 전체 데이터를 중첩 JSON으로 직렬화 (백업/내보내기).
/// 구조: businesses[] > { business, projects[]{project, tasks[]}, tasks(직속), documents[]{document, blocks[]} }
pub fn export_data(conn: &Connection) -> Result<Value> {
    let mut biz_arr: Vec<Value> = Vec::new();

    for b in repo::business::list(conn)? {
        let mut proj_arr: Vec<Value> = Vec::new();
        for p in repo::project::list_by_business(conn, b.id)? {
            let tasks = repo::task::list(conn, b.id, Some(p.id))?;
            proj_arr.push(json!({ "project": p, "tasks": tasks }));
        }

        let direct_tasks = repo::task::list(conn, b.id, None)?;

        let mut doc_arr: Vec<Value> = Vec::new();
        for d in repo::document::list_by_business(conn, b.id)? {
            let blocks = repo::document::list_blocks(conn, d.id)?;
            doc_arr.push(json!({ "document": d, "blocks": blocks }));
        }

        biz_arr.push(json!({
            "business": b,
            "projects": proj_arr,
            "tasks": direct_tasks,
            "documents": doc_arr,
        }));
    }

    Ok(json!({ "schemaVersion": 1, "businesses": biz_arr }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::repo::{business, document, project, task};

    #[test]
    fn export_empty_db_has_no_businesses() {
        let c = db::open_in_memory().unwrap();
        let v = export_data(&c).unwrap();
        assert_eq!(v["schemaVersion"], 1);
        assert_eq!(v["businesses"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn export_includes_full_tree() {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "사업", "si", None).unwrap();
        let p = project::create(&c, b.id, "P").unwrap();
        task::create(&c, b.id, Some(p.id), "태스크", 2).unwrap();
        let d = document::create(&c, b.id, None, "문서").unwrap();
        document::create_block(&c, d.id, "paragraph", "{\"text\":\"본문\"}", 1.0).unwrap();

        let v = export_data(&c).unwrap();
        let businesses = v["businesses"].as_array().unwrap();
        assert_eq!(businesses.len(), 1);
        assert_eq!(businesses[0]["business"]["name"], "사업");
        assert_eq!(businesses[0]["projects"].as_array().unwrap().len(), 1);
        assert_eq!(businesses[0]["projects"][0]["tasks"].as_array().unwrap().len(), 1);
        let docs = businesses[0]["documents"].as_array().unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0]["blocks"].as_array().unwrap().len(), 1);
    }
}
