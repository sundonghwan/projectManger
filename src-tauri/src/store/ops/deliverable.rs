use crate::error::{AppError, Result};
use crate::store::ids::{new_id, now};
use crate::store::model::{Deliverable, DeliverableVersion};
use crate::store::ops::folder;
use crate::store::ops::util::cmp_sort;
use crate::store::Store;

const STATUSES: [&str; 3] = ["draft", "review", "done"];
const KINDS: [&str; 2] = ["file", "document"];

pub fn list_by_business(store: &Store, business_id: &str) -> Result<Vec<Deliverable>> {
    let mut out: Vec<Deliverable> = store
        .deliverables
        .list()
        .into_iter()
        .filter(|d| d.business_id == business_id && d.archived_at.is_none())
        .collect();
    out.sort_by(|a, b| cmp_sort(a.sort_order, &a.id, b.sort_order, &b.id));
    Ok(out)
}

pub fn get(store: &Store, id: &str) -> Result<Deliverable> {
    store.deliverables.get(id).cloned().ok_or(AppError::NotFound)
}

fn check_project(store: &Store, project_id: Option<&str>, business_id: &str) -> Result<()> {
    if let Some(pid) = project_id {
        let ok = store
            .projects
            .get(pid)
            .map(|p| p.business_id == business_id)
            .unwrap_or(false);
        if !ok {
            return Err(AppError::Invalid("프로젝트가 해당 사업 소속이 아닙니다".into()));
        }
    }
    Ok(())
}

fn next_sort(store: &Store, business_id: &str) -> f64 {
    store
        .deliverables
        .list()
        .iter()
        .filter(|d| d.business_id == business_id)
        .map(|d| d.sort_order)
        .fold(0.0_f64, f64::max)
        + 1.0
}

