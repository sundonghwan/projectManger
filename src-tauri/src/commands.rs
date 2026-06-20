use crate::error::Result;
use crate::models::Business;
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
