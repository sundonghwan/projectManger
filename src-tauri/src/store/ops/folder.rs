use crate::error::{AppError, Result};
use crate::store::ids::{new_id, now};
use crate::store::model::Folder;
use crate::store::ops::util::cmp_sort;
use crate::store::Store;
use std::collections::HashSet;

const KINDS: [&str; 2] = ["document", "deliverable"];

pub fn list_by_business(store: &Store, business_id: &str) -> Result<Vec<Folder>> {
    let mut out: Vec<Folder> = store
        .folders
        .list()
        .into_iter()
        .filter(|f| f.business_id == business_id && f.archived_at.is_none())
        .collect();
    out.sort_by(|a, b| cmp_sort(a.sort_order, &a.id, b.sort_order, &b.id));
    Ok(out)
}

pub fn get(store: &Store, id: &str) -> Result<Folder> {
    store.folders.get(id).cloned().ok_or(AppError::NotFound)
}

/// folder_id 가 None 이면 통과. 있으면 같은 사업·kind 소속이어야 함.
pub fn ensure_owns(
    store: &Store,
    folder_id: Option<&str>,
    business_id: &str,
    kind: &str,
) -> Result<()> {
    let Some(fid) = folder_id else { return Ok(()) };
    let f = get(store, fid)?;
    if f.business_id != business_id || f.kind != kind {
        return Err(AppError::Invalid("폴더가 해당 사업/종류 소속이 아닙니다".into()));
    }
    Ok(())
}

fn next_sort(store: &Store, business_id: &str, kind: &str, parent_id: Option<&str>) -> f64 {
    store
        .folders
        .list()
        .iter()
        .filter(|f| f.business_id == business_id && f.kind == kind && f.parent_id.as_deref() == parent_id)
        .map(|f| f.sort_order)
        .fold(0.0_f64, f64::max)
        + 1.0
}

pub fn create(
    store: &mut Store,
    business_id: &str,
    kind: &str,
    parent_id: Option<&str>,
    name: &str,
) -> Result<Folder> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invalid("폴더 이름은 비어 있을 수 없습니다".into()));
    }
    if !KINDS.contains(&kind) {
        return Err(AppError::Invalid(format!("알 수 없는 종류: {kind}")));
    }
    if let Some(pid) = parent_id {
        let parent = get(store, pid)?;
        if parent.business_id != business_id || parent.kind != kind {
            return Err(AppError::Invalid("상위 폴더가 해당 사업/종류 소속이 아닙니다".into()));
        }
        if parent.parent_id.is_some() {
            return Err(AppError::Invalid("폴더는 2단계까지만 만들 수 있습니다".into()));
        }
    }
    let sort_order = next_sort(store, business_id, kind, parent_id);
    let ts = now();
    let f = Folder {
        id: new_id(),
        business_id: business_id.to_string(),
        kind: kind.to_string(),
        parent_id: parent_id.map(|s| s.to_string()),
        name: name.to_string(),
        sort_order,
        archived_at: None,
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.folders.put(f.clone())?;
    Ok(f)
}

pub fn rename(store: &mut Store, id: &str, name: &str) -> Result<Folder> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invalid("폴더 이름은 비어 있을 수 없습니다".into()));
    }
    let mut f = get(store, id)?;
    f.name = name.to_string();
    f.updated_at = now();
    store.folders.put(f.clone())?;
    Ok(f)
}

