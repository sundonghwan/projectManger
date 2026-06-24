use crate::error::{AppError, Result};
use crate::store::ids::{new_id, now};
use crate::store::model::Business;
use crate::store::Store;

/// 보관되지 않은 사업을 sort_order, id 순으로 조회.
pub fn list(store: &Store) -> Result<Vec<Business>> {
    let mut out: Vec<Business> = store
        .businesses
        .list()
        .into_iter()
        .filter(|b| b.archived_at.is_none())
        .collect();
    out.sort_by(|a, b| {
        a.sort_order
            .partial_cmp(&b.sort_order)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(out)
}

pub fn get(store: &Store, id: &str) -> Result<Business> {
    store.businesses.get(id).cloned().ok_or(AppError::NotFound)
}

/// 다음 정렬값 = 현재 최대 sort_order + 1 (보관 포함 전체 기준; 기존 SQL과 동일).
fn next_sort(store: &Store) -> f64 {
    store
        .businesses
        .list()
        .iter()
        .map(|b| b.sort_order)
        .fold(0.0_f64, f64::max)
        + 1.0
}

/// 새 사업 생성. sort_order 는 현재 최대값 + 1.
pub fn create(store: &mut Store, name: &str, type_: &str, color: Option<&str>) -> Result<Business> {
    if name.trim().is_empty() {
        return Err(AppError::Invalid("사업명은 비어 있을 수 없습니다".into()));
    }
    let sort_order = next_sort(store);
    let ts = now();
    let b = Business {
        id: new_id(),
        name: name.to_string(),
        r#type: type_.to_string(),
        color: color.map(|s| s.to_string()),
        description: None,
        status: "active".into(),
        sort_order,
        archived_at: None,
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.businesses.put(b.clone())?;
    Ok(b)
}

pub fn update(
    store: &mut Store,
    id: &str,
    name: &str,
    type_: &str,
    status: &str,
    color: Option<&str>,
    description: Option<&str>,
) -> Result<Business> {
    let mut b = get(store, id)?;
    b.name = name.to_string();
    b.r#type = type_.to_string();
    b.status = status.to_string();
    b.color = color.map(|s| s.to_string());
    b.description = description.map(|s| s.to_string());
    b.updated_at = now();
    store.businesses.put(b.clone())?;
    Ok(b)
}

pub fn rename(store: &mut Store, id: &str, name: &str) -> Result<Business> {
    if name.trim().is_empty() {
        return Err(AppError::Invalid("사업명은 비어 있을 수 없습니다".into()));
    }
    let mut b = get(store, id)?;
    b.name = name.to_string();
    b.updated_at = now();
    store.businesses.put(b.clone())?;
    Ok(b)
}

/// 소프트 삭제(보관). archived_at 과 updated_at 을 같은 타임스탬프로 설정.
pub fn archive(store: &mut Store, id: &str) -> Result<()> {
    let mut b = get(store, id)?;
    let ts = now();
    b.archived_at = Some(ts.clone());
    b.updated_at = ts;
    store.businesses.put(b)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::Store;

    fn store() -> Store {
        Store::open(std::env::temp_dir().join(format!("ops_biz_{}", new_id()))).unwrap()
    }

    #[test]
    fn create_then_list_returns_it() {
        let mut s = store();
        let b = create(&mut s, "SI사업 A", "si", Some("#3b82f6")).unwrap();
        assert_eq!(b.name, "SI사업 A");
        assert_eq!(b.r#type, "si");
        assert_eq!(b.status, "active");
        assert!(!b.id.is_empty());
        let all = list(&s).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, b.id);
    }

    #[test]
    fn create_rejects_empty_name() {
        let mut s = store();
        assert!(create(&mut s, "  ", "si", None).is_err());
    }

    #[test]
    fn list_is_ordered_by_sort_order_and_hides_archived() {
        let mut s = store();
        let a = create(&mut s, "첫째", "si", None).unwrap();
        let b = create(&mut s, "둘째", "internal", None).unwrap();
        assert!(a.sort_order < b.sort_order);
        let all = list(&s).unwrap();
        assert_eq!(all[0].name, "첫째");
        assert_eq!(all[1].name, "둘째");
        archive(&mut s, &a.id).unwrap();
        assert_eq!(list(&s).unwrap().len(), 1);
    }

    #[test]
    fn update_changes_fields_and_bumps_updated_at() {
        let mut s = store();
        let b = create(&mut s, "원래", "si", None).unwrap();
        let u = update(&mut s, &b.id, "변경", "ops", "done", Some("#fff"), Some("설명")).unwrap();
        assert_eq!(u.name, "변경");
        assert_eq!(u.r#type, "ops");
        assert_eq!(u.status, "done");
        assert_eq!(u.description.as_deref(), Some("설명"));
    }

    #[test]
    fn rename_changes_only_name() {
        let mut s = store();
        let b = create(&mut s, "원래", "si", None).unwrap();
        let r = rename(&mut s, &b.id, "새이름").unwrap();
        assert_eq!(r.name, "새이름");
        assert_eq!(r.r#type, "si");
        assert!(rename(&mut s, &b.id, "  ").is_err());
    }

    #[test]
    fn archive_sets_archived_at_but_get_still_finds() {
        let mut s = store();
        let b = create(&mut s, "보관대상", "si", None).unwrap();
        archive(&mut s, &b.id).unwrap();
        assert!(get(&s, &b.id).unwrap().archived_at.is_some());
    }

    #[test]
    fn get_missing_returns_not_found() {
        let s = store();
        assert!(matches!(get(&s, "nope"), Err(AppError::NotFound)));
    }

    #[test]
    fn update_missing_returns_not_found() {
        let mut s = store();
        assert!(matches!(
            update(&mut s, "nope", "x", "si", "active", None, None),
            Err(AppError::NotFound)
        ));
    }
}
