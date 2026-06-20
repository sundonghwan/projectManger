use crate::error::Result;
use crate::models::{Business, Project, Task};
use crate::repo;
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::State;

/// 앱 전역 상태 — SQLite 연결을 Mutex로 보호.
pub struct AppState {
    pub db: Mutex<Connection>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessCreate {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub color: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessUpdate {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub status: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[tauri::command]
pub fn business_list(state: State<AppState>) -> Result<Vec<Business>> {
    let conn = state.db.lock().unwrap();
    repo::business::list(&conn)
}

#[tauri::command]
pub fn business_create(state: State<AppState>, input: BusinessCreate) -> Result<Business> {
    let conn = state.db.lock().unwrap();
    repo::business::create(&conn, &input.name, &input.type_, input.color.as_deref())
}

#[tauri::command]
pub fn business_update(state: State<AppState>, input: BusinessUpdate) -> Result<Business> {
    let conn = state.db.lock().unwrap();
    repo::business::update(
        &conn,
        input.id,
        &input.name,
        &input.type_,
        &input.status,
        input.color.as_deref(),
        input.description.as_deref(),
    )
}

#[tauri::command]
pub fn business_archive(state: State<AppState>, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::business::archive(&conn, id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCreate {
    pub business_id: i64,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdate {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
}

#[tauri::command]
pub fn project_list(state: State<AppState>, business_id: i64) -> Result<Vec<Project>> {
    let conn = state.db.lock().unwrap();
    repo::project::list_by_business(&conn, business_id)
}

#[tauri::command]
pub fn project_create(state: State<AppState>, input: ProjectCreate) -> Result<Project> {
    let conn = state.db.lock().unwrap();
    repo::project::create(&conn, input.business_id, &input.name)
}

#[tauri::command]
pub fn project_update(state: State<AppState>, input: ProjectUpdate) -> Result<Project> {
    let conn = state.db.lock().unwrap();
    repo::project::update(
        &conn,
        input.id,
        &input.name,
        &input.status,
        input.description.as_deref(),
        input.due_date.as_deref(),
    )
}

#[tauri::command]
pub fn project_archive(state: State<AppState>, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::project::archive(&conn, id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCreate {
    pub business_id: i64,
    pub project_id: Option<i64>,
    pub title: String,
    #[serde(default = "default_priority")]
    pub priority: i64,
}
fn default_priority() -> i64 {
    2
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskUpdate {
    pub id: i64,
    pub title: String,
    pub priority: i64,
    pub due_date: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskMove {
    pub id: i64,
    pub status: String,
    pub sort_order: f64,
}

#[tauri::command]
pub fn task_list(
    state: State<AppState>,
    business_id: i64,
    project_id: Option<i64>,
) -> Result<Vec<Task>> {
    let conn = state.db.lock().unwrap();
    repo::task::list(&conn, business_id, project_id)
}

#[tauri::command]
pub fn task_create(state: State<AppState>, input: TaskCreate) -> Result<Task> {
    let conn = state.db.lock().unwrap();
    repo::task::create(&conn, input.business_id, input.project_id, &input.title, input.priority)
}

#[tauri::command]
pub fn task_update(state: State<AppState>, input: TaskUpdate) -> Result<Task> {
    let conn = state.db.lock().unwrap();
    repo::task::update(
        &conn,
        input.id,
        &input.title,
        input.priority,
        input.due_date.as_deref(),
        input.description.as_deref(),
    )
}

#[tauri::command]
pub fn task_move(state: State<AppState>, input: TaskMove) -> Result<Task> {
    let conn = state.db.lock().unwrap();
    repo::task::move_task(&conn, input.id, &input.status, input.sort_order)
}

#[tauri::command]
pub fn task_archive(state: State<AppState>, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::task::archive(&conn, id)
}
