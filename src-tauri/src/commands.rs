use crate::error::Result;
use crate::models::{
    Block, Business, CommandSnippet, Deliverable, DeliverableVersion, Document, Folder, Label,
    Memo, Project, RecurringTask, SearchHit, ServerConnection, Task, TaskLabel, Template, TrashItem,
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
    pub folder_id: Option<i64>,
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
    repo::document::create(&conn, input.business_id, input.project_id, input.folder_id, &input.title)
}

#[tauri::command]
pub fn document_move(state: State<AppState>, id: i64, folder_id: Option<i64>) -> Result<Document> {
    let conn = state.db.lock().unwrap();
    repo::document::set_folder(&conn, id, folder_id)
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
pub fn document_get(state: State<AppState>, id: i64) -> Result<Document> {
    let conn = state.db.lock().unwrap();
    repo::document::get(&conn, id)
}

#[tauri::command]
pub fn document_set_body(state: State<AppState>, id: i64, body: String) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::document::set_body(&conn, id, &body)
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
    pub folder_id: Option<i64>,
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
    repo::deliverable::create(&conn, input.business_id, input.project_id, input.folder_id, &input.title, &input.kind)
}

#[tauri::command]
pub fn deliverable_move(state: State<AppState>, id: i64, folder_id: Option<i64>) -> Result<Deliverable> {
    let conn = state.db.lock().unwrap();
    repo::deliverable::set_folder(&conn, id, folder_id)
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

/// 파일 업로드(다중). 프론트가 다이얼로그로 고른 경로들을 받아 앱 데이터 폴더로 복사하고
/// 각각 산출물 행을 생성한다. 개별 파일 실패는 건너뛰고 성공분만 반환한다.
#[tauri::command]
pub fn deliverable_upload(
    app: tauri::AppHandle,
    state: State<AppState>,
    business_id: i64,
    project_id: Option<i64>,
    folder_id: Option<i64>,
    paths: Vec<String>,
) -> Result<Vec<Deliverable>> {
    use tauri::Manager;
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| crate::error::AppError::Invalid("앱 데이터 폴더를 찾을 수 없음".into()))?;
    let conn = state.db.lock().unwrap();
    let mut created = Vec::new();
    for path in paths {
        let src = std::path::Path::new(&path);
        let filename = match src.file_name().and_then(|f| f.to_str()) {
            Some(f) if !f.is_empty() => f.to_string(),
            _ => continue,
        };
        let size = match std::fs::metadata(src) {
            Ok(m) if m.is_file() => m.len() as i64,
            _ => continue,
        };
        // 1) 행 먼저 생성해 id 확보
        let d = match repo::deliverable::create_file(&conn, business_id, project_id, folder_id, &filename, &filename, size) {
            Ok(d) => d,
            Err(_) => continue,
        };
        // 2) <appData>/deliverables/<id>/<filename> 로 복사
        let dest_dir = data_dir.join("deliverables").join(d.id.to_string());
        if std::fs::create_dir_all(&dest_dir).is_err() {
            let _ = repo::deliverable::delete(&conn, d.id);
            continue;
        }
        let dest = dest_dir.join(&filename);
        if std::fs::copy(src, &dest).is_err() {
            let _ = repo::deliverable::delete(&conn, d.id);
            let _ = std::fs::remove_dir(&dest_dir);
            continue;
        }
        // 3) 복사본 경로 기록
        let dest_str = dest.to_string_lossy().to_string();
        if repo::deliverable::set_file_path(&conn, d.id, &dest_str).is_err() {
            continue;
        }
        created.push(repo::deliverable::get(&conn, d.id)?);
    }
    Ok(created)
}

#[tauri::command]
pub fn deliverable_rename(state: State<AppState>, id: i64, title: String) -> Result<Deliverable> {
    let conn = state.db.lock().unwrap();
    repo::deliverable::rename(&conn, id, &title)
}

// ---- 폴더(산출물·문서 분류) ----

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderCreate {
    pub business_id: i64,
    pub kind: String, // document | deliverable
    pub parent_id: Option<i64>,
    pub name: String,
}

#[tauri::command]
pub fn folder_list(state: State<AppState>, business_id: i64) -> Result<Vec<Folder>> {
    let conn = state.db.lock().unwrap();
    repo::folder::list_by_business(&conn, business_id)
}

#[tauri::command]
pub fn folder_create(state: State<AppState>, input: FolderCreate) -> Result<Folder> {
    let conn = state.db.lock().unwrap();
    repo::folder::create(&conn, input.business_id, &input.kind, input.parent_id, &input.name)
}

#[tauri::command]
pub fn folder_rename(state: State<AppState>, id: i64, name: String) -> Result<Folder> {
    let conn = state.db.lock().unwrap();
    repo::folder::rename(&conn, id, &name)
}

#[tauri::command]
pub fn folder_delete(state: State<AppState>, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::folder::delete(&conn, id)
}

