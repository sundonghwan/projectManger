use crate::error::{AppError, Result};
use crate::store::ids::{new_id, now};
use crate::store::model::Template;
use crate::store::ops::{document, project, task};
use crate::store::Store;
use serde_json::Value;

const KINDS: [&str; 2] = ["project", "document"];

pub fn list(store: &Store) -> Result<Vec<Template>> {
    let mut out = store.templates.list();
    out.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
    Ok(out)
}

pub fn get(store: &Store, id: &str) -> Result<Template> {
    store.templates.get(id).cloned().ok_or(AppError::NotFound)
}

pub fn create(store: &mut Store, name: &str, kind: &str, payload: &str) -> Result<Template> {
    if name.trim().is_empty() {
        return Err(AppError::Invalid("템플릿명은 필수입니다".into()));
    }
    if !KINDS.contains(&kind) {
        return Err(AppError::Invalid(format!("알 수 없는 종류: {kind}")));
    }
    serde_json::from_str::<Value>(payload)
        .map_err(|e| AppError::Invalid(format!("payload JSON 오류: {e}")))?;
    let ts = now();
    let t = Template {
        id: new_id(),
        name: name.to_string(),
        kind: kind.to_string(),
        payload: payload.to_string(),
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.templates.put(t.clone())?;
    Ok(t)
}

pub fn delete(store: &mut Store, id: &str) -> Result<()> {
    if store.templates.get(id).is_none() {
        return Err(AppError::NotFound);
    }
    store.templates.remove(id)?;
    Ok(())
}

/// 프로젝트 템플릿을 사업에 적용 → 프로젝트 + 태스크 + 문서 생성. 새 프로젝트 id 반환.
pub fn apply_project(store: &mut Store, template_id: &str, business_id: &str) -> Result<String> {
    let t = get(store, template_id)?;
    if t.kind != "project" {
        return Err(AppError::Invalid("프로젝트 템플릿이 아닙니다".into()));
    }
    let payload: Value = serde_json::from_str(&t.payload).unwrap_or(Value::Null);
    let proj = project::create(store, business_id, &t.name)?;
    if let Some(tasks) = payload.get("tasks").and_then(|x| x.as_array()) {
        for tk in tasks {
            let title = tk.get("title").and_then(|x| x.as_str()).unwrap_or("태스크");
            let priority = tk.get("priority").and_then(|x| x.as_i64()).unwrap_or(2);
            task::create(store, business_id, Some(proj.id.as_str()), title, priority)?;
        }
    }
    if let Some(docs) = payload.get("documents").and_then(|x| x.as_array()) {
        for d in docs {
            let title = d.get("title").and_then(|x| x.as_str()).unwrap_or("제목 없음");
            document::create(store, business_id, Some(proj.id.as_str()), None, title)?;
        }
    }
    Ok(proj.id)
}

/// 문서 템플릿을 적용 → 문서 + 블록 생성. 새 문서 id 반환.
pub fn apply_document(
    store: &mut Store,
    template_id: &str,
    business_id: &str,
    project_id: Option<&str>,
) -> Result<String> {
    let t = get(store, template_id)?;
    if t.kind != "document" {
        return Err(AppError::Invalid("문서 템플릿이 아닙니다".into()));
    }
    let payload: Value = serde_json::from_str(&t.payload).unwrap_or(Value::Null);
    let doc = document::create(store, business_id, project_id, None, &t.name)?;
    if let Some(blocks) = payload.get("blocks").and_then(|x| x.as_array()) {
        for (i, b) in blocks.iter().enumerate() {
            let btype = b.get("type").and_then(|x| x.as_str()).unwrap_or("paragraph");
            let content = b.get("content").and_then(|x| x.as_str()).unwrap_or("{}");
            document::create_block(store, &doc.id, btype, content, (i + 1) as f64)?;
        }
    }
    Ok(doc.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::ops::{business, document, task};
    use crate::store::Store;

    fn store() -> Store {
        Store::open(std::env::temp_dir().join(format!("ops_tpl_{}", new_id()))).unwrap()
    }

    #[test]
    fn create_validates_kind_and_json() {
        let mut s = store();
        assert!(create(&mut s, "t", "bogus", "{}").is_err());
        assert!(create(&mut s, "t", "project", "not json").is_err());
        assert!(create(&mut s, "", "project", "{}").is_err());
        assert!(create(&mut s, "ok", "project", "{}").is_ok());
    }

    #[test]
    fn apply_project_creates_tasks_and_docs() {
        let mut s = store();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let payload = r#"{"tasks":[{"title":"킥오프","priority":3},{"title":"설계"}],"documents":[{"title":"요건"}]}"#;
        let t = create(&mut s, "표준 프로젝트", "project", payload).unwrap();
        let proj_id = apply_project(&mut s, &t.id, &b.id).unwrap();
        let tasks = task::list(&s, &b.id, Some(&proj_id)).unwrap();
        assert_eq!(tasks.len(), 2);
        assert!(tasks.iter().any(|t| t.title == "킥오프" && t.priority == 3));
        let docs = document::list_by_business(&s, &b.id).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].title, "요건");
    }

    #[test]
    fn apply_document_creates_blocks() {
        let mut s = store();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let payload = r#"{"blocks":[{"type":"heading","content":"{\"text\":\"개요\"}"},{"type":"paragraph","content":"{}"}]}"#;
        let t = create(&mut s, "회의록", "document", payload).unwrap();
        let doc_id = apply_document(&mut s, &t.id, &b.id, None).unwrap();
        let blocks = document::list_blocks(&s, &doc_id).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].r#type, "heading");
    }

    #[test]
    fn apply_rejects_wrong_kind() {
        let mut s = store();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let t = create(&mut s, "문서템플", "document", "{}").unwrap();
        assert!(apply_project(&mut s, &t.id, &b.id).is_err());
    }

    #[test]
    fn list_and_delete() {
        let mut s = store();
        let t = create(&mut s, "t", "project", "{}").unwrap();
        assert_eq!(list(&s).unwrap().len(), 1);
        delete(&mut s, &t.id).unwrap();
        assert!(list(&s).unwrap().is_empty());
        assert!(matches!(delete(&mut s, "nope"), Err(AppError::NotFound)));
    }
}
