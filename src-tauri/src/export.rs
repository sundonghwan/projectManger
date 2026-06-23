use crate::error::{AppError, Result};
use crate::repo;
use rusqlite::{params, Connection};
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

fn s(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(|x| x.to_string())
}
fn i(v: &Value, key: &str) -> Option<i64> {
    v.get(key).and_then(|x| x.as_i64())
}

/// 내보낸 JSON을 현재 DB에 추가(append) 가져오기. 새 id로 삽입하며 계층 관계를 재매핑.
pub fn import_data(conn: &Connection, value: &Value) -> Result<()> {
    let businesses = value
        .get("businesses")
        .and_then(|x| x.as_array())
        .ok_or_else(|| AppError::Invalid("businesses 배열이 없습니다".into()))?;

    for node in businesses {
        let b = node.get("business").unwrap_or(&Value::Null);
        let name = s(b, "name").ok_or_else(|| AppError::Invalid("business.name 누락".into()))?;
        let type_ = s(b, "type").unwrap_or_else(|| "etc".into());
        conn.execute(
            "INSERT INTO business (name, type, color, description, status, sort_order) \
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                name,
                type_,
                s(b, "color"),
                s(b, "description"),
                s(b, "status").unwrap_or_else(|| "active".into()),
                b.get("sortOrder").and_then(|x| x.as_f64()).unwrap_or(0.0),
            ],
        )?;
        let new_biz = conn.last_insert_rowid();

        // 프로젝트 + 프로젝트 소속 태스크
        if let Some(projects) = node.get("projects").and_then(|x| x.as_array()) {
            for pn in projects {
                let p = pn.get("project").unwrap_or(&Value::Null);
                let pname = s(p, "name").unwrap_or_else(|| "프로젝트".into());
                conn.execute(
                    "INSERT INTO project (business_id, name, description, status, start_date, due_date, sort_order) \
                     VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![
                        new_biz,
                        pname,
                        s(p, "description"),
                        s(p, "status").unwrap_or_else(|| "active".into()),
                        s(p, "startDate"),
                        s(p, "dueDate"),
                        p.get("sortOrder").and_then(|x| x.as_f64()).unwrap_or(0.0),
                    ],
                )?;
                let new_proj = conn.last_insert_rowid();
                if let Some(tasks) = pn.get("tasks").and_then(|x| x.as_array()) {
                    for t in tasks {
                        insert_task(conn, new_biz, Some(new_proj), t)?;
                    }
                }
            }
        }

        // 사업 직속 태스크 (projectId == null)
        if let Some(tasks) = node.get("tasks").and_then(|x| x.as_array()) {
            for t in tasks {
                if t.get("projectId").map(|x| x.is_null()).unwrap_or(true) {
                    insert_task(conn, new_biz, None, t)?;
                }
            }
        }

        // 문서 + 블록
        if let Some(docs) = node.get("documents").and_then(|x| x.as_array()) {
            for dn in docs {
                let d = dn.get("document").unwrap_or(&Value::Null);
                conn.execute(
                    "INSERT INTO document (business_id, project_id, title, icon, sort_order) \
                     VALUES (?1, NULL, ?2, ?3, ?4)",
                    params![
                        new_biz,
                        s(d, "title").unwrap_or_else(|| "제목 없음".into()),
                        s(d, "icon"),
                        d.get("sortOrder").and_then(|x| x.as_f64()).unwrap_or(0.0),
                    ],
                )?;
                let new_doc = conn.last_insert_rowid();
                if let Some(blocks) = dn.get("blocks").and_then(|x| x.as_array()) {
                    for blk in blocks {
                        conn.execute(
                            "INSERT INTO block (document_id, type, content, sort_order) \
                             VALUES (?1,?2,?3,?4)",
                            params![
                                new_doc,
                                s(blk, "type").unwrap_or_else(|| "paragraph".into()),
                                s(blk, "content").unwrap_or_else(|| "{}".into()),
                                blk.get("sortOrder").and_then(|x| x.as_f64()).unwrap_or(0.0),
                            ],
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn insert_task(conn: &Connection, biz: i64, proj: Option<i64>, t: &Value) -> Result<()> {
    conn.execute(
        "INSERT INTO task (business_id, project_id, title, description, status, priority, due_date, sort_order) \
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            biz,
            proj,
            s(t, "title").unwrap_or_else(|| "태스크".into()),
            s(t, "description"),
            s(t, "status").unwrap_or_else(|| "todo".into()),
            i(t, "priority").unwrap_or(2),
            s(t, "dueDate"),
            t.get("sortOrder").and_then(|x| x.as_f64()).unwrap_or(0.0),
        ],
    )?;
    Ok(())
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
        let d = document::create(&c, b.id, None, None, "문서").unwrap();
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

    #[test]
    fn import_roundtrip_into_fresh_db() {
        // 원본 DB 구성
        let src = db::open_in_memory().unwrap();
        let b = business::create(&src, "사업", "si", None).unwrap();
        let p = project::create(&src, b.id, "P").unwrap();
        task::create(&src, b.id, Some(p.id), "프로젝트 태스크", 3).unwrap();
        task::create(&src, b.id, None, "직속 태스크", 2).unwrap();
        let d = document::create(&src, b.id, None, None, "문서").unwrap();
        document::create_block(&src, d.id, "paragraph", "{\"text\":\"x\"}", 1.0).unwrap();
        let exported = export_data(&src).unwrap();

        // 새 DB로 가져오기
        let dst = db::open_in_memory().unwrap();
        import_data(&dst, &exported).unwrap();

        let count = |sql: &str| -> i64 { dst.query_row(sql, [], |r| r.get(0)).unwrap() };
        assert_eq!(count("SELECT count(*) FROM business"), 1);
        assert_eq!(count("SELECT count(*) FROM project"), 1);
        assert_eq!(count("SELECT count(*) FROM task"), 2);
        assert_eq!(count("SELECT count(*) FROM document"), 1);
        assert_eq!(count("SELECT count(*) FROM block"), 1);
        // 프로젝트 소속/직속 태스크가 올바르게 재매핑됨
        assert_eq!(count("SELECT count(*) FROM task WHERE project_id IS NOT NULL"), 1);
        assert_eq!(count("SELECT count(*) FROM task WHERE project_id IS NULL"), 1);
    }

    #[test]
    fn import_rejects_invalid_shape() {
        let c = db::open_in_memory().unwrap();
        assert!(import_data(&c, &serde_json::json!({"nope": 1})).is_err());
    }
}
