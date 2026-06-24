use crate::error::{AppError, Result};
use crate::store::ids::{new_id, now};
use crate::store::model::Task;
use crate::store::Store;
use crate::store::ops::util::cmp_sort;

const STATUSES: [&str; 4] = ["todo", "doing", "review", "done"];

/// 사업(선택적으로 프로젝트)의 보관되지 않은 태스크를 status, sort_order, id 순으로 조회.
/// project_id=None 이면 사업 전체(직속+프로젝트 소속).
pub fn list(store: &Store, business_id: &str, project_id: Option<&str>) -> Result<Vec<Task>> {
    let mut out: Vec<Task> = store
        .tasks
        .list()
        .into_iter()
        .filter(|t| {
            t.business_id == business_id
                && t.archived_at.is_none()
                && match project_id {
                    Some(pid) => t.project_id.as_deref() == Some(pid),
                    None => true,
                }
        })
        .collect();
    out.sort_by(|a, b| {
        a.status
            .cmp(&b.status)
            .then_with(|| cmp_sort(a.sort_order, &a.id, b.sort_order, &b.id))
    });
    Ok(out)
}

pub fn get(store: &Store, id: &str) -> Result<Task> {
    store.tasks.get(id).cloned().ok_or(AppError::NotFound)
}

/// 신규 태스크 정렬값 = 같은 사업의 status='todo' 태스크 중 최대 sort_order + 1.
fn next_sort(store: &Store, business_id: &str) -> f64 {
    store
        .tasks
        .list()
        .iter()
        .filter(|t| t.business_id == business_id && t.status == "todo")
        .map(|t| t.sort_order)
        .fold(0.0_f64, f64::max)
        + 1.0
}

/// 새 태스크 생성. business 필수, project_id 가 있으면 그 프로젝트가 해당 사업 소속이어야 함.
pub fn create(
    store: &mut Store,
    business_id: &str,
    project_id: Option<&str>,
    title: &str,
    priority: i64,
) -> Result<Task> {
    if title.trim().is_empty() {
        return Err(AppError::Invalid("태스크 제목은 비어 있을 수 없습니다".into()));
    }
    if !(0..=4).contains(&priority) {
        return Err(AppError::Invalid("우선순위는 0~4".into()));
    }
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
    let sort_order = next_sort(store, business_id);
    let ts = now();
    let t = Task {
        id: new_id(),
        business_id: business_id.to_string(),
        project_id: project_id.map(|s| s.to_string()),
        parent_task_id: None,
        title: title.to_string(),
        description: None,
        status: "todo".into(),
        priority,
        due_date: None,
        sort_order,
        completed_at: None,
        archived_at: None,
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.tasks.put(t.clone())?;
    Ok(t)
}

pub fn update(
    store: &mut Store,
    id: &str,
    title: &str,
    priority: i64,
    due_date: Option<&str>,
    description: Option<&str>,
) -> Result<Task> {
    let mut t = get(store, id)?;
    t.title = title.to_string();
    t.priority = priority;
    t.due_date = due_date.map(|s| s.to_string());
    t.description = description.map(|s| s.to_string());
    t.updated_at = now();
    store.tasks.put(t.clone())?;
    Ok(t)
}

/// 칸반 드래그: 상태/정렬 변경. status='done' 이면 completed_at 설정, 아니면 해제.
pub fn move_task(store: &mut Store, id: &str, status: &str, sort_order: f64) -> Result<Task> {
    if !STATUSES.contains(&status) {
        return Err(AppError::Invalid(format!("알 수 없는 상태: {status}")));
    }
    let mut t = get(store, id)?;
    let ts = now();
    t.status = status.to_string();
    t.sort_order = sort_order;
    t.completed_at = if status == "done" { Some(ts.clone()) } else { None };
    t.updated_at = ts;
    store.tasks.put(t.clone())?;
    Ok(t)
}

pub fn archive(store: &mut Store, id: &str) -> Result<()> {
    let mut t = get(store, id)?;
    let ts = now();
    t.archived_at = Some(ts.clone());
    t.updated_at = ts;
    store.tasks.put(t)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::ops::{business, project};
    use crate::store::Store;

    fn setup() -> (Store, String, String) {
        let mut s = Store::open(std::env::temp_dir().join(format!("ops_task_{}", new_id()))).unwrap();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let p = project::create(&mut s, &b.id, "프로젝트").unwrap();
        (s, b.id, p.id)
    }

    #[test]
    fn create_defaults_to_todo() {
        let (mut s, biz, _) = setup();
        let t = create(&mut s, &biz, None, "로그인 API", 3).unwrap();
        assert_eq!(t.status, "todo");
        assert_eq!(t.priority, 3);
        assert!(t.completed_at.is_none());
    }

    #[test]
    fn create_rejects_empty_title_and_bad_priority() {
        let (mut s, biz, _) = setup();
        assert!(create(&mut s, &biz, None, "  ", 2).is_err());
        assert!(create(&mut s, &biz, None, "x", 9).is_err());
    }

    #[test]
    fn create_rejects_project_of_other_business() {
        let (mut s, biz, _) = setup();
        let other = business::create(&mut s, "다른사업", "ops", None).unwrap();
        let other_proj = project::create(&mut s, &other.id, "P").unwrap();
        assert!(create(&mut s, &biz, Some(&other_proj.id), "x", 2).is_err());
    }

    #[test]
    fn list_filters_by_project() {
        let (mut s, biz, proj) = setup();
        create(&mut s, &biz, Some(&proj), "프로젝트 소속", 2).unwrap();
        create(&mut s, &biz, None, "사업 직속", 2).unwrap();
        assert_eq!(list(&s, &biz, None).unwrap().len(), 2);
        assert_eq!(list(&s, &biz, Some(&proj)).unwrap().len(), 1);
    }

    #[test]
    fn move_to_done_sets_completed_at() {
        let (mut s, biz, _) = setup();
        let t = create(&mut s, &biz, None, "x", 2).unwrap();
        let moved = move_task(&mut s, &t.id, "done", 5.0).unwrap();
        assert_eq!(moved.status, "done");
        assert!(moved.completed_at.is_some());
        assert_eq!(moved.sort_order, 5.0);
        let back = move_task(&mut s, &t.id, "doing", 2.0).unwrap();
        assert!(back.completed_at.is_none());
    }

    #[test]
    fn move_rejects_unknown_status() {
        let (mut s, biz, _) = setup();
        let t = create(&mut s, &biz, None, "x", 2).unwrap();
        assert!(matches!(move_task(&mut s, &t.id, "bogus", 1.0), Err(AppError::Invalid(_))));
    }

    #[test]
    fn archive_hides_from_list() {
        let (mut s, biz, _) = setup();
        let t = create(&mut s, &biz, None, "x", 2).unwrap();
        archive(&mut s, &t.id).unwrap();
        assert!(list(&s, &biz, None).unwrap().is_empty());
    }

    #[test]
    fn update_changes_fields() {
        let (mut s, biz, _) = setup();
        let t = create(&mut s, &biz, None, "원래", 2).unwrap();
        let u = update(&mut s, &t.id, "변경", 4, Some("2026-07-01"), Some("메모")).unwrap();
        assert_eq!(u.title, "변경");
        assert_eq!(u.priority, 4);
        assert_eq!(u.due_date.as_deref(), Some("2026-07-01"));
    }
}
