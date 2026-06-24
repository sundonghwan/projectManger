use crate::error::{AppError, Result};
use crate::models::Memo;
use rusqlite::{params, Connection, Row};

/// 허용 색상 키(팔레트). NULL/None 은 기본색.
const COLORS: [&str; 9] = [
    "default", "red", "orange", "yellow", "green", "teal", "blue", "purple", "gray",
];

fn map_memo(row: &Row) -> rusqlite::Result<Memo> {
    Ok(Memo {
        id: row.get("id")?,
        business_id: row.get("business_id")?,
        title: row.get("title")?,
        body: row.get("body")?,
        color: row.get("color")?,
        pinned: row.get("pinned")?,
        sort_order: row.get("sort_order")?,
        archived_at: row.get("archived_at")?,
        created_at: row.get("created_at")?,
    })
}

/// 사업의 활성 메모. 고정 우선 → sort_order → id.
pub fn list_by_business(conn: &Connection, business_id: i64) -> Result<Vec<Memo>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM memo WHERE business_id=?1 AND archived_at IS NULL \
         ORDER BY pinned DESC, sort_order, id",
    )?;
    let rows = stmt.query_map(params![business_id], map_memo)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: i64) -> Result<Memo> {
    conn.query_row("SELECT * FROM memo WHERE id=?1", params![id], map_memo)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Db(other),
        })
}

/// 메모 생성. 제목·본문 둘 다 비어도 허용(빈 메모 폐기는 프론트 책임). 새 메모는 맨 위(sort_order 최소-1).
pub fn create(conn: &Connection, business_id: i64, title: &str, body: &str) -> Result<Memo> {
    let next: f64 = conn.query_row(
        "SELECT COALESCE(MIN(sort_order),0)-1 FROM memo WHERE business_id=?1",
        params![business_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO memo (business_id, title, body, sort_order) VALUES (?1,?2,?3,?4)",
        params![business_id, title, body, next],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, title: &str, body: &str) -> Result<Memo> {
    let n = conn.execute(
        "UPDATE memo SET title=?2, body=?3, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, title, body],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

/// 색상 변경. color=None 이면 기본색으로 초기화. 'default' 도 기본색(NULL 저장).
pub fn set_color(conn: &Connection, id: i64, color: Option<&str>) -> Result<Memo> {
    let stored: Option<&str> = match color {
        None | Some("default") => None,
        Some(c) if COLORS.contains(&c) => Some(c),
        Some(c) => return Err(AppError::Invalid(format!("알 수 없는 색상: {c}"))),
    };
    let n = conn.execute(
        "UPDATE memo SET color=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, stored],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

pub fn set_pinned(conn: &Connection, id: i64, pinned: bool) -> Result<Memo> {
    let n = conn.execute(
        "UPDATE memo SET pinned=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, pinned as i64],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

pub fn archive(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute(
        "UPDATE memo SET archived_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, repo::business};

    fn setup() -> (Connection, i64) {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "사업", "si", None).unwrap();
        (c, b.id)
    }

    #[test]
    fn create_defaults_and_lists() {
        let (c, biz) = setup();
        let m = create(&c, biz, "제목", "본문").unwrap();
        assert_eq!(m.title, "제목");
        assert_eq!(m.pinned, 0);
        assert_eq!(m.color, None);
        assert_eq!(list_by_business(&c, biz).unwrap().len(), 1);
    }

    #[test]
    fn pinned_sorts_first() {
        let (c, biz) = setup();
        let _a = create(&c, biz, "A", "").unwrap();
        let b = create(&c, biz, "B", "").unwrap();
        set_pinned(&c, b.id, true).unwrap();
        let list = list_by_business(&c, biz).unwrap();
        assert_eq!(list[0].id, b.id); // 고정된 B 가 먼저
        assert_eq!(list[0].pinned, 1);
    }

    #[test]
    fn set_color_validates_and_default_clears() {
        let (c, biz) = setup();
        let m = create(&c, biz, "", "").unwrap();
        assert_eq!(set_color(&c, m.id, Some("blue")).unwrap().color.as_deref(), Some("blue"));
        assert_eq!(set_color(&c, m.id, Some("default")).unwrap().color, None);
        assert_eq!(set_color(&c, m.id, None).unwrap().color, None);
        assert!(set_color(&c, m.id, Some("chartreuse")).is_err());
    }

    #[test]
    fn update_changes_title_body() {
        let (c, biz) = setup();
        let m = create(&c, biz, "old", "oldbody").unwrap();
        let u = update(&c, m.id, "new", "newbody").unwrap();
        assert_eq!(u.title, "new");
        assert_eq!(u.body, "newbody");
    }

    #[test]
    fn archive_hides_from_list() {
        let (c, biz) = setup();
        let m = create(&c, biz, "x", "").unwrap();
        archive(&c, m.id).unwrap();
        assert!(list_by_business(&c, biz).unwrap().is_empty());
    }
}
