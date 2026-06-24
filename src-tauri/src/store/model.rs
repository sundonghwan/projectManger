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
}
