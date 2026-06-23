use crate::error::{AppError, Result};
use crate::models::Folder;
use rusqlite::{params, Connection, Row};

const KINDS: [&str; 2] = ["document", "deliverable"];

fn map_folder(row: &Row) -> rusqlite::Result<Folder> {
    Ok(Folder {
        id: row.get("id")?,
        business_id: row.get("business_id")?,
        kind: row.get("kind")?,
        parent_id: row.get("parent_id")?,
        name: row.get("name")?,
        sort_order: row.get("sort_order")?,
        archived_at: row.get("archived_at")?,
    })
}

/// 사업의 모든(문서·산출물) 활성 폴더를 조회. 프론트가 kind/parent 로 트리를 구성한다.
pub fn list_by_business(conn: &Connection, business_id: i64) -> Result<Vec<Folder>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM folder WHERE business_id=?1 AND archived_at IS NULL \
         ORDER BY sort_order, id",
    )?;
    let rows = stmt.query_map(params![business_id], map_folder)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: i64) -> Result<Folder> {
    conn.query_row("SELECT * FROM folder WHERE id=?1", params![id], map_folder)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => AppError::Db(other),
        })
}

/// 주어진 폴더가 해당 사업·kind 소속인지 확인. 항목(문서/산출물) 배치 검증용.
/// parent_id 가 없으면(None) 미분류로 간주하고 항상 통과.
pub fn ensure_owns(
    conn: &Connection,
    folder_id: Option<i64>,
    business_id: i64,
    kind: &str,
) -> Result<()> {
    let Some(fid) = folder_id else { return Ok(()) };
    let f = get(conn, fid)?;
    if f.business_id != business_id || f.kind != kind {
        return Err(AppError::Invalid("폴더가 해당 사업/종류 소속이 아닙니다".into()));
    }
    Ok(())
}

/// 폴더 생성. parent_id 가 있으면 그 부모는 같은 사업·kind 의 루트 폴더여야 한다(2단계 강제).
pub fn create(
    conn: &Connection,
    business_id: i64,
    kind: &str,
    parent_id: Option<i64>,
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
        let parent = get(conn, pid)?;
        if parent.business_id != business_id || parent.kind != kind {
            return Err(AppError::Invalid("상위 폴더가 해당 사업/종류 소속이 아닙니다".into()));
        }
        if parent.parent_id.is_some() {
            return Err(AppError::Invalid("폴더는 2단계까지만 만들 수 있습니다".into()));
        }
    }
    let next: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order),0)+1 FROM folder \
         WHERE business_id=?1 AND kind=?2 AND parent_id IS ?3",
        params![business_id, kind, parent_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO folder (business_id, kind, parent_id, name, sort_order) \
         VALUES (?1,?2,?3,?4,?5)",
        params![business_id, kind, parent_id, name, next],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn rename(conn: &Connection, id: i64, name: &str) -> Result<Folder> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invalid("폴더 이름은 비어 있을 수 없습니다".into()));
    }
    let n = conn.execute(
        "UPDATE folder SET name=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
        params![id, name],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

/// 폴더 삭제(즉시). 자식 폴더는 FK CASCADE, 안의 항목은 folder_id→NULL(미분류).
pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let n = conn.execute("DELETE FROM folder WHERE id=?1", params![id])?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, repo::business, repo::deliverable, repo::document};

    fn setup() -> (Connection, i64) {
        let c = db::open_in_memory().unwrap();
        let b = business::create(&c, "사업", "si", None).unwrap();
        (c, b.id)
    }

    #[test]
    fn create_root_and_sub_two_levels() {
        let (c, biz) = setup();
        let root = create(&c, biz, "document", None, "보고서").unwrap();
        assert_eq!(root.parent_id, None);
        let sub = create(&c, biz, "document", Some(root.id), "1차").unwrap();
        assert_eq!(sub.parent_id, Some(root.id));
        assert_eq!(list_by_business(&c, biz).unwrap().len(), 2);
    }

    #[test]
    fn rejects_third_level() {
        let (c, biz) = setup();
        let root = create(&c, biz, "deliverable", None, "A").unwrap();
        let sub = create(&c, biz, "deliverable", Some(root.id), "B").unwrap();
        // 손자(3단계)는 거부
        assert!(create(&c, biz, "deliverable", Some(sub.id), "C").is_err());
    }

    #[test]
    fn validates_name_kind_and_parent_ownership() {
        let (c, biz) = setup();
        assert!(create(&c, biz, "document", None, "  ").is_err());
        assert!(create(&c, biz, "bogus", None, "x").is_err());
        // 다른 사업의 폴더를 부모로 지정 불가
        let other = business::create(&c, "다른", "ops", None).unwrap();
        let foreign = create(&c, other.id, "document", None, "F").unwrap();
        assert!(create(&c, biz, "document", Some(foreign.id), "x").is_err());
        // kind 가 다른 부모도 불가
        let droot = create(&c, biz, "document", None, "문서폴더").unwrap();
        assert!(create(&c, biz, "deliverable", Some(droot.id), "x").is_err());
    }

    #[test]
    fn delete_uncategorizes_items_and_cascades_children() {
        let (c, biz) = setup();
        let root = create(&c, biz, "deliverable", None, "납품").unwrap();
        let sub = create(&c, biz, "deliverable", Some(root.id), "최종").unwrap();
        let d = deliverable::create_file(&c, biz, None, Some(sub.id), "a.pdf", "a.pdf", 10).unwrap();
        assert_eq!(d.folder_id, Some(sub.id));
        // 루트 삭제 → 자식 폴더 cascade, 항목은 미분류로
        delete(&c, root.id).unwrap();
        assert!(get(&c, sub.id).is_err());
        assert_eq!(deliverable::get(&c, d.id).unwrap().folder_id, None);
    }

    #[test]
    fn ensure_owns_guards_cross_business_and_kind() {
        let (c, biz) = setup();
        let docf = create(&c, biz, "document", None, "DF").unwrap();
        // 문서 폴더에 문서 OK, 산출물은 거부
        assert!(ensure_owns(&c, Some(docf.id), biz, "document").is_ok());
        assert!(ensure_owns(&c, Some(docf.id), biz, "deliverable").is_err());
        assert!(ensure_owns(&c, None, biz, "document").is_ok());
        // 문서를 그 폴더에 생성 가능
        let doc = document::create(&c, biz, None, Some(docf.id), "문서").unwrap();
        assert_eq!(doc.folder_id, Some(docf.id));
    }
}
