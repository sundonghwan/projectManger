use crate::error::{AppError, Result};
use crate::store::ids::{new_id, now};
use crate::store::model::{Label, TaskLabel};
use crate::store::Store;
use serde::Serialize;

/// map_for_business 가 반환하는 denormalized 뷰(프론트 taskId→labels 맵 구성용).
/// 저장 엔티티 TaskLabel(관계)과 다른, 조회 전용 구조.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLabelView {
    pub task_id: String,
    pub label_id: String,
    pub name: String,
    pub color: Option<String>,
}

/// 라벨 전체를 name, id 순으로.
pub fn list(store: &Store) -> Result<Vec<Label>> {
    let mut out = store.labels.list();
    out.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
    Ok(out)
}

/// 라벨 생성. 같은 이름이 있으면 기존 것 반환(멱등).
pub fn create(store: &mut Store, name: &str, color: Option<&str>) -> Result<Label> {
    if name.trim().is_empty() {
        return Err(AppError::Invalid("라벨명은 비어 있을 수 없습니다".into()));
    }
    if let Some(existing) = store.labels.list().into_iter().find(|l| l.name == name) {
        return Ok(existing);
    }
    let ts = now();
    let l = Label {
        id: new_id(),
        name: name.to_string(),
        color: color.map(|s| s.to_string()),
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.labels.put(l.clone())?;
    Ok(l)
}

/// 태스크에 라벨 부여(멱등). 이미 (task,label) 관계가 있으면 no-op.
pub fn assign(store: &mut Store, task_id: &str, label_id: &str) -> Result<()> {
    let exists = store
        .task_labels
        .list()
        .iter()
        .any(|tl| tl.task_id == task_id && tl.label_id == label_id);
    if exists {
        return Ok(());
    }
    let ts = now();
    store.task_labels.put(TaskLabel {
        id: new_id(),
        task_id: task_id.to_string(),
        label_id: label_id.to_string(),
        created_at: ts.clone(),
        updated_at: ts,
    })?;
    Ok(())
}

/// 부여 해제. 매칭되는 관계 엔티티를 모두 제거(없으면 no-op).
pub fn unassign(store: &mut Store, task_id: &str, label_id: &str) -> Result<()> {
    let ids: Vec<String> = store
        .task_labels
        .list()
        .into_iter()
        .filter(|tl| tl.task_id == task_id && tl.label_id == label_id)
        .map(|tl| tl.id)
        .collect();
    for id in ids {
        store.task_labels.remove(&id)?;
    }
    Ok(())
}

/// 한 사업의 모든 태스크-라벨 매핑을 뷰로(해당 사업 소속 task + 존재하는 label 만).
pub fn map_for_business(store: &Store, business_id: &str) -> Result<Vec<TaskLabelView>> {
    let mut out = Vec::new();
    for tl in store.task_labels.list() {
        let Some(task) = store.tasks.get(&tl.task_id) else { continue };
        if task.business_id != business_id {
            continue;
        }
        let Some(label) = store.labels.get(&tl.label_id) else { continue };
        out.push(TaskLabelView {
            task_id: tl.task_id,
            label_id: tl.label_id,
            name: label.name.clone(),
            color: label.color.clone(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::ops::{business, task};
    use crate::store::Store;

    fn setup() -> (Store, String, String) {
        let mut s = Store::open(std::env::temp_dir().join(format!("ops_label_{}", new_id()))).unwrap();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let t = task::create(&mut s, &b.id, None, "태스크", 2).unwrap();
        (s, b.id, t.id)
    }

    #[test]
    fn create_is_idempotent_by_name() {
        let (mut s, _, _) = setup();
        let a = create(&mut s, "백엔드", Some("#3b82f6")).unwrap();
        let b = create(&mut s, "백엔드", None).unwrap();
        assert_eq!(a.id, b.id);
        assert_eq!(list(&s).unwrap().len(), 1);
    }

    #[test]
    fn create_rejects_empty() {
        let (mut s, _, _) = setup();
        assert!(create(&mut s, "  ", None).is_err());
    }

    #[test]
    fn assign_idempotent_and_map() {
        let (mut s, biz, task_id) = setup();
        let l1 = create(&mut s, "백엔드", Some("#3b82f6")).unwrap();
        let l2 = create(&mut s, "긴급", Some("#ef4444")).unwrap();
        assign(&mut s, &task_id, &l1.id).unwrap();
        assign(&mut s, &task_id, &l2.id).unwrap();
        assign(&mut s, &task_id, &l1.id).unwrap(); // 중복 무시
        let map = map_for_business(&s, &biz).unwrap();
        assert_eq!(map.len(), 2);
        assert!(map.iter().all(|m| m.task_id == task_id));
        assert!(map.iter().any(|m| m.name == "백엔드"));
    }

    #[test]
    fn unassign_removes() {
        let (mut s, biz, task_id) = setup();
        let l = create(&mut s, "백엔드", None).unwrap();
        assign(&mut s, &task_id, &l.id).unwrap();
        unassign(&mut s, &task_id, &l.id).unwrap();
        assert!(map_for_business(&s, &biz).unwrap().is_empty());
    }

    #[test]
    fn map_excludes_other_business_and_missing() {
        let (mut s, biz, task_id) = setup();
        let l = create(&mut s, "x", None).unwrap();
        assign(&mut s, &task_id, &l.id).unwrap();
        let other = business::create(&mut s, "다른", "ops", None).unwrap();
        assert!(map_for_business(&s, &other.id).unwrap().is_empty());
        assert_eq!(map_for_business(&s, &biz).unwrap().len(), 1);
    }

    #[test]
    fn view_serializes_camelcase() {
        let v = TaskLabelView { task_id: "t".into(), label_id: "l".into(), name: "n".into(), color: None };
        let j = serde_json::to_value(&v).unwrap();
        assert_eq!(j["taskId"], "t");
        assert_eq!(j["labelId"], "l");
    }
}
