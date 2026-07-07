use crate::store::entity::Entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Business {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub color: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub sort_order: f64,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for Business {
    fn collection() -> &'static str { "businesses" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub business_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub sort_order: f64,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for Project {
    fn collection() -> &'static str { "projects" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub business_id: String,
    pub project_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: i64,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub sort_order: f64,
    pub completed_at: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for Task {
    fn collection() -> &'static str { "tasks" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub business_id: String,
    pub kind: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub sort_order: f64,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for Folder {
    fn collection() -> &'static str { "folders" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Memo {
    pub id: String,
    pub business_id: String,
    pub title: String,
    pub body: String,
    pub color: Option<String>,
    pub pinned: i64,
    pub sort_order: f64,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for Memo {
    fn collection() -> &'static str { "memos" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for Label {
    fn collection() -> &'static str { "labels" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

/// 태스크-라벨 관계(자체 uuid 보유). 연결=행 존재, 해제=행 삭제(향후 tombstone).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLabel {
    pub id: String,
    pub task_id: String,
    pub label_id: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for TaskLabel {
    fn collection() -> &'static str { "task_labels" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub payload: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for Template {
    fn collection() -> &'static str { "templates" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringTask {
    pub id: String,
    pub business_id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub priority: i64,
    pub interval_days: i64,
    pub next_run: String,
    pub active: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for RecurringTask {
    fn collection() -> &'static str { "recurring" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub id: String,
    pub parent_block_id: Option<String>,
    pub r#type: String,
    pub content: String,
    pub sort_order: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: String,
    pub business_id: String,
    pub project_id: Option<String>,
    pub folder_id: Option<String>,
    pub title: String,
    pub icon: Option<String>,
    pub body: String,
    pub editor_body: Option<String>,
    pub editor_body_format: Option<String>,
    pub collaboration_state: Option<String>,
    pub blocks: Vec<Block>,
    pub sort_order: f64,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for Document {
    fn collection() -> &'static str { "documents" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliverableVersion {
    pub id: String,
    pub version: i64,
    pub file_path: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deliverable {
    pub id: String,
    pub business_id: String,
    pub project_id: Option<String>,
    pub folder_id: Option<String>,
    pub title: String,
    pub kind: String,
    pub document_id: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub original_name: Option<String>,
    pub status: String,
    pub current_version: i64,
    pub versions: Vec<DeliverableVersion>,
    pub sort_order: f64,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for Deliverable {
    fn collection() -> &'static str { "deliverables" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: String,
    pub business_id: String,
    pub project_id: Option<String>,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub auth_type: String,
    pub key_path: Option<String>,
    pub secret_ref: Option<String>,
    pub last_used_at: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for Server {
    fn collection() -> &'static str { "servers" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub command: String,
    pub sort_order: f64,
    pub created_at: String,
    pub updated_at: String,
}

impl Entity for Snippet {
    fn collection() -> &'static str { "snippets" }
    fn id(&self) -> &str { &self.id }
    fn updated_at(&self) -> &str { &self.updated_at }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::entity::Entity;

    #[test]
    fn entity_metadata_is_correct() {
        assert_eq!(Business::collection(), "businesses");
        assert_eq!(Project::collection(), "projects");
        assert_eq!(Task::collection(), "tasks");
    }

    #[test]
    fn business_serializes_camelcase() {
        let b = Business {
            id: "b1".into(),
            name: "사업".into(),
            r#type: "si".into(),
            color: None,
            description: None,
            status: "active".into(),
            sort_order: 0.0,
            archived_at: None,
            created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-02T00:00:00.000Z".into(),
        };
        let v = serde_json::to_value(&b).unwrap();
        assert_eq!(v["sortOrder"], 0.0);
        assert_eq!(v["createdAt"], "2026-01-01T00:00:00.000Z");
        assert_eq!(b.id(), "b1");
        assert_eq!(b.updated_at(), "2026-01-02T00:00:00.000Z");
    }

    #[test]
    fn task_optional_fks_roundtrip() {
        let t = Task {
            id: "t1".into(),
            business_id: "b1".into(),
            project_id: Some("p1".into()),
            parent_task_id: None,
            title: "할 일".into(),
            description: None,
            status: "todo".into(),
            priority: 2,
            start_date: None,
            due_date: None,
            sort_order: 1.0,
            completed_at: None,
            archived_at: None,
            created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-01T00:00:00.000Z".into(),
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(back.project_id.as_deref(), Some("p1"));
        assert_eq!(back.parent_task_id, None);
    }

    #[test]
    fn simple_entities_metadata() {
        assert_eq!(Folder::collection(), "folders");
        assert_eq!(Memo::collection(), "memos");
        assert_eq!(Label::collection(), "labels");
        assert_eq!(TaskLabel::collection(), "task_labels");
        assert_eq!(Template::collection(), "templates");
        assert_eq!(RecurringTask::collection(), "recurring");
    }

    #[test]
    fn folder_optional_parent_roundtrip_camelcase() {
        let f = Folder {
            id: "f1".into(), business_id: "b1".into(), kind: "document".into(),
            parent_id: None, name: "폴더".into(), sort_order: 1.0,
            archived_at: None, created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-01T00:00:00.000Z".into(),
        };
        let v = serde_json::to_value(&f).unwrap();
        assert_eq!(v["businessId"], "b1");
        assert!(v["parentId"].is_null());
        assert_eq!(v["sortOrder"], 1.0);
        let back: Folder = serde_json::from_value(v).unwrap();
        assert_eq!(back.parent_id, None);
        assert_eq!(f.id(), "f1");
    }

    #[test]
    fn task_label_is_relation_entity() {
        let tl = TaskLabel {
            id: "tl1".into(), task_id: "t1".into(), label_id: "l1".into(),
            created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-02T00:00:00.000Z".into(),
        };
        let v = serde_json::to_value(&tl).unwrap();
        assert_eq!(v["taskId"], "t1");
        assert_eq!(v["labelId"], "l1");
        assert_eq!(tl.id(), "tl1");
        assert_eq!(tl.updated_at(), "2026-01-02T00:00:00.000Z");
    }

    #[test]
    fn document_with_inline_blocks_roundtrip() {
        assert_eq!(Document::collection(), "documents");
        let d = Document {
            id: "d1".into(), business_id: "b1".into(), project_id: None,
            folder_id: Some("f1".into()), title: "문서".into(), icon: None,
            body: "본문".into(), editor_body: None, editor_body_format: None,
            collaboration_state: None,
            blocks: vec![Block {
                id: "blk1".into(), parent_block_id: None,
                r#type: "paragraph".into(), content: "{}".into(), sort_order: 1.0,
            }],
            sort_order: 0.0, archived_at: None,
            created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-01T00:00:00.000Z".into(),
        };
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(v["folderId"], "f1");
        assert_eq!(v["blocks"][0]["type"], "paragraph");
        let back: Document = serde_json::from_value(v).unwrap();
        assert_eq!(back.blocks.len(), 1);
        assert_eq!(back.blocks[0].id, "blk1");
        assert_eq!(d.id(), "d1");
    }

    #[test]
    fn deliverable_with_inline_versions_roundtrip() {
        assert_eq!(Deliverable::collection(), "deliverables");
        let d = Deliverable {
            id: "dv1".into(), business_id: "b1".into(), project_id: Some("p1".into()),
            folder_id: None, title: "산출물".into(), kind: "file".into(),
            document_id: None, file_path: Some("/x".into()), file_size: Some(10),
            original_name: Some("a.pdf".into()), status: "draft".into(),
            current_version: 2,
            versions: vec![DeliverableVersion {
                id: "ver1".into(), version: 1, file_path: Some("/x".into()), note: None,
                created_at: "2026-01-01T00:00:00.000Z".into(),
            }],
            sort_order: 0.0, archived_at: None,
            created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-01T00:00:00.000Z".into(),
        };
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(v["currentVersion"], 2);
        assert_eq!(v["originalName"], "a.pdf");
        assert_eq!(v["versions"][0]["version"], 1);
        let back: Deliverable = serde_json::from_value(v).unwrap();
        assert_eq!(back.versions.len(), 1);
        assert_eq!(back.document_id, None);
    }

    #[test]
    fn server_and_snippet_metadata_and_roundtrip() {
        assert_eq!(Server::collection(), "servers");
        assert_eq!(Snippet::collection(), "snippets");
        let s = Server {
            id: "s1".into(), business_id: "b1".into(), project_id: None, name: "스테이징".into(),
            host: "10.0.0.5".into(), port: 22, username: "deploy".into(), auth_type: "key".into(),
            key_path: Some("/home/u/.ssh/id".into()), secret_ref: None, last_used_at: None,
            archived_at: None, created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-02T00:00:00.000Z".into(),
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["businessId"], "b1");
        assert_eq!(v["authType"], "key");
        assert!(v["projectId"].is_null());
        assert_eq!(s.id(), "s1");
        let sn = Snippet {
            id: "n1".into(), server_id: "s1".into(), name: "배포".into(), command: "./d.sh".into(),
            sort_order: 1.0, created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-01T00:00:00.000Z".into(),
        };
        let vn = serde_json::to_value(&sn).unwrap();
        assert_eq!(vn["serverId"], "s1");
        assert_eq!(sn.id(), "n1");
    }
}
