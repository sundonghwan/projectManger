use serde::{Deserialize, Serialize};

/// 프론트(TS)와 동일한 camelCase 직렬화. docs/02-데이터모델 기준.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Business {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub color: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub sort_order: f64,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: i64,
    pub business_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub sort_order: f64,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: i64,
    pub business_id: i64,
    pub project_id: Option<i64>,
    pub title: String,
    pub icon: Option<String>,
    pub sort_order: f64,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub id: i64,
    pub document_id: i64,
    pub parent_block_id: Option<i64>,
    pub r#type: String,
    pub content: String,
    pub sort_order: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: i64,
    pub business_id: i64,
    pub project_id: Option<i64>,
    pub parent_task_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: i64,
    pub due_date: Option<String>,
    pub sort_order: f64,
    pub completed_at: Option<String>,
    pub archived_at: Option<String>,
}
