use crate::error::{AppError, Result};
use crate::store::collection::Collection;
use crate::store::entity::Entity;
use crate::store::ids::now;
use crate::store::Store;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashItem {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub archived_at: String,
    pub file_size: Option<i64>,
}

/// 보관된(archived_at IS NOT NULL) 항목 전체를 archived_at DESC 로.
pub fn list_archived(store: &Store) -> Result<Vec<TrashItem>> {
    let mut out: Vec<TrashItem> = Vec::new();
    for b in store.businesses.list() {
        if let Some(a) = b.archived_at {
            out.push(TrashItem { kind: "business".into(), id: b.id, title: b.name, archived_at: a, file_size: None });
        }
    }
    for p in store.projects.list() {
        if let Some(a) = p.archived_at {
            out.push(TrashItem { kind: "project".into(), id: p.id, title: p.name, archived_at: a, file_size: None });
        }
    }
    for t in store.tasks.list() {
        if let Some(a) = t.archived_at {
            out.push(TrashItem { kind: "task".into(), id: t.id, title: t.title, archived_at: a, file_size: None });
        }
    }
    for d in store.documents.list() {
        if let Some(a) = d.archived_at {
            out.push(TrashItem { kind: "document".into(), id: d.id, title: d.title, archived_at: a, file_size: None });
        }
    }
    for d in store.deliverables.list() {
        if let Some(a) = d.archived_at {
            out.push(TrashItem { kind: "deliverable".into(), id: d.id, title: d.title, archived_at: a, file_size: d.file_size });
        }
    }
    for m in store.memos.list() {
        if let Some(a) = m.archived_at {
            let title = if m.title.is_empty() { "(제목 없음)".to_string() } else { m.title };
            out.push(TrashItem { kind: "memo".into(), id: m.id, title, archived_at: a, file_size: None });
        }
    }
    out.sort_by(|a, b| b.archived_at.cmp(&a.archived_at));
    Ok(out)
}

/// 보관 해제(복구). archived_at=None + updated_at bump.
pub fn restore(store: &mut Store, kind: &str, id: &str) -> Result<()> {
    match kind {
        "business" => {
            let mut x = store.businesses.get(id).cloned().ok_or(AppError::NotFound)?;
            x.archived_at = None;
            x.updated_at = now();
            store.businesses.put(x)?;
        }
        "project" => {
            let mut x = store.projects.get(id).cloned().ok_or(AppError::NotFound)?;
            x.archived_at = None;
            x.updated_at = now();
            store.projects.put(x)?;
        }
        "task" => {
            let mut x = store.tasks.get(id).cloned().ok_or(AppError::NotFound)?;
            x.archived_at = None;
            x.updated_at = now();
            store.tasks.put(x)?;
        }
        "document" => {
            let mut x = store.documents.get(id).cloned().ok_or(AppError::NotFound)?;
            x.archived_at = None;
            x.updated_at = now();
            store.documents.put(x)?;
        }
        "deliverable" => {
            let mut x = store.deliverables.get(id).cloned().ok_or(AppError::NotFound)?;
            x.archived_at = None;
            x.updated_at = now();
            store.deliverables.put(x)?;
        }
        "memo" => {
            let mut x = store.memos.get(id).cloned().ok_or(AppError::NotFound)?;
            x.archived_at = None;
            x.updated_at = now();
            store.memos.put(x)?;
        }
        other => return Err(AppError::Invalid(format!("알 수 없는 종류: {other}"))),
    }
    Ok(())
}

/// 영구 삭제. FK CASCADE/SET NULL 규칙을 앱 로직으로 재현.
pub fn purge(store: &mut Store, kind: &str, id: &str) -> Result<()> {
    match kind {
        "business" => purge_business(store, id),
        "project" => purge_project(store, id),
        "task" => purge_task(store, id),
        "document" => purge_document(store, id),
        "deliverable" => purge_simple(store, id, "deliverable"),
        "memo" => purge_simple(store, id, "memo"),
        other => Err(AppError::Invalid(format!("알 수 없는 종류: {other}"))),
    }
}

/// 컬렉션에서 술어를 만족하는 id 목록.
fn collect_ids<T: Entity>(c: &Collection<T>, pred: impl Fn(&T) -> bool) -> Vec<String> {
    c.list()
        .into_iter()
        .filter(|x| pred(x))
        .map(|x| x.id().to_string())
        .collect()
}

/// 주어진 태스크들에 걸린 task_label 관계를 모두 제거.
fn remove_task_labels_for(store: &mut Store, task_ids: &[String]) -> Result<()> {
    if task_ids.is_empty() {
        return Ok(());
    }
    let set: HashSet<&str> = task_ids.iter().map(|s| s.as_str()).collect();
    let tl_ids: Vec<String> = store
        .task_labels
        .list()
        .into_iter()
        .filter(|tl| set.contains(tl.task_id.as_str()))
        .map(|tl| tl.id)
        .collect();
    for x in tl_ids {
        store.task_labels.remove(&x)?;
    }
    Ok(())
}

