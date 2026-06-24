use crate::error::{AppError, Result};
use crate::store::ids::{new_id, now};
use crate::store::model::Memo;
use crate::store::ops::util::cmp_sort;
use crate::store::Store;

/// 허용 색상 키(팔레트). None/"default"는 기본색(None 저장).
const COLORS: [&str; 9] = [
    "default", "red", "orange", "yellow", "green", "teal", "blue", "purple", "gray",
];

/// 사업의 활성 메모. 고정(pinned) 우선 → sort_order → id.
pub fn list_by_business(store: &Store, business_id: &str) -> Result<Vec<Memo>> {
    let mut out: Vec<Memo> = store
        .memos
        .list()
        .into_iter()
        .filter(|m| m.business_id == business_id && m.archived_at.is_none())
        .collect();
    out.sort_by(|a, b| {
        b.pinned
            .cmp(&a.pinned)
            .then_with(|| cmp_sort(a.sort_order, &a.id, b.sort_order, &b.id))
    });
    Ok(out)
}

pub fn get(store: &Store, id: &str) -> Result<Memo> {
    store.memos.get(id).cloned().ok_or(AppError::NotFound)
}

/// 새 메모는 맨 위: sort_order = 현재 최소값 - 1 (빈 스코프면 -1).
fn next_top(store: &Store, business_id: &str) -> f64 {
    store
        .memos
        .list()
        .iter()
        .filter(|m| m.business_id == business_id)
        .map(|m| m.sort_order)
        .fold(0.0_f64, f64::min)
        - 1.0
}

pub fn create(store: &mut Store, business_id: &str, title: &str, body: &str) -> Result<Memo> {
    let sort_order = next_top(store, business_id);
    let ts = now();
    let m = Memo {
        id: new_id(),
        business_id: business_id.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        color: None,
        pinned: 0,
        sort_order,
        archived_at: None,
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.memos.put(m.clone())?;
    Ok(m)
}

pub fn update(store: &mut Store, id: &str, title: &str, body: &str) -> Result<Memo> {
    let mut m = get(store, id)?;
    m.title = title.to_string();
    m.body = body.to_string();
    m.updated_at = now();
    store.memos.put(m.clone())?;
    Ok(m)
}

/// 색상 변경. None|"default"는 기본색(None 저장). 팔레트 외 색상은 Invalid.
pub fn set_color(store: &mut Store, id: &str, color: Option<&str>) -> Result<Memo> {
    let stored: Option<String> = match color {
        None | Some("default") => None,
        Some(c) if COLORS.contains(&c) => Some(c.to_string()),
        Some(c) => return Err(AppError::Invalid(format!("알 수 없는 색상: {c}"))),
    };
    let mut m = get(store, id)?;
    m.color = stored;
    m.updated_at = now();
    store.memos.put(m.clone())?;
    Ok(m)
}

pub fn set_pinned(store: &mut Store, id: &str, pinned: bool) -> Result<Memo> {
    let mut m = get(store, id)?;
    m.pinned = pinned as i64;
    m.updated_at = now();
    store.memos.put(m.clone())?;
    Ok(m)
}

pub fn archive(store: &mut Store, id: &str) -> Result<()> {
    let mut m = get(store, id)?;
    let ts = now();
    m.archived_at = Some(ts.clone());
    m.updated_at = ts;
    store.memos.put(m)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::ops::business;
    use crate::store::Store;

    fn setup() -> (Store, String) {
        let mut s = Store::open(std::env::temp_dir().join(format!("ops_memo_{}", new_id()))).unwrap();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        (s, b.id)
    }

    #[test]
    fn create_defaults_and_lists() {
        let (mut s, biz) = setup();
        let m = create(&mut s, &biz, "제목", "본문").unwrap();
        assert_eq!(m.title, "제목");
        assert_eq!(m.pinned, 0);
        assert_eq!(m.color, None);
        assert_eq!(list_by_business(&s, &biz).unwrap().len(), 1);
    }

    #[test]
    fn new_memo_goes_on_top() {
        let (mut s, biz) = setup();
        let a = create(&mut s, &biz, "A", "").unwrap();
        let b = create(&mut s, &biz, "B", "").unwrap();
        assert!(b.sort_order < a.sort_order); // 나중에 만든 게 더 작은 sort_order(위)
        assert_eq!(list_by_business(&s, &biz).unwrap()[0].id, b.id);
    }

    #[test]
    fn pinned_sorts_first() {
        let (mut s, biz) = setup();
        create(&mut s, &biz, "A", "").unwrap();
        let b = create(&mut s, &biz, "B", "").unwrap();
        set_pinned(&mut s, &b.id, true).unwrap();
        let list = list_by_business(&s, &biz).unwrap();
        assert_eq!(list[0].id, b.id);
        assert_eq!(list[0].pinned, 1);
    }

    #[test]
    fn set_color_validates_and_default_clears() {
        let (mut s, biz) = setup();
        let m = create(&mut s, &biz, "", "").unwrap();
        assert_eq!(set_color(&mut s, &m.id, Some("blue")).unwrap().color.as_deref(), Some("blue"));
        assert_eq!(set_color(&mut s, &m.id, Some("default")).unwrap().color, None);
        assert_eq!(set_color(&mut s, &m.id, None).unwrap().color, None);
        assert!(set_color(&mut s, &m.id, Some("chartreuse")).is_err());
    }

    #[test]
    fn update_and_archive() {
        let (mut s, biz) = setup();
        let m = create(&mut s, &biz, "old", "oldbody").unwrap();
        let u = update(&mut s, &m.id, "new", "newbody").unwrap();
        assert_eq!(u.title, "new");
        assert_eq!(u.body, "newbody");
        archive(&mut s, &m.id).unwrap();
        assert!(list_by_business(&s, &biz).unwrap().is_empty());
    }
}
