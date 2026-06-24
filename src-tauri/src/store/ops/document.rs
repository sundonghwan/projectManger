use crate::error::{AppError, Result};
use crate::store::ids::{new_id, now};
use crate::store::model::{Block, Document};
use crate::store::ops::folder;
use crate::store::ops::util::cmp_sort;
use crate::store::Store;

pub fn list_by_business(store: &Store, business_id: &str) -> Result<Vec<Document>> {
    let mut out: Vec<Document> = store
        .documents
        .list()
        .into_iter()
        .filter(|d| d.business_id == business_id && d.archived_at.is_none())
        .collect();
    out.sort_by(|a, b| cmp_sort(a.sort_order, &a.id, b.sort_order, &b.id));
    Ok(out)
}

pub fn get(store: &Store, id: &str) -> Result<Document> {
    store.documents.get(id).cloned().ok_or(AppError::NotFound)
}

fn next_sort(store: &Store, business_id: &str) -> f64 {
    store
        .documents
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
) -> Result<Document> {
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
    folder::ensure_owns(store, folder_id, business_id, "document")?;
    let title = if title.trim().is_empty() { "제목 없음" } else { title };
    let sort_order = next_sort(store, business_id);
    let ts = now();
    let d = Document {
        id: new_id(),
        business_id: business_id.to_string(),
        project_id: project_id.map(|s| s.to_string()),
        folder_id: folder_id.map(|s| s.to_string()),
        title: title.to_string(),
        icon: None,
        body: String::new(),
        blocks: Vec::new(),
        sort_order,
        archived_at: None,
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.documents.put(d.clone())?;
    Ok(d)
}

pub fn set_folder(store: &mut Store, id: &str, folder_id: Option<&str>) -> Result<Document> {
    let mut d = get(store, id)?;
    folder::ensure_owns(store, folder_id, &d.business_id, "document")?;
    d.folder_id = folder_id.map(|s| s.to_string());
    d.updated_at = now();
    store.documents.put(d.clone())?;
    Ok(d)
}

pub fn rename(store: &mut Store, id: &str, title: &str) -> Result<Document> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::Invalid("문서 제목은 비어 있을 수 없습니다".into()));
    }
    let mut d = get(store, id)?;
    d.title = title.to_string();
    d.updated_at = now();
    store.documents.put(d.clone())?;
    Ok(d)
}

pub fn set_body(store: &mut Store, id: &str, body: &str) -> Result<()> {
    let mut d = get(store, id)?;
    d.body = body.to_string();
    d.updated_at = now();
    store.documents.put(d)?;
    Ok(())
}

pub fn archive(store: &mut Store, id: &str) -> Result<()> {
    let mut d = get(store, id)?;
    let ts = now();
    d.archived_at = Some(ts.clone());
    d.updated_at = ts;
    store.documents.put(d)?;
    Ok(())
}

// ---- 블록 (Document.blocks 에 인라인) ----

pub fn list_blocks(store: &Store, document_id: &str) -> Result<Vec<Block>> {
    let d = get(store, document_id)?;
    let mut blocks = d.blocks.clone();
    blocks.sort_by(|a, b| cmp_sort(a.sort_order, &a.id, b.sort_order, &b.id));
    Ok(blocks)
}

/// 블록 id 로 소유 문서를 찾는다(인라인 모델이라 문서 스캔).
fn find_doc_id_of_block(store: &Store, block_id: &str) -> Option<String> {
    store
        .documents
        .list()
        .into_iter()
        .find(|d| d.blocks.iter().any(|b| b.id == block_id))
        .map(|d| d.id)
}

pub fn get_block(store: &Store, block_id: &str) -> Result<Block> {
    let doc_id = find_doc_id_of_block(store, block_id).ok_or(AppError::NotFound)?;
    let d = get(store, &doc_id)?;
    d.blocks
        .into_iter()
        .find(|b| b.id == block_id)
        .ok_or(AppError::NotFound)
}

pub fn create_block(
    store: &mut Store,
    document_id: &str,
    type_: &str,
    content: &str,
    sort_order: f64,
) -> Result<Block> {
    let mut d = get(store, document_id)?;
    let b = Block {
        id: new_id(),
        parent_block_id: None,
        r#type: type_.to_string(),
        content: content.to_string(),
        sort_order,
    };
    d.blocks.push(b.clone());
    d.updated_at = now();
    store.documents.put(d)?;
    Ok(b)
}

