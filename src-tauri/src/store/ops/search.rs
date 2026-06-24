use crate::error::Result;
use crate::store::Store;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub business_id: String,
    pub project_id: Option<String>,
}

/// 사업/프로젝트/태스크/문서(제목+본문)/산출물 제목을 부분일치 검색. 보관 항목 제외.
/// 각 종류는 제목순 + 상한 적용, business→project→task→document→deliverable 순으로 이어 붙인다.
pub fn search(store: &Store, query: &str) -> Result<Vec<SearchHit>> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let mut out: Vec<SearchHit> = Vec::new();

    // business (LIMIT 20)
    let mut biz: Vec<_> = store
        .businesses
        .list()
        .into_iter()
        .filter(|b| b.archived_at.is_none() && b.name.contains(q))
        .collect();
    biz.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
    for b in biz.into_iter().take(20) {
        out.push(SearchHit {
            kind: "business".into(),
            id: b.id.clone(),
            title: b.name,
            business_id: b.id,
            project_id: None,
        });
    }

    // project (LIMIT 20)
    let mut proj: Vec<_> = store
        .projects
        .list()
        .into_iter()
        .filter(|p| p.archived_at.is_none() && p.name.contains(q))
        .collect();
    proj.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
    for p in proj.into_iter().take(20) {
        out.push(SearchHit {
            kind: "project".into(),
            id: p.id.clone(),
            title: p.name,
            business_id: p.business_id,
            project_id: Some(p.id),
        });
    }

    // task (LIMIT 30)
    let mut tasks: Vec<_> = store
        .tasks
        .list()
        .into_iter()
        .filter(|t| t.archived_at.is_none() && t.title.contains(q))
        .collect();
    tasks.sort_by(|a, b| a.title.cmp(&b.title).then_with(|| a.id.cmp(&b.id)));
    for t in tasks.into_iter().take(30) {
        out.push(SearchHit {
            kind: "task".into(),
            id: t.id,
            title: t.title,
            business_id: t.business_id,
            project_id: t.project_id,
        });
    }

    // document: 제목 또는 본문 일치 (LIMIT 20)
    let mut docs: Vec<_> = store
        .documents
        .list()
        .into_iter()
        .filter(|d| d.archived_at.is_none() && (d.title.contains(q) || d.body.contains(q)))
        .collect();
    docs.sort_by(|a, b| a.title.cmp(&b.title).then_with(|| a.id.cmp(&b.id)));
    for d in docs.into_iter().take(20) {
        out.push(SearchHit {
            kind: "document".into(),
            id: d.id,
            title: d.title,
            business_id: d.business_id,
            project_id: d.project_id,
        });
    }

    // deliverable (LIMIT 20)
    let mut dels: Vec<_> = store
        .deliverables
        .list()
        .into_iter()
        .filter(|d| d.archived_at.is_none() && d.title.contains(q))
        .collect();
    dels.sort_by(|a, b| a.title.cmp(&b.title).then_with(|| a.id.cmp(&b.id)));
    for d in dels.into_iter().take(20) {
        out.push(SearchHit {
            kind: "deliverable".into(),
            id: d.id,
            title: d.title,
            business_id: d.business_id,
            project_id: d.project_id,
        });
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::ops::{business, document, project, task};
    use crate::store::Store;

    fn store() -> Store {
        Store::open(std::env::temp_dir().join(format!("ops_search_{}", new_id()))).unwrap()
    }

    #[test]
    fn empty_query_returns_nothing() {
        let mut s = store();
        business::create(&mut s, "사업", "si", None).unwrap();
        assert!(search(&s, "  ").unwrap().is_empty());
    }

    #[test]
    fn finds_across_kinds() {
        let mut s = store();
        let b = business::create(&mut s, "알파 사업", "si", None).unwrap();
        let p = project::create(&mut s, &b.id, "알파 프로젝트").unwrap();
        task::create(&mut s, &b.id, Some(&p.id), "알파 태스크", 2).unwrap();
        document::create(&mut s, &b.id, None, None, "알파 문서").unwrap();
        let hits = search(&s, "알파").unwrap();
        let kinds: Vec<&str> = hits.iter().map(|h| h.kind.as_str()).collect();
        assert!(kinds.contains(&"business"));
        assert!(kinds.contains(&"project"));
        assert!(kinds.contains(&"task"));
        assert!(kinds.contains(&"document"));
        assert_eq!(hits.len(), 4);
    }

    #[test]
    fn finds_document_by_body() {
        let mut s = store();
        let b = business::create(&mut s, "사업", "si", None).unwrap();
        let d = document::create(&mut s, &b.id, None, None, "회의록").unwrap();
        document::set_body(&mut s, &d.id, "# 안건\n예산 3억 검토").unwrap();
        let hits = search(&s, "예산").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, "document");
        assert_eq!(hits[0].title, "회의록");
    }

    #[test]
    fn excludes_archived_and_non_matching() {
        let mut s = store();
        let b = business::create(&mut s, "찾을것", "si", None).unwrap();
        business::create(&mut s, "다른것", "si", None).unwrap();
        let t = task::create(&mut s, &b.id, None, "보관될태스크", 2).unwrap();
        task::archive(&mut s, &t.id).unwrap();
        let hits = search(&s, "찾을").unwrap();
        assert_eq!(hits.len(), 1);
        assert!(search(&s, "보관될").unwrap().is_empty());
    }

    #[test]
    fn hit_serializes_camelcase() {
        let h = SearchHit { kind: "task".into(), id: "t".into(), title: "T".into(), business_id: "b".into(), project_id: Some("p".into()) };
        let j = serde_json::to_value(&h).unwrap();
        assert_eq!(j["businessId"], "b");
        assert_eq!(j["projectId"], "p");
    }
}