pub fn create(
    store: &mut Store,
    business_id: &str,
    project_id: Option<&str>,
    folder_id: Option<&str>,
    title: &str,
    kind: &str,
) -> Result<Deliverable> {
    if title.trim().is_empty() {
        return Err(AppError::Invalid("산출물명은 비어 있을 수 없습니다".into()));
    }
    if !KINDS.contains(&kind) {
        return Err(AppError::Invalid(format!("알 수 없는 종류: {kind}")));
    }
    check_project(store, project_id, business_id)?;
    folder::ensure_owns(store, folder_id, business_id, "deliverable")?;
    let sort_order = next_sort(store, business_id);
    let ts = now();
    let d = Deliverable {
        id: new_id(),
        business_id: business_id.to_string(),
        project_id: project_id.map(|s| s.to_string()),
        folder_id: folder_id.map(|s| s.to_string()),
        title: title.to_string(),
        kind: kind.to_string(),
        document_id: None,
        file_path: None,
        file_size: None,
        original_name: None,
        status: "draft".into(),
        current_version: 1,
        versions: vec![DeliverableVersion {
            id: new_id(),
            version: 1,
            file_path: None,
            note: Some("최초 생성".into()),
            created_at: ts.clone(),
        }],
        sort_order,
        archived_at: None,
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.deliverables.put(d.clone())?;
    Ok(d)
}

pub fn create_file(
    store: &mut Store,
    business_id: &str,
    project_id: Option<&str>,
    folder_id: Option<&str>,
    title: &str,
    original_name: &str,
    file_size: i64,
) -> Result<Deliverable> {
    if title.trim().is_empty() {
        return Err(AppError::Invalid("산출물명은 비어 있을 수 없습니다".into()));
    }
    check_project(store, project_id, business_id)?;
    folder::ensure_owns(store, folder_id, business_id, "deliverable")?;
    let sort_order = next_sort(store, business_id);
    let ts = now();
    let d = Deliverable {
        id: new_id(),
        business_id: business_id.to_string(),
        project_id: project_id.map(|s| s.to_string()),
        folder_id: folder_id.map(|s| s.to_string()),
        title: title.to_string(),
        kind: "file".into(),
        document_id: None,
        file_path: None,
        file_size: Some(file_size),
        original_name: Some(original_name.to_string()),
        status: "draft".into(),
        current_version: 1,
        versions: Vec::new(),
        sort_order,
        archived_at: None,
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.deliverables.put(d.clone())?;
    Ok(d)
}

pub fn set_folder(store: &mut Store, id: &str, folder_id: Option<&str>) -> Result<Deliverable> {
    let mut d = get(store, id)?;
    folder::ensure_owns(store, folder_id, &d.business_id, "deliverable")?;
    d.folder_id = folder_id.map(|s| s.to_string());
    d.updated_at = now();
    store.deliverables.put(d.clone())?;
    Ok(d)
}

pub fn set_file_path(store: &mut Store, id: &str, file_path: &str) -> Result<()> {
    let mut d = get(store, id)?;
    d.file_path = Some(file_path.to_string());
    d.updated_at = now();
    store.deliverables.put(d)?;
    Ok(())
}

pub fn rename(store: &mut Store, id: &str, title: &str) -> Result<Deliverable> {
    if title.trim().is_empty() {
        return Err(AppError::Invalid("산출물명은 비어 있을 수 없습니다".into()));
    }
    let mut d = get(store, id)?;
    d.title = title.trim().to_string();
    d.updated_at = now();
    store.deliverables.put(d.clone())?;
    Ok(d)
}

pub fn file_path_of(store: &Store, id: &str) -> Result<Option<String>> {
    let d = get(store, id)?;
    Ok(d.file_path)
}

pub fn delete(store: &mut Store, id: &str) -> Result<()> {
    if store.deliverables.get(id).is_none() {
        return Err(AppError::NotFound);
    }
    store.deliverables.remove(id)?;
    Ok(())
}

pub fn update_status(store: &mut Store, id: &str, status: &str) -> Result<Deliverable> {
    if !STATUSES.contains(&status) {
        return Err(AppError::Invalid(format!("알 수 없는 상태: {status}")));
    }
    let mut d = get(store, id)?;
    d.status = status.to_string();
    d.updated_at = now();
    store.deliverables.put(d.clone())?;
    Ok(d)
}

/// 새 버전 기록 (current_version + 1). 인라인 versions 에 push.
pub fn add_version(
    store: &mut Store,
    id: &str,
    note: Option<&str>,
    file_path: Option<&str>,
) -> Result<Deliverable> {
    let mut d = get(store, id)?;
    let next = d.current_version + 1;
    let ts = now();
    d.versions.push(DeliverableVersion {
        id: new_id(),
        version: next,
        file_path: file_path.map(|s| s.to_string()),
        note: note.map(|s| s.to_string()),
        created_at: ts.clone(),
    });
    d.current_version = next;
    d.updated_at = ts;
    store.deliverables.put(d.clone())?;
    Ok(d)
}

pub fn list_versions(store: &Store, deliverable_id: &str) -> Result<Vec<DeliverableVersion>> {
    let d = get(store, deliverable_id)?;
    let mut vs = d.versions.clone();
    vs.sort_by(|a, b| b.version.cmp(&a.version)); // 최신(높은 version) 먼저
    Ok(vs)
}

pub fn archive(store: &mut Store, id: &str) -> Result<()> {
    let mut d = get(store, id)?;
    let ts = now();
    d.archived_at = Some(ts.clone());
    d.updated_at = ts;
    store.deliverables.put(d)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::ops::{business, project};
    use crate::store::Store;

    fn setup() -> (Store, String, String) {
        let mut s = Store::open(std::env::temp_dir().join(format!("ops_deliv_{}", new_id()))).unwrap();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let p = project::create(&mut s, &b.id, "프로젝트").unwrap();
        (s, b.id, p.id)
    }

    #[test]
    fn create_defaults_and_initial_version() {
        let (mut s, biz, _) = setup();
        let d = create(&mut s, &biz, None, None, "제안서", "document").unwrap();
        assert_eq!(d.status, "draft");
        assert_eq!(d.current_version, 1);
        let versions = list_versions(&s, &d.id).unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version, 1);
    }

    #[test]
    fn create_validates_kind_title_project() {
        let (mut s, biz, _) = setup();
        assert!(create(&mut s, &biz, None, None, " ", "file").is_err());
        assert!(create(&mut s, &biz, None, None, "x", "bogus").is_err());
        let other = business::create(&mut s, "다른", "ops", None).unwrap();
        let op = project::create(&mut s, &other.id, "P").unwrap();
        assert!(create(&mut s, &biz, Some(&op.id), None, "x", "file").is_err());
    }

    #[test]
    fn create_file_sets_fields_without_version() {
        let (mut s, biz, _) = setup();
        let d = create_file(&mut s, &biz, None, None, "보고서.pdf", "보고서.pdf", 2048).unwrap();
        assert_eq!(d.kind, "file");
        assert_eq!(d.status, "draft");
        assert_eq!(d.original_name.as_deref(), Some("보고서.pdf"));
        assert_eq!(d.file_size, Some(2048));
        assert!(d.file_path.is_none());
        assert!(list_versions(&s, &d.id).unwrap().is_empty());
        set_file_path(&mut s, &d.id, "/tmp/x/보고서.pdf").unwrap();
        assert_eq!(get(&s, &d.id).unwrap().file_path.as_deref(), Some("/tmp/x/보고서.pdf"));
        assert_eq!(file_path_of(&s, &d.id).unwrap().as_deref(), Some("/tmp/x/보고서.pdf"));
    }

    #[test]
    fn add_version_increments_and_records() {
        let (mut s, biz, _) = setup();
        let d = create(&mut s, &biz, None, None, "산출물", "file").unwrap();
        let d2 = add_version(&mut s, &d.id, Some("피드백 반영"), None).unwrap();
        assert_eq!(d2.current_version, 2);
        let versions = list_versions(&s, &d.id).unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, 2); // 최신 먼저
        assert_eq!(versions[0].note.as_deref(), Some("피드백 반영"));
    }

    #[test]
    fn rename_changes_title_only() {
        let (mut s, biz, _) = setup();
        let d = create_file(&mut s, &biz, None, None, "a.txt", "a.txt", 1).unwrap();
        let r = rename(&mut s, &d.id, "최종본").unwrap();
        assert_eq!(r.title, "최종본");
        assert_eq!(r.original_name.as_deref(), Some("a.txt"));
        assert!(rename(&mut s, &d.id, "  ").is_err());
    }

    #[test]
    fn update_status_validates_and_archive_delete() {
        let (mut s, biz, _) = setup();
        let d = create(&mut s, &biz, None, None, "x", "file").unwrap();
        assert_eq!(update_status(&mut s, &d.id, "review").unwrap().status, "review");
        assert!(update_status(&mut s, &d.id, "bogus").is_err());
        archive(&mut s, &d.id).unwrap();
        assert!(list_by_business(&s, &biz).unwrap().is_empty());
        let d2 = create(&mut s, &biz, None, None, "y", "file").unwrap();
        delete(&mut s, &d2.id).unwrap();
        assert!(get(&s, &d2.id).is_err());
    }
}
