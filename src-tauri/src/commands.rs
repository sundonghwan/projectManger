use crate::error::Result;
use crate::models::{
    Block, Business, Deliverable, DeliverableVersion, Document, Label, Project, SearchHit,
    ServerConnection, Task, TaskLabel, TrashItem,
};
use crate::secrets;
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
pub fn business_rename(state: State<AppState>, id: i64, name: String) -> Result<Business> {
    let conn = state.db.lock().unwrap();
    repo::business::rename(&conn, id, &name)
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
pub fn project_rename(state: State<AppState>, id: i64, name: String) -> Result<Project> {
    let conn = state.db.lock().unwrap();
    repo::project::rename(&conn, id, &name)
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentCreate {
    pub business_id: i64,
    pub project_id: Option<i64>,
    pub title: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockCreate {
    pub document_id: i64,
    #[serde(rename = "type")]
    pub type_: String,
    pub content: String,
    pub sort_order: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockUpdate {
    pub id: i64,
    #[serde(rename = "type")]
    pub type_: String,
    pub content: String,
}

#[tauri::command]
pub fn document_list(state: State<AppState>, business_id: i64) -> Result<Vec<Document>> {
    let conn = state.db.lock().unwrap();
    repo::document::list_by_business(&conn, business_id)
}

#[tauri::command]
pub fn document_create(state: State<AppState>, input: DocumentCreate) -> Result<Document> {
    let conn = state.db.lock().unwrap();
    repo::document::create(&conn, input.business_id, input.project_id, &input.title)
}

#[tauri::command]
pub fn document_rename(state: State<AppState>, id: i64, title: String) -> Result<Document> {
    let conn = state.db.lock().unwrap();
    repo::document::rename(&conn, id, &title)
}

#[tauri::command]
pub fn document_archive(state: State<AppState>, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::document::archive(&conn, id)
}

#[tauri::command]
pub fn block_list(state: State<AppState>, document_id: i64) -> Result<Vec<Block>> {
    let conn = state.db.lock().unwrap();
    repo::document::list_blocks(&conn, document_id)
}

#[tauri::command]
pub fn block_create(state: State<AppState>, input: BlockCreate) -> Result<Block> {
    let conn = state.db.lock().unwrap();
    repo::document::create_block(&conn, input.document_id, &input.type_, &input.content, input.sort_order)
}

#[tauri::command]
pub fn block_update(state: State<AppState>, input: BlockUpdate) -> Result<Block> {
    let conn = state.db.lock().unwrap();
    repo::document::update_block(&conn, input.id, &input.type_, &input.content)
}

#[tauri::command]
pub fn block_delete(state: State<AppState>, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::document::delete_block(&conn, id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelCreate {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelAssign {
    pub task_id: i64,
    pub label_id: i64,
}

#[tauri::command]
pub fn label_list(state: State<AppState>) -> Result<Vec<Label>> {
    let conn = state.db.lock().unwrap();
    repo::label::list(&conn)
}

#[tauri::command]
pub fn label_create(state: State<AppState>, input: LabelCreate) -> Result<Label> {
    let conn = state.db.lock().unwrap();
    repo::label::create(&conn, &input.name, input.color.as_deref())
}

#[tauri::command]
pub fn label_assign(state: State<AppState>, input: LabelAssign) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::label::assign(&conn, input.task_id, input.label_id)
}

#[tauri::command]
pub fn label_unassign(state: State<AppState>, input: LabelAssign) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::label::unassign(&conn, input.task_id, input.label_id)
}

#[tauri::command]
pub fn task_label_map(state: State<AppState>, business_id: i64) -> Result<Vec<TaskLabel>> {
    let conn = state.db.lock().unwrap();
    repo::label::map_for_business(&conn, business_id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliverableCreate {
    pub business_id: i64,
    pub project_id: Option<i64>,
    pub title: String,
    pub kind: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliverableVersionAdd {
    pub id: i64,
    pub note: Option<String>,
    pub file_path: Option<String>,
}

#[tauri::command]
pub fn deliverable_list(state: State<AppState>, business_id: i64) -> Result<Vec<Deliverable>> {
    let conn = state.db.lock().unwrap();
    repo::deliverable::list_by_business(&conn, business_id)
}

#[tauri::command]
pub fn deliverable_create(state: State<AppState>, input: DeliverableCreate) -> Result<Deliverable> {
    let conn = state.db.lock().unwrap();
    repo::deliverable::create(&conn, input.business_id, input.project_id, &input.title, &input.kind)
}

#[tauri::command]
pub fn deliverable_set_status(state: State<AppState>, id: i64, status: String) -> Result<Deliverable> {
    let conn = state.db.lock().unwrap();
    repo::deliverable::update_status(&conn, id, &status)
}

#[tauri::command]
pub fn deliverable_add_version(state: State<AppState>, input: DeliverableVersionAdd) -> Result<Deliverable> {
    let conn = state.db.lock().unwrap();
    repo::deliverable::add_version(&conn, input.id, input.note.as_deref(), input.file_path.as_deref())
}

#[tauri::command]
pub fn deliverable_versions(state: State<AppState>, deliverable_id: i64) -> Result<Vec<DeliverableVersion>> {
    let conn = state.db.lock().unwrap();
    repo::deliverable::list_versions(&conn, deliverable_id)
}

#[tauri::command]
pub fn deliverable_archive(state: State<AppState>, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::deliverable::archive(&conn, id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCreate {
    pub business_id: i64,
    pub project_id: Option<i64>,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub auth_type: String,
    pub key_path: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerUpdate {
    pub id: i64,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub auth_type: String,
    pub key_path: Option<String>,
}

#[tauri::command]
pub fn server_list(state: State<AppState>, business_id: i64) -> Result<Vec<ServerConnection>> {
    let conn = state.db.lock().unwrap();
    repo::server::list_by_business(&conn, business_id)
}

#[tauri::command]
pub fn server_create(state: State<AppState>, input: ServerCreate) -> Result<ServerConnection> {
    let conn = state.db.lock().unwrap();
    repo::server::create(
        &conn,
        input.business_id,
        input.project_id,
        &input.name,
        &input.host,
        input.port,
        &input.username,
        &input.auth_type,
        input.key_path.as_deref(),
    )
}

#[tauri::command]
pub fn server_update(state: State<AppState>, input: ServerUpdate) -> Result<ServerConnection> {
    let conn = state.db.lock().unwrap();
    repo::server::update(
        &conn,
        input.id,
        &input.name,
        &input.host,
        input.port,
        &input.username,
        &input.auth_type,
        input.key_path.as_deref(),
    )
}

#[tauri::command]
pub fn server_archive(state: State<AppState>, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    // 보관 시 키체인 시크릿도 제거
    let _ = secrets::delete(&secrets::ref_for_server(id));
    repo::server::archive(&conn, id)
}

/// 서버 비밀값(비밀번호/패스프레이즈)을 OS 키체인에 저장. DB엔 참조 키만 기록.
#[tauri::command]
pub fn server_set_secret(state: State<AppState>, id: i64, secret: String) -> Result<()> {
    let secret_ref = secrets::ref_for_server(id);
    secrets::set(&secret_ref, &secret)?;
    let conn = state.db.lock().unwrap();
    repo::server::set_secret_ref(&conn, id, Some(&secret_ref))
}

/// 키체인에 저장된 비밀값 제거 + 참조 해제.
#[tauri::command]
pub fn server_clear_secret(state: State<AppState>, id: i64) -> Result<()> {
    secrets::delete(&secrets::ref_for_server(id))?;
    let conn = state.db.lock().unwrap();
    repo::server::set_secret_ref(&conn, id, None)
}

/// 비밀값 저장 여부(참조 키 존재).
#[tauri::command]
pub fn server_has_secret(state: State<AppState>, id: i64) -> Result<bool> {
    let conn = state.db.lock().unwrap();
    Ok(repo::server::get(&conn, id)?.secret_ref.is_some())
}

// ---- SSH 터미널 ----

#[tauri::command]
pub fn ssh_connect(
    app: tauri::AppHandle,
    state: State<AppState>,
    term: State<crate::terminal::TerminalManager>,
    id: i64,
) -> Result<()> {
    let server = {
        let conn = state.db.lock().unwrap();
        repo::server::get(&conn, id)?
    };
    crate::terminal::connect(&app, &term, &server)?;
    let conn = state.db.lock().unwrap();
    let _ = repo::server::touch_last_used(&conn, id);
    Ok(())
}

#[tauri::command]
pub fn ssh_write(term: State<crate::terminal::TerminalManager>, id: i64, data: String) -> Result<()> {
    crate::terminal::write(&term, id, &data)
}

#[tauri::command]
pub fn ssh_resize(
    term: State<crate::terminal::TerminalManager>,
    id: i64,
    rows: u16,
    cols: u16,
) -> Result<()> {
    crate::terminal::resize(&term, id, rows, cols)
}

#[tauri::command]
pub fn ssh_disconnect(term: State<crate::terminal::TerminalManager>, id: i64) -> Result<()> {
    crate::terminal::disconnect(&term, id)
}

#[tauri::command]
pub fn search(state: State<AppState>, query: String) -> Result<Vec<SearchHit>> {
    let conn = state.db.lock().unwrap();
    repo::search::search(&conn, &query)
}

#[tauri::command]
pub fn trash_list(state: State<AppState>) -> Result<Vec<TrashItem>> {
    let conn = state.db.lock().unwrap();
    repo::trash::list_archived(&conn)
}

#[tauri::command]
pub fn trash_restore(state: State<AppState>, kind: String, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::trash::restore(&conn, &kind, id)
}

#[tauri::command]
pub fn trash_purge(state: State<AppState>, kind: String, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::trash::purge(&conn, &kind, id)
}

/// 전체 데이터를 JSON으로 내보낸다. path 미지정 시 앱 데이터 폴더의 backup.json.
/// 저장한 경로를 반환.
#[tauri::command]
pub fn export_json(
    app: tauri::AppHandle,
    state: State<AppState>,
    path: Option<String>,
) -> Result<String> {
    use crate::error::AppError;
    use tauri::Manager;

    let target = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let dir = app
                .path()
                .app_data_dir()
                .map_err(|e| AppError::Invalid(format!("앱 데이터 경로 오류: {e}")))?;
            std::fs::create_dir_all(&dir).ok();
            dir.join("backup.json")
        }
    };

    let data = {
        let conn = state.db.lock().unwrap();
        crate::export::export_data(&conn)?
    };
    let pretty =
        serde_json::to_string_pretty(&data).map_err(|e| AppError::Invalid(e.to_string()))?;
    std::fs::write(&target, pretty.as_bytes())
        .map_err(|e| AppError::Invalid(format!("파일 쓰기 실패: {e}")))?;
    Ok(target.to_string_lossy().to_string())
}
