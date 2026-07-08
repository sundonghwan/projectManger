pub mod ids;
pub mod io;
pub mod entity;
pub mod collection;
pub mod model;
pub mod ops;
pub mod layout;
pub mod local;

use crate::error::Result;
use crate::store::collection::Collection;
use crate::store::model::{
    Business, Deliverable, Document, Folder, Label, Memo, Project, RecurringTask, Task, TaskLabel,
    Template,
};
use std::path::PathBuf;

/// 파일 기반 데이터 저장소. 컬렉션별 인메모리 인덱스를 모아 제공한다.
pub struct Store {
    pub root: PathBuf,
    pub businesses: Collection<Business>,
    pub projects: Collection<Project>,
    pub tasks: Collection<Task>,
    pub folders: Collection<Folder>,
    pub memos: Collection<Memo>,
    pub labels: Collection<Label>,
    pub task_labels: Collection<TaskLabel>,
    pub templates: Collection<Template>,
    pub recurring: Collection<RecurringTask>,
    pub documents: Collection<Document>,
    pub deliverables: Collection<Deliverable>,
}

impl Store {
    /// root를 기준으로 store를 열고 디스크 내용을 로드한다.
    pub fn open(root: PathBuf) -> Result<Store> {
        let mut s = Store {
            businesses: Collection::new(&root),
            projects: Collection::new(&root),
            tasks: Collection::new(&root),
            folders: Collection::new(&root),
            memos: Collection::new(&root),
            labels: Collection::new(&root),
            task_labels: Collection::new(&root),
            templates: Collection::new(&root),
            recurring: Collection::new(&root),
            documents: Collection::new(&root),
            deliverables: Collection::new(&root),
            root,
        };
        s.load()?;
        Ok(s)
    }

    /// 모든 컬렉션을 디스크에서 다시 로드한다.
    pub fn load(&mut self) -> Result<()> {
        self.businesses.load()?;
        self.projects.load()?;
        self.tasks.load()?;
        self.folders.load()?;
        self.memos.load()?;
        self.labels.load()?;
        self.task_labels.load()?;
        self.templates.load()?;
        self.recurring.load()?;
        self.documents.load()?;
        self.deliverables.load()?;
        Ok(())
    }

    /// 존재하지 않는 business 또는 project를 가리키는 task id 목록(고아).
    pub fn orphan_task_ids(&self) -> Vec<String> {
        self.tasks
            .list()
            .into_iter()
            .filter(|t| {
                let biz_missing = self.businesses.get(&t.business_id).is_none();
                let proj_missing = t
                    .project_id
                    .as_ref()
                    .map(|p| self.projects.get(p).is_none())
                    .unwrap_or(false);
                biz_missing || proj_missing
            })
            .map(|t| t.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::{new_id, now};
    use crate::store::model::{Business, Project, Task};

    fn tmp_root() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("store_root_{}", new_id()))
    }

    fn biz(id: &str) -> Business {
        Business {
            id: id.into(), name: "사업".into(), r#type: "si".into(), color: None,
            description: None, status: "active".into(), sort_order: 0.0,
            archived_at: None, created_at: now(), updated_at: now(),
        }
    }
    fn proj(id: &str, biz: &str) -> Project {
        Project {
            id: id.into(), business_id: biz.into(), name: "P".into(), description: None,
            status: "active".into(), start_date: None, due_date: None, sort_order: 0.0,
            archived_at: None, created_at: now(), updated_at: now(),
        }
    }
    fn task(id: &str, biz: &str, proj: Option<&str>) -> Task {
        Task {
            id: id.into(), business_id: biz.into(),
            project_id: proj.map(|s| s.to_string()), parent_task_id: None,
            title: "T".into(), description: None, status: "todo".into(), priority: 2,
            start_date: None, due_date: None, sort_order: 0.0, completed_at: None, archived_at: None,
            created_at: now(), updated_at: now(),
        }
    }

    #[test]
    fn open_empty_root_creates_usable_store() {
        let mut s = Store::open(tmp_root()).unwrap();
        assert_eq!(s.businesses.len(), 0);
        s.businesses.put(biz("b1")).unwrap();
        assert_eq!(s.businesses.len(), 1);
    }

    #[test]
    fn data_survives_reopen() {
        let root = tmp_root();
        {
            let mut s = Store::open(root.clone()).unwrap();
            s.businesses.put(biz("b1")).unwrap();
            s.projects.put(proj("p1", "b1")).unwrap();
            s.tasks.put(task("t1", "b1", Some("p1"))).unwrap();
        }
        let s2 = Store::open(root).unwrap();
        assert_eq!(s2.businesses.len(), 1);
        assert_eq!(s2.projects.len(), 1);
        assert_eq!(s2.tasks.len(), 1);
        assert_eq!(s2.tasks.get("t1").unwrap().project_id.as_deref(), Some("p1"));
    }

    #[test]
    fn detects_orphan_tasks() {
        let mut s = Store::open(tmp_root()).unwrap();
        s.businesses.put(biz("b1")).unwrap();
        s.projects.put(proj("p1", "b1")).unwrap();
        s.tasks.put(task("ok", "b1", Some("p1"))).unwrap();   // 정상
        s.tasks.put(task("no_biz", "ghost", None)).unwrap();   // 사업 없음
        s.tasks.put(task("no_proj", "b1", Some("ghost"))).unwrap(); // 프로젝트 없음
        let mut orphans = s.orphan_task_ids();
        orphans.sort();
        assert_eq!(orphans, vec!["no_biz".to_string(), "no_proj".to_string()]);
    }

    #[test]
    fn all_collections_persist_across_reopen() {
        use crate::store::model::{Folder, Memo, Document, Block};
        let root = tmp_root();
        {
            let mut s = Store::open(root.clone()).unwrap();
            s.folders.put(Folder {
                id: "f1".into(), business_id: "b1".into(), kind: "document".into(),
                parent_id: None, name: "F".into(), sort_order: 0.0, archived_at: None,
                created_at: now(), updated_at: now(),
            }).unwrap();
            s.memos.put(Memo {
                id: "m1".into(), business_id: "b1".into(), title: "T".into(), body: "B".into(),
                color: None, pinned: 0, sort_order: 0.0, archived_at: None,
                created_at: now(), updated_at: now(),
            }).unwrap();
            s.documents.put(Document {
                id: "d1".into(), business_id: "b1".into(), project_id: None, folder_id: None,
                title: "D".into(), icon: None, body: "".into(),
                editor_body: None, editor_body_format: None, collaboration_state: None,
                blocks: vec![Block { id: "blk".into(), parent_block_id: None,
                    r#type: "paragraph".into(), content: "{}".into(), sort_order: 0.0 }],
                sort_order: 0.0, archived_at: None, created_at: now(), updated_at: now(),
            }).unwrap();
        }
        let s2 = Store::open(root).unwrap();
        assert_eq!(s2.folders.len(), 1);
        assert_eq!(s2.memos.len(), 1);
        assert_eq!(s2.documents.get("d1").unwrap().blocks.len(), 1);
    }
}