fn purge_business(store: &mut Store, id: &str) -> Result<()> {
    if store.businesses.get(id).is_none() {
        return Err(AppError::NotFound);
    }
    let task_ids = collect_ids(&store.tasks, |t| t.business_id == id);
    remove_task_labels_for(store, &task_ids)?;
    for x in &task_ids {
        store.tasks.remove(x)?;
    }
    for x in collect_ids(&store.projects, |p| p.business_id == id) {
        store.projects.remove(&x)?;
    }
    for x in collect_ids(&store.documents, |d| d.business_id == id) {
        store.documents.remove(&x)?;
    }
    for x in collect_ids(&store.deliverables, |d| d.business_id == id) {
        store.deliverables.remove(&x)?;
    }
    for x in collect_ids(&store.memos, |m| m.business_id == id) {
        store.memos.remove(&x)?;
    }
    for x in collect_ids(&store.folders, |f| f.business_id == id) {
        store.folders.remove(&x)?;
    }
    for x in collect_ids(&store.recurring, |r| r.business_id == id) {
        store.recurring.remove(&x)?;
    }
    store.businesses.remove(id)?;
    Ok(())
}

fn purge_project(store: &mut Store, id: &str) -> Result<()> {
    if store.projects.get(id).is_none() {
        return Err(AppError::NotFound);
    }
    let task_ids = collect_ids(&store.tasks, |t| t.project_id.as_deref() == Some(id));
    remove_task_labels_for(store, &task_ids)?;
    for x in &task_ids {
        store.tasks.remove(x)?;
    }
    for x in collect_ids(&store.documents, |d| d.project_id.as_deref() == Some(id)) {
        store.documents.remove(&x)?;
    }
    for x in collect_ids(&store.deliverables, |d| d.project_id.as_deref() == Some(id)) {
        store.deliverables.remove(&x)?;
    }
    for x in collect_ids(&store.recurring, |r| r.project_id.as_deref() == Some(id)) {
        store.recurring.remove(&x)?;
    }
    store.projects.remove(id)?;
    Ok(())
}

fn purge_task(store: &mut Store, id: &str) -> Result<()> {
    if store.tasks.get(id).is_none() {
        return Err(AppError::NotFound);
    }
    let all = store.tasks.list();
    let mut to_remove: Vec<String> = vec![id.to_string()];
    let mut i = 0;
    while i < to_remove.len() {
        let cur = to_remove[i].clone();
        for t in &all {
            if t.parent_task_id.as_deref() == Some(cur.as_str()) && !to_remove.iter().any(|x| x == &t.id) {
                to_remove.push(t.id.clone());
            }
        }
        i += 1;
    }
    remove_task_labels_for(store, &to_remove)?;
    for x in &to_remove {
        store.tasks.remove(x)?;
    }
    Ok(())
}

fn purge_document(store: &mut Store, id: &str) -> Result<()> {
    if store.documents.get(id).is_none() {
        return Err(AppError::NotFound);
    }
    // deliverable.document_id 를 SET NULL
    let to_fix: Vec<_> = store
        .deliverables
        .list()
        .into_iter()
        .filter(|d| d.document_id.as_deref() == Some(id))
        .collect();
    for mut d in to_fix {
        d.document_id = None;
        d.updated_at = now();
        store.deliverables.put(d)?;
    }
    store.documents.remove(id)?;
    Ok(())
}