/// 폴더 삭제(영구). 자신 + 직계 자식 폴더(2단계)를 삭제하고,
/// 그 폴더들을 가리키던 문서/산출물의 folder_id 를 None(미분류)으로 정리한다.
pub fn delete(store: &mut Store, id: &str) -> Result<()> {
    if store.folders.get(id).is_none() {
        return Err(AppError::NotFound);
    }
    // 삭제 대상: 자신 + 직계 자식 폴더
    let mut to_delete: Vec<String> = vec![id.to_string()];
    for f in store.folders.list() {
        if f.parent_id.as_deref() == Some(id) {
            to_delete.push(f.id);
        }
    }
    let del_set: HashSet<&str> = to_delete.iter().map(|s| s.as_str()).collect();

    // 문서 미분류 처리
    let docs_to_fix: Vec<_> = store
        .documents
        .list()
        .into_iter()
        .filter(|d| d.folder_id.as_deref().map(|fid| del_set.contains(fid)).unwrap_or(false))
        .collect();
    for mut d in docs_to_fix {
        d.folder_id = None;
        d.updated_at = now();
        store.documents.put(d)?;
    }
    // 산출물 미분류 처리
    let dels_to_fix: Vec<_> = store
        .deliverables
        .list()
        .into_iter()
        .filter(|d| d.folder_id.as_deref().map(|fid| del_set.contains(fid)).unwrap_or(false))
        .collect();
    for mut d in dels_to_fix {
        d.folder_id = None;
        d.updated_at = now();
        store.deliverables.put(d)?;
    }
    // 폴더 삭제
    for fid in &to_delete {
        store.folders.remove(fid)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::{new_id, now};
    use crate::store::model::Deliverable;
    use crate::store::ops::business;
    use crate::store::Store;

    fn setup() -> (Store, String) {
        let mut s = Store::open(std::env::temp_dir().join(format!("ops_folder_{}", new_id()))).unwrap();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        (s, b.id)
    }

    #[test]
    fn create_root_and_sub_two_levels() {
        let (mut s, biz) = setup();
        let root = create(&mut s, &biz, "document", None, "보고서").unwrap();
        assert_eq!(root.parent_id, None);
        let sub = create(&mut s, &biz, "document", Some(&root.id), "1차").unwrap();
        assert_eq!(sub.parent_id.as_deref(), Some(root.id.as_str()));
        assert_eq!(list_by_business(&s, &biz).unwrap().len(), 2);
    }

    #[test]
    fn rejects_third_level() {
        let (mut s, biz) = setup();
        let root = create(&mut s, &biz, "deliverable", None, "A").unwrap();
        let sub = create(&mut s, &biz, "deliverable", Some(&root.id), "B").unwrap();
        assert!(create(&mut s, &biz, "deliverable", Some(&sub.id), "C").is_err());
    }

    #[test]
    fn validates_name_kind_and_parent_ownership() {
        let (mut s, biz) = setup();
        assert!(create(&mut s, &biz, "document", None, "  ").is_err());
        assert!(create(&mut s, &biz, "bogus", None, "x").is_err());
        let other = business::create(&mut s, "다른", "ops", None).unwrap();
        let foreign = create(&mut s, &other.id, "document", None, "F").unwrap();
        assert!(create(&mut s, &biz, "document", Some(&foreign.id), "x").is_err());
        let droot = create(&mut s, &biz, "document", None, "문서폴더").unwrap();
        assert!(create(&mut s, &biz, "deliverable", Some(&droot.id), "x").is_err());
    }

    #[test]
    fn ensure_owns_guards_cross_business_and_kind() {
        let (mut s, biz) = setup();
        let docf = create(&mut s, &biz, "document", None, "DF").unwrap();
        assert!(ensure_owns(&s, Some(&docf.id), &biz, "document").is_ok());
        assert!(ensure_owns(&s, Some(&docf.id), &biz, "deliverable").is_err());
        assert!(ensure_owns(&s, None, &biz, "document").is_ok());
    }

    #[test]
    fn delete_uncategorizes_items_and_cascades_children() {
        let (mut s, biz) = setup();
        let root = create(&mut s, &biz, "deliverable", None, "납품").unwrap();
        let sub = create(&mut s, &biz, "deliverable", Some(&root.id), "최종").unwrap();
        // sub 폴더를 가리키는 산출물을 직접 put
        let ts = now();
        let del = Deliverable {
            id: new_id(), business_id: biz.clone(), project_id: None, folder_id: Some(sub.id.clone()),
            title: "a.pdf".into(), kind: "file".into(), document_id: None, file_path: None,
            file_size: Some(10), original_name: Some("a.pdf".into()), status: "draft".into(),
            current_version: 1, versions: vec![], sort_order: 1.0, archived_at: None,
            created_at: ts.clone(), updated_at: ts,
        };
        s.deliverables.put(del.clone()).unwrap();
        // 루트 삭제 → 자식 폴더 cascade, 항목은 미분류로
        delete(&mut s, &root.id).unwrap();
        assert!(get(&s, &sub.id).is_err());
        assert!(get(&s, &root.id).is_err());
        assert_eq!(s.deliverables.get(&del.id).unwrap().folder_id, None);
    }

    #[test]
    fn delete_missing_returns_not_found() {
        let (mut s, _) = setup();
        assert!(matches!(delete(&mut s, "nope"), Err(AppError::NotFound)));
    }
}