// ---- 메모(사업별 Keep식) ----

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoCreate {
    pub business_id: i64,
    pub title: String,
    pub body: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoUpdate {
    pub id: i64,
    pub title: String,
    pub body: String,
}

#[tauri::command]
pub fn memo_list(state: State<AppState>, business_id: i64) -> Result<Vec<Memo>> {
    let conn = state.db.lock().unwrap();
    repo::memo::list_by_business(&conn, business_id)
}

#[tauri::command]
pub fn memo_create(state: State<AppState>, input: MemoCreate) -> Result<Memo> {
    let conn = state.db.lock().unwrap();
    repo::memo::create(&conn, input.business_id, &input.title, &input.body)
}

#[tauri::command]
pub fn memo_update(state: State<AppState>, input: MemoUpdate) -> Result<Memo> {
    let conn = state.db.lock().unwrap();
    repo::memo::update(&conn, input.id, &input.title, &input.body)
}

#[tauri::command]
pub fn memo_set_color(state: State<AppState>, id: i64, color: Option<String>) -> Result<Memo> {
    let conn = state.db.lock().unwrap();
    repo::memo::set_color(&conn, id, color.as_deref())
}

#[tauri::command]
pub fn memo_set_pinned(state: State<AppState>, id: i64, pinned: bool) -> Result<Memo> {
    let conn = state.db.lock().unwrap();
    repo::memo::set_pinned(&conn, id, pinned)
}

#[tauri::command]
pub fn memo_archive(state: State<AppState>, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::memo::archive(&conn, id)
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetCreate {
    pub server_id: i64,
    pub name: String,
    pub command: String,
}

#[tauri::command]
pub fn snippet_list(state: State<AppState>, server_id: i64) -> Result<Vec<CommandSnippet>> {
    let conn = state.db.lock().unwrap();
    repo::snippet::list_by_server(&conn, server_id)
}

#[tauri::command]
pub fn snippet_create(state: State<AppState>, input: SnippetCreate) -> Result<CommandSnippet> {
    let conn = state.db.lock().unwrap();
    repo::snippet::create(&conn, input.server_id, &input.name, &input.command)
}

#[tauri::command]
pub fn snippet_delete(state: State<AppState>, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::snippet::delete(&conn, id)
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

/// SFTP 디렉터리 나열 (키 기반 인증).
#[tauri::command]
pub fn sftp_list(
    app: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
    path: String,
) -> Result<Vec<crate::sftp::SftpEntry>> {
    let server = {
        let conn = state.db.lock().unwrap();
        repo::server::get(&conn, id)?
    };
    let kh = crate::hostkey::known_hosts_path(&app).map(|p| p.to_string_lossy().to_string());
    crate::sftp::list(&server, &path, kh.as_deref())
}

// ---- SSH 호스트 키 신뢰 (지문 확인 후 등록) ----

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostScanResult {
    pub fingerprint: String,
    pub key_lines: String,
}

/// 해당 서버 호스트가 이미 신뢰(known_hosts 등록)되어 있는지.
#[tauri::command]
pub fn ssh_host_status(app: tauri::AppHandle, state: State<AppState>, id: i64) -> Result<bool> {
    let server = {
        let conn = state.db.lock().unwrap();
        repo::server::get(&conn, id)?
    };
    let kh = crate::hostkey::known_hosts_path(&app)
        .ok_or_else(|| crate::error::AppError::Invalid("앱 데이터 경로 오류".into()))?;
    Ok(crate::hostkey::is_known(&kh, &server.host, server.port))
}

/// ssh-keyscan 으로 호스트 공개키·지문을 가져온다(아직 신뢰하지 않음). 사용자 확인용.
#[tauri::command]
pub fn ssh_scan_host(state: State<AppState>, id: i64) -> Result<HostScanResult> {
    let server = {
        let conn = state.db.lock().unwrap();
        repo::server::get(&conn, id)?
    };
    let (fingerprint, key_lines) = crate::hostkey::scan(&server.host, server.port)?;
    Ok(HostScanResult { fingerprint, key_lines })
}

/// 사용자가 지문을 확인·수락한 호스트 키를 앱 known_hosts 에 등록.
#[tauri::command]
pub fn ssh_trust_host(app: tauri::AppHandle, key_lines: String) -> Result<()> {
    let kh = crate::hostkey::known_hosts_path(&app)
        .ok_or_else(|| crate::error::AppError::Invalid("앱 데이터 경로 오류".into()))?;
    crate::hostkey::trust(&kh, &key_lines)
}

// ---- 템플릿 ----

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateCreate {
    pub name: String,
    pub kind: String,
    pub payload: String,
}

#[tauri::command]
pub fn template_list(state: State<AppState>) -> Result<Vec<Template>> {
    let conn = state.db.lock().unwrap();
    repo::template::list(&conn)
}

#[tauri::command]
pub fn template_create(state: State<AppState>, input: TemplateCreate) -> Result<Template> {
    let conn = state.db.lock().unwrap();
    repo::template::create(&conn, &input.name, &input.kind, &input.payload)
}

#[tauri::command]
pub fn template_delete(state: State<AppState>, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::template::delete(&conn, id)
}

#[tauri::command]
pub fn template_apply_project(state: State<AppState>, template_id: i64, business_id: i64) -> Result<i64> {
    let conn = state.db.lock().unwrap();
    repo::template::apply_project(&conn, template_id, business_id)
}

#[tauri::command]
pub fn template_apply_document(
    state: State<AppState>,
    template_id: i64,
    business_id: i64,
    project_id: Option<i64>,
) -> Result<i64> {
    let conn = state.db.lock().unwrap();
    repo::template::apply_document(&conn, template_id, business_id, project_id)
}

// ---- 반복 태스크 ----

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringCreate {
    pub business_id: i64,
    pub project_id: Option<i64>,
    pub title: String,
    pub priority: i64,
    pub interval_days: i64,
    pub next_run: String,
}

#[tauri::command]
pub fn recurring_list(state: State<AppState>, business_id: i64) -> Result<Vec<RecurringTask>> {
    let conn = state.db.lock().unwrap();
    repo::recurring::list_by_business(&conn, business_id)
}

#[tauri::command]
pub fn recurring_create(state: State<AppState>, input: RecurringCreate) -> Result<RecurringTask> {
    let conn = state.db.lock().unwrap();
    repo::recurring::create(
        &conn,
        input.business_id,
        input.project_id,
        &input.title,
        input.priority,
        input.interval_days,
        &input.next_run,
    )
}

#[tauri::command]
pub fn recurring_set_active(state: State<AppState>, id: i64, active: bool) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::recurring::set_active(&conn, id, active)
}

#[tauri::command]
pub fn recurring_delete(state: State<AppState>, id: i64) -> Result<()> {
    let conn = state.db.lock().unwrap();
    repo::recurring::delete(&conn, id)
}

/// today(YYYY-MM-DD) 기준 도래한 반복 태스크 생성. 생성 수 반환.
#[tauri::command]
pub fn recurring_generate(state: State<AppState>, today: String) -> Result<usize> {
    let conn = state.db.lock().unwrap();
    repo::recurring::generate_due(&conn, &today)
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
    // 산출물은 영구삭제 시 복사 보관된 물리 파일도 함께 제거한다.
    let file_path = if kind == "deliverable" {
        repo::deliverable::file_path_of(&conn, id).ok().flatten()
    } else {
        None
    };
    repo::trash::purge(&conn, &kind, id)?;
    if let Some(path) = file_path {
        let p = std::path::Path::new(&path);
        let _ = std::fs::remove_file(p);
        // <appData>/deliverables/<id>/ 폴더도 비었으면 정리
        if let Some(parent) = p.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    }
    Ok(())
}

/// 전체 데이터를 JSON으로 내보낸다. path 미지정 시 앱 데이터 폴더의 backup.json.
/// 저장한 경로를 반환.
///
/// 보안(CWE-22/73): 외부에서 임의 경로가 들어와도 앱 데이터 폴더 밖을 읽고/쓰지 못하도록
/// 제한한다. 정상 흐름은 path=None(앱 데이터의 backup.json)이며, path가 주어지면 앱 데이터
/// 폴더 하위인지 검증한다.
fn resolve_backup_path(app: &tauri::AppHandle, path: Option<String>) -> Result<std::path::PathBuf> {
    use crate::error::AppError;
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Invalid(format!("앱 데이터 경로 오류: {e}")))?;
    std::fs::create_dir_all(&dir).ok();
    let base = std::fs::canonicalize(&dir).unwrap_or(dir);
    match path {
        None => Ok(base.join("backup.json")),
        Some(p) => {
            let target = std::path::PathBuf::from(&p);
            let parent = target.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(&base);
            let parent_canon = std::fs::canonicalize(parent)
                .map_err(|_| AppError::Invalid("백업 경로가 유효하지 않습니다".into()))?;
            if !parent_canon.starts_with(&base) {
                return Err(AppError::Invalid("백업 경로는 앱 데이터 폴더 안이어야 합니다".into()));
            }
            let name = target.file_name().ok_or_else(|| AppError::Invalid("백업 파일명이 필요합니다".into()))?;
            Ok(parent_canon.join(name))
        }
    }
}

#[tauri::command]
pub fn export_json(
    app: tauri::AppHandle,
    state: State<AppState>,
    path: Option<String>,
) -> Result<String> {
    use crate::error::AppError;

    let target = resolve_backup_path(&app, path)?;

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

/// JSON 백업 파일을 현재 DB에 추가 가져오기. path 미지정 시 앱 데이터 폴더의 backup.json.
#[tauri::command]
pub fn import_json(
    app: tauri::AppHandle,
    state: State<AppState>,
    path: Option<String>,
) -> Result<()> {
    use crate::error::AppError;

    let target = resolve_backup_path(&app, path)?;
    let text = std::fs::read_to_string(&target)
        .map_err(|e| AppError::Invalid(format!("파일 읽기 실패: {e}")))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| AppError::Invalid(format!("JSON 파싱 실패: {e}")))?;
    let conn = state.db.lock().unwrap();
    crate::export::import_data(&conn, &value)
}
