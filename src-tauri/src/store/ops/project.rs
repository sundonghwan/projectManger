use crate::error::{AppError, Result};
use crate::store::ids::{new_id, now};
use crate::store::model::Project;
use crate::store::Store;

/// 특정 사업의 보관되지 않은 프로젝트를 sort_order, id 순으로 조회.
pub fn list_by_business(store: &Store, business_id: &str) -> Result<Vec<Project>> {
    let mut out: Vec<Project> = store
        .projects
        .list()
        .into_iter()
        .filter(|p| p.business_id == business_id && p.archived_at.is_none())
        .collect();
    out.sort_by(|a, b| {
        a.sort_order
            .partial_cmp(&b.sort_order)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(out)
}

pub fn get(store: &Store, id: &str) -> Result<Project> {
    store.projects.get(id).cloned().ok_or(AppError::NotFound)
}

fn next_sort(store: &Store, business_id: &str) -> f64 {
    store
        .projects
        .list()
        .iter()
        .filter(|p| p.business_id == business_id)
        .map(|p| p.sort_order)
        .fold(0.0_f64, f64::max)
        + 1.0
}

/// 새 프로젝트 생성. 소속 사업이 존재하고 보관되지 않아야 한다.
pub fn create(store: &mut Store, business_id: &str, name: &str) -> Result<Project> {
    if name.trim().is_empty() {
        return Err(AppError::Invalid("프로젝트명은 비어 있을 수 없습니다".into()));
    }
    let active = store
        .businesses
        .get(business_id)
        .map(|b| b.archived_at.is_none())
        .unwrap_or(false);
    if !active {
        return Err(AppError::Invalid("존재하지 않는 사업입니다".into()));
    }
    let sort_order = next_sort(store, business_id);
    let ts = now();
    let p = Project {
        id: new_id(),
        business_id: business_id.to_string(),
        name: name.to_string(),
        description: None,
        status: "active".into(),
        start_date: None,
        due_date: None,
        sort_order,
        archived_at: None,
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.projects.put(p.clone())?;
    Ok(p)
}

pub fn update(
    store: &mut Store,
    id: &str,
    name: &str,
    status: &str,
    description: Option<&str>,
    due_date: Option<&str>,
) -> Result<Project> {
    let mut p = get(store, id)?;
    p.name = name.to_string();
    p.status = status.to_string();
    p.description = description.map(|s| s.to_string());
    p.due_date = due_date.map(|s| s.to_string());
    p.updated_at = now();
    store.projects.put(p.clone())?;
    Ok(p)
}

pub fn rename(store: &mut Store, id: &str, name: &str) -> Result<Project> {
    if name.trim().is_empty() {
        return Err(AppError::Invalid("프로젝트명은 비어 있을 수 없습니다".into()));
    }
    let mut p = get(store, id)?;
    p.name = name.to_string();
    p.updated_at = now();
    store.projects.put(p.clone())?;
    Ok(p)
}

pub fn archive(store: &mut Store, id: &str) -> Result<()> {
    let mut p = get(store, id)?;
    let ts = now();
    p.archived_at = Some(ts.clone());
    p.updated_at = ts;
    store.projects.put(p)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::ops::business;
    use crate::store::Store;

    fn setup() -> (Store, String) {
        let mut s = Store::open(std::env::temp_dir().join(format!("ops_proj_{}", new_id()))).unwrap();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        (s, b.id)
    }

    #[test]
    fn create_then_list_by_business() {
        let (mut s, biz) = setup();
        let p = create(&mut s, &biz, "프로젝트 1").unwrap();
        assert_eq!(p.business_id, biz);
        assert_eq!(p.status, "active");
        let list = list_by_business(&s, &biz).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, p.id);
    }

    #[test]
    fn create_rejects_unknown_business() {
        let (mut s, _) = setup();
        assert!(matches!(create(&mut s, "ghost", "x"), Err(AppError::Invalid(_))));
    }

    #[test]
    fn create_rejects_archived_business() {
        let (mut s, biz) = setup();
        business::archive(&mut s, &biz).unwrap();
        assert!(matches!(create(&mut s, &biz, "x"), Err(AppError::Invalid(_))));
    }

    #[test]
    fn create_rejects_empty_name() {
        let (mut s, biz) = setup();
        assert!(create(&mut s, &biz, "   ").is_err());
    }

    #[test]
    fn list_only_returns_own_business() {
        let (mut s, biz1) = setup();
        let biz2 = business::create(&mut s, "사업2", "ops", None).unwrap();
        create(&mut s, &biz1, "P1").unwrap();
        create(&mut s, &biz2.id, "P2").unwrap();
        assert_eq!(list_by_business(&s, &biz1).unwrap().len(), 1);
        assert_eq!(list_by_business(&s, &biz2.id).unwrap().len(), 1);
    }

    #[test]
    fn archive_hides_from_list() {
        let (mut s, biz) = setup();
        let p = create(&mut s, &biz, "P").unwrap();
        archive(&mut s, &p.id).unwrap();
        assert!(list_by_business(&s, &biz).unwrap().is_empty());
    }

    #[test]
    fn update_changes_fields() {
        let (mut s, biz) = setup();
        let p = create(&mut s, &biz, "원래").unwrap();
        let u = update(&mut s, &p.id, "변경", "done", Some("설명"), Some("2026-07-30")).unwrap();
        assert_eq!(u.name, "변경");
        assert_eq!(u.status, "done");
        assert_eq!(u.due_date.as_deref(), Some("2026-07-30"));
    }

    #[test]
    fn rename_changes_name() {
        let (mut s, biz) = setup();
        let p = create(&mut s, &biz, "원래").unwrap();
        assert_eq!(rename(&mut s, &p.id, "새").unwrap().name, "새");
        assert!(rename(&mut s, &p.id, " ").is_err());
    }
}
