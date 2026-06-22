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
pub struct CommandSnippet {
    pub id: i64,
    pub server_connection_id: Option<i64>,
    pub name: String,
    pub command: String,
    pub sort_order: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConnection {
    pub id: i64,
    pub business_id: i64,
    pub project_id: Option<i64>,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub auth_type: String,
    pub key_path: Option<String>,
    /// OS 키체인 참조 키 — 실제 비밀값은 DB에 없음
    pub secret_ref: Option<String>,
    pub last_used_at: Option<String>,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringTask {
    pub id: i64,
    pub business_id: i64,
    pub project_id: Option<i64>,
    pub title: String,
    pub priority: i64,
    pub interval_days: i64,
    pub next_run: String,
    pub active: i64,
}

/// 휴지통(보관) 항목
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashItem {
    pub kind: String, // business | project | task | document
    pub id: i64,
    pub title: String,
    pub archived_at: Option<String>,
}

/// 전역 검색 결과 항목
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub kind: String, // business | project | task | document
    pub id: i64,
    pub title: String,
    pub business_id: i64,
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
}

/// 태스크-라벨 조인 행 (사업 단위 일괄 조회용)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLabel {
    pub task_id: i64,
    pub label_id: i64,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: i64,
    pub business_id: i64,
    pub project_id: Option<i64>,
    pub title: String,
    pub icon: Option<String>,
    pub body: String,
    pub sort_order: f64,
    pub archived_at: Option<String>,
    pub created_at: String,
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
pub struct Deliverable {
    pub id: i64,
    pub business_id: i64,
    pub project_id: Option<i64>,
    pub title: String,
    pub kind: String,
    pub document_id: Option<i64>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub original_name: Option<String>,
    pub status: String,
    pub current_version: i64,
    pub sort_order: f64,
    pub archived_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliverableVersion {
    pub id: i64,
    pub deliverable_id: i64,
    pub version: i64,
    pub file_path: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
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