pub fn update_block(store: &mut Store, block_id: &str, type_: &str, content: &str) -> Result<Block> {
    let doc_id = find_doc_id_of_block(store, block_id).ok_or(AppError::NotFound)?;
    let mut d = get(store, &doc_id)?;
    let blk = d
        .blocks
        .iter_mut()
        .find(|b| b.id == block_id)
        .ok_or(AppError::NotFound)?;
    blk.r#type = type_.to_string();
    blk.content = content.to_string();
    let updated = blk.clone();
    d.updated_at = now();
    store.documents.put(d)?;
    Ok(updated)
}

pub fn delete_block(store: &mut Store, block_id: &str) -> Result<()> {
    let doc_id = find_doc_id_of_block(store, block_id).ok_or(AppError::NotFound)?;
    let mut d = get(store, &doc_id)?;
    let before = d.blocks.len();
    d.blocks.retain(|b| b.id != block_id);
    if d.blocks.len() == before {
        return Err(AppError::NotFound);
    }
    d.updated_at = now();
    store.documents.put(d)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::ops::{business, folder, project};
    use crate::store::Store;

    fn setup() -> (Store, String, String) {
        let mut s = Store::open(std::env::temp_dir().join(format!("ops_doc_{}", new_id()))).unwrap();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let p = project::create(&mut s, &b.id, "프로젝트").unwrap();
        (s, b.id, p.id)
    }

    #[test]
    fn create_direct_and_in_project() {
        let (mut s, biz, proj) = setup();
        let d1 = create(&mut s, &biz, None, None, "직속").unwrap();
        let d2 = create(&mut s, &biz, Some(&proj), None, "프로젝트").unwrap();
        assert_eq!(d1.project_id, None);
        assert_eq!(d2.project_id.as_deref(), Some(proj.as_str()));
        assert_eq!(list_by_business(&s, &biz).unwrap().len(), 2);
    }

    #[test]
    fn blank_title_defaults() {
        let (mut s, biz, _) = setup();
        assert_eq!(create(&mut s, &biz, None, None, "   ").unwrap().title, "제목 없음");
    }

    #[test]
    fn rejects_foreign_project() {
        let (mut s, biz, _) = setup();
        let other = business::create(&mut s, "다른", "ops", None).unwrap();
        let op = project::create(&mut s, &other.id, "P").unwrap();
        assert!(create(&mut s, &biz, Some(&op.id), None, "x").is_err());
    }

    #[test]
    fn create_in_folder_and_move() {
        let (mut s, biz, _) = setup();
        let f = folder::create(&mut s, &biz, "document", None, "보고서").unwrap();
        let d = create(&mut s, &biz, None, Some(&f.id), "분류").unwrap();
        assert_eq!(d.folder_id.as_deref(), Some(f.id.as_str()));
        assert_eq!(set_folder(&mut s, &d.id, None).unwrap().folder_id, None);
        // 산출물 폴더로는 이동 불가
        let df = folder::create(&mut s, &biz, "deliverable", None, "납품").unwrap();
        assert!(set_folder(&mut s, &d.id, Some(&df.id)).is_err());
    }

    #[test]
    fn rename_set_body_archive() {
        let (mut s, biz, _) = setup();
        let d = create(&mut s, &biz, None, None, "원래").unwrap();
        assert_eq!(rename(&mut s, &d.id, "새이름").unwrap().title, "새이름");
        set_body(&mut s, &d.id, "# 본문").unwrap();
        assert_eq!(get(&s, &d.id).unwrap().body, "# 본문");
        archive(&mut s, &d.id).unwrap();
        assert!(list_by_business(&s, &biz).unwrap().is_empty());
    }

    #[test]
    fn blocks_crud_and_order_bumps_document() {
        let (mut s, biz, _) = setup();
        let d = create(&mut s, &biz, None, None, "문서").unwrap();
        let b1 = create_block(&mut s, &d.id, "heading", "{\"text\":\"제목\"}", 1.0).unwrap();
        create_block(&mut s, &d.id, "paragraph", "{\"text\":\"본문\"}", 2.0).unwrap();
        let blocks = list_blocks(&s, &d.id).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].r#type, "heading"); // sort_order 1.0 먼저
        let updated = update_block(&mut s, &b1.id, "heading", "{\"text\":\"수정\"}").unwrap();
        assert!(updated.content.contains("수정"));
        assert_eq!(get_block(&s, &b1.id).unwrap().content, updated.content);
        delete_block(&mut s, &b1.id).unwrap();
        assert_eq!(list_blocks(&s, &d.id).unwrap().len(), 1);
        assert!(matches!(get_block(&s, &b1.id), Err(AppError::NotFound)));
    }
}
