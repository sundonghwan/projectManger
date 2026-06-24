use crate::error::{AppError, Result};
use crate::store::ids::{new_id, now};
use crate::store::model::RecurringTask;
use crate::store::ops::task;
use crate::store::Store;
use chrono::{Duration, NaiveDate};

pub fn list_by_business(store: &Store, business_id: &str) -> Result<Vec<RecurringTask>> {
    let mut out: Vec<RecurringTask> = store
        .recurring
        .list()
        .into_iter()
        .filter(|r| r.business_id == business_id)
        .collect();
    out.sort_by(|a, b| a.next_run.cmp(&b.next_run).then_with(|| a.id.cmp(&b.id)));
    Ok(out)
}

pub fn create(
    store: &mut Store,
    business_id: &str,
    project_id: Option<&str>,
    title: &str,
    priority: i64,
    interval_days: i64,
    next_run: &str,
) -> Result<RecurringTask> {
    if title.trim().is_empty() {
        return Err(AppError::Invalid("제목은 필수입니다".into()));
    }
    if interval_days < 1 {
        return Err(AppError::Invalid("반복 간격은 1일 이상이어야 합니다".into()));
    }
    let ts = now();
    let r = RecurringTask {
        id: new_id(),
        business_id: business_id.to_string(),
        project_id: project_id.map(|s| s.to_string()),
        title: title.to_string(),
        priority,
        interval_days,
        next_run: next_run.to_string(),
        active: 1,
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.recurring.put(r.clone())?;
    Ok(r)
}

pub fn set_active(store: &mut Store, id: &str, active: bool) -> Result<()> {
    let mut r = store.recurring.get(id).cloned().ok_or(AppError::NotFound)?;
    r.active = active as i64;
    r.updated_at = now();
    store.recurring.put(r)?;
    Ok(())
}

pub fn delete(store: &mut Store, id: &str) -> Result<()> {
    if store.recurring.get(id).is_none() {
        return Err(AppError::NotFound);
    }
    store.recurring.remove(id)?;
    Ok(())
}

/// today(YYYY-MM-DD) 기준 도래한 활성 반복 항목마다 태스크를 생성하고
/// next_run 을 interval_days 만큼 전진. 생성한 태스크 수 반환.
pub fn generate_due(store: &mut Store, today: &str) -> Result<usize> {
    let mut due: Vec<RecurringTask> = store
        .recurring
        .list()
        .into_iter()
        .filter(|r| r.active == 1 && r.next_run.as_str() <= today)
        .collect();
    due.sort_by(|a, b| a.id.cmp(&b.id));

    let mut created = 0usize;
    for r in &due {
        task::create(store, &r.business_id, r.project_id.as_deref(), &r.title, r.priority)?;
        let date = NaiveDate::parse_from_str(&r.next_run, "%Y-%m-%d")
            .map_err(|e| AppError::Invalid(format!("날짜 형식 오류: {e}")))?;
        let new_next = date
            .checked_add_signed(Duration::days(r.interval_days))
            .ok_or_else(|| AppError::Invalid("날짜 계산 오버플로우".into()))?
            .format("%Y-%m-%d")
            .to_string();
        let mut updated = r.clone();
        updated.next_run = new_next;
        updated.updated_at = now();
        store.recurring.put(updated)?;
        created += 1;
    }
    Ok(created)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::ops::{business, task};
    use crate::store::Store;

    fn setup() -> (Store, String) {
        let mut s = Store::open(std::env::temp_dir().join(format!("ops_recur_{}", new_id()))).unwrap();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        (s, b.id)
    }

    #[test]
    fn create_validates() {
        let (mut s, b) = setup();
        assert!(create(&mut s, &b, None, "", 2, 7, "2026-07-01").is_err());
        assert!(create(&mut s, &b, None, "주간보고", 2, 0, "2026-07-01").is_err());
        assert!(create(&mut s, &b, None, "주간보고", 2, 7, "2026-07-01").is_ok());
    }

    #[test]
    fn generate_due_creates_task_and_advances() {
        let (mut s, b) = setup();
        create(&mut s, &b, None, "주간보고", 2, 7, "2026-06-20").unwrap();
        create(&mut s, &b, None, "미래", 2, 7, "2026-12-31").unwrap();
        let made = generate_due(&mut s, "2026-06-20").unwrap();
        assert_eq!(made, 1);
        let tasks = task::list(&s, &b, None).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "주간보고");
        let items = list_by_business(&s, &b).unwrap();
        let weekly = items.iter().find(|i| i.title == "주간보고").unwrap();
        assert_eq!(weekly.next_run, "2026-06-27");
    }

    #[test]
    fn inactive_not_generated() {
        let (mut s, b) = setup();
        let r = create(&mut s, &b, None, "보고", 2, 1, "2026-06-20").unwrap();
        set_active(&mut s, &r.id, false).unwrap();
        assert_eq!(generate_due(&mut s, "2026-06-20").unwrap(), 0);
    }

    #[test]
    fn delete_works() {
        let (mut s, b) = setup();
        let r = create(&mut s, &b, None, "x", 2, 1, "2026-06-20").unwrap();
        delete(&mut s, &r.id).unwrap();
        assert!(list_by_business(&s, &b).unwrap().is_empty());
        assert!(matches!(delete(&mut s, "nope"), Err(AppError::NotFound)));
    }
}