fn purge_simple(store: &mut Store, id: &str, kind: &str) -> Result<()> {
    match kind {
        "deliverable" => {
            if store.deliverables.get(id).is_none() {
                return Err(AppError::NotFound);
            }
            store.deliverables.remove(id)?;
        }
        "memo" => {
            if store.memos.get(id).is_none() {
                return Err(AppError::NotFound);
            }
            store.memos.remove(id)?;
        }
        _ => return Err(AppError::Invalid(format!("알 수 없는 종류: {kind}"))),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::ops::{business, deliverable, document, label, memo, project, task};
    use crate::store::Store;

    fn store() -> Store {
        Store::open(std::env::temp_dir().join(format!("ops_trash_{}", new_id()))).unwrap()
    }

    #[test]
    fn lists_only_archived_desc_with_memo_default_title() {
        let mut s = store();
        let b = business::create(&mut s, "보관사업", "si", None).unwrap();
        business::create(&mut s, "활성사업", "si", None).unwrap();
        let m = memo::create(&mut s, &b.id, "", "본문").unwrap();
        business::archive(&mut s, &b.id).unwrap();
        memo::archive(&mut s, &m.id).unwrap();
        let items = list_archived(&s).unwrap();
        assert_eq!(items.len(), 2);
        // archived_at DESC: 메모가 사업보다 나중에 보관됨 → 먼저
        assert_eq!(items[0].kind, "memo");
        assert_eq!(items[0].title, "(제목 없음)");
        assert!(items.iter().any(|i| i.kind == "business" && i.title == "보관사업"));
    }

    #[test]
    fn restore_brings_back() {
        let mut s = store();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let t = task::create(&mut s, &b.id, None, "태스크", 2).unwrap();
        task::archive(&mut s, &t.id).unwrap();
        assert_eq!(list_archived(&s).unwrap().len(), 1);
        restore(&mut s, "task", &t.id).unwrap();
        assert!(list_archived(&s).unwrap().is_empty());
        assert!(task::get(&s, &t.id).unwrap().archived_at.is_none());
    }

    #[test]
    fn rejects_unknown_kind() {
        let mut s = store();
        assert!(restore(&mut s, "bogus", "1").is_err());
        assert!(purge(&mut s, "bogus", "1").is_err());
    }

    #[test]
    fn purge_missing_returns_not_found() {
        let mut s = store();
        assert!(matches!(purge(&mut s, "task", "nope"), Err(AppError::NotFound)));
    }

    #[test]
    fn purge_business_cascades_everything() {
        let mut s = store();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let p = project::create(&mut s, &b.id, "P").unwrap();
        let t = task::create(&mut s, &b.id, Some(&p.id), "T", 2).unwrap();
        let l = label::create(&mut s, "L", None).unwrap();
        label::assign(&mut s, &t.id, &l.id).unwrap();
        document::create(&mut s, &b.id, Some(&p.id), None, "D").unwrap();
        deliverable::create(&mut s, &b.id, Some(&p.id), None, "DV", "file").unwrap();
        memo::create(&mut s, &b.id, "M", "").unwrap();

        purge(&mut s, "business", &b.id).unwrap();
        assert!(business::list(&s).unwrap().is_empty());
        assert!(project::list_by_business(&s, &b.id).unwrap().is_empty());
        assert!(task::list(&s, &b.id, None).unwrap().is_empty());
        assert!(document::list_by_business(&s, &b.id).unwrap().is_empty());
        assert!(deliverable::list_by_business(&s, &b.id).unwrap().is_empty());
        assert!(memo::list_by_business(&s, &b.id).unwrap().is_empty());
        // task_label 도 정리됨(라벨 자체는 남음)
        assert!(label::map_for_business(&s, &b.id).unwrap().is_empty());
        assert_eq!(label::list(&s).unwrap().len(), 1);
    }

    #[test]
    fn purge_project_cascades_children() {
        let mut s = store();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let p = project::create(&mut s, &b.id, "P").unwrap();
        task::create(&mut s, &b.id, Some(&p.id), "T", 2).unwrap();
        document::create(&mut s, &b.id, Some(&p.id), None, "D").unwrap();
        // 프로젝트 소속이 아닌 직속 태스크는 남아야
        task::create(&mut s, &b.id, None, "직속", 2).unwrap();

        purge(&mut s, "project", &p.id).unwrap();
        let tasks = task::list(&s, &b.id, None).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "직속");
        assert!(document::list_by_business(&s, &b.id).unwrap().is_empty());
    }

    #[test]
    fn purge_task_cascades_subtasks_and_labels() {
        let mut s = store();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let parent = task::create(&mut s, &b.id, None, "부모", 2).unwrap();
        // 자식 태스크를 parent_task_id 지정해 직접 put
        let child = task::create(&mut s, &b.id, None, "자식", 2).unwrap();
        let mut child2 = task::get(&s, &child.id).unwrap();
        child2.parent_task_id = Some(parent.id.clone());
        s.tasks.put(child2).unwrap();
        let l = label::create(&mut s, "L", None).unwrap();
        label::assign(&mut s, &parent.id, &l.id).unwrap();

        purge(&mut s, "task", &parent.id).unwrap();
        assert!(task::get(&s, &parent.id).is_err());
        assert!(task::get(&s, &child.id).is_err()); // 자식도 cascade
        assert!(label::map_for_business(&s, &b.id).unwrap().is_empty());
    }

    #[test]
    fn purge_document_nulls_deliverable_link() {
        let mut s = store();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let doc = document::create(&mut s, &b.id, None, None, "문서").unwrap();
        let dv = deliverable::create(&mut s, &b.id, None, None, "산출물", "document").unwrap();
        let mut dv2 = deliverable::get(&s, &dv.id).unwrap();
        dv2.document_id = Some(doc.id.clone());
        s.deliverables.put(dv2).unwrap();

        purge(&mut s, "document", &doc.id).unwrap();
        assert!(document::get(&s, &doc.id).is_err());
        // 산출물은 남고 document_id 만 None
        assert_eq!(deliverable::get(&s, &dv.id).unwrap().document_id, None);
    }
}
