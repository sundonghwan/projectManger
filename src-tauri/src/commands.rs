use crate::error::Result;
use crate::secrets;
use crate::store::model::{
    Block, Business, Deliverable, DeliverableVersion, Document, Folder, Label, Memo, Project,
    RecurringTask, Server, Snippet, Task, Template,
};
use crate::store::ops;
use crate::store::ops::label::TaskLabelView;
use crate::store::ops::search::SearchHit;
use crate::store::ops::trash::TrashItem;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::State;

/// 앱 전역 상태 — vault 데이터용 파일 Store와, SSH 등 로컬 전용 LocalStore.
pub struct AppState {
    pub store: Mutex<crate::store::Store>,
    pub local: Mutex<crate::store::local::LocalStore>,
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
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub status: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[tauri::command]
pub fn business_list(state: State<AppState>) -> Result<Vec<Business>> {
    let store = state.store.lock().unwrap();
    ops::business::list(&store)
}

#[tauri::command]
pub fn business_create(state: State<AppState>, input: BusinessCreate) -> Result<Business> {
    let mut store = state.store.lock().unwrap();
    ops::business::create(&mut store, &input.name, &input.type_, input.color.as_deref())
}

#[tauri::command]
pub fn business_update(state: State<AppState>, input: BusinessUpdate) -> Result<Business> {
    let mut store = state.store.lock().unwrap();
    ops::business::update(
        &mut store,
        &input.id,
        &input.name,
        &input.type_,
        &input.status,
        input.color.as_deref(),
        input.description.as_deref(),
    )
}

#[tauri::command]
pub fn business_rename(state: State<AppState>, id: String, name: String) -> Result<Business> {
    let mut store = state.store.lock().unwrap();
    ops::business::rename(&mut store, &id, &name)
}

#[tauri::command]
pub fn business_archive(state: State<AppState>, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::business::archive(&mut store, &id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCreate {
    pub business_id: String,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdate {
    pub id: String,
    pub name: String,
    pub status: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
}

#[tauri::command]
pub fn project_list(state: State<AppState>, business_id: String) -> Result<Vec<Project>> {
    let store = state.store.lock().unwrap();
    ops::project::list_by_business(&store, &business_id)
}

#[tauri::command]
pub fn project_create(state: State<AppState>, input: ProjectCreate) -> Result<Project> {
    let mut store = state.store.lock().unwrap();
    ops::project::create(&mut store, &input.business_id, &input.name)
}

#[tauri::command]
pub fn project_update(state: State<AppState>, input: ProjectUpdate) -> Result<Project> {
    let mut store = state.store.lock().unwrap();
    ops::project::update(
        &mut store,
        &input.id,
        &input.name,
        &input.status,
        input.description.as_deref(),
        input.due_date.as_deref(),
    )
}

#[tauri::command]
pub fn project_rename(state: State<AppState>, id: String, name: String) -> Result<Project> {
    let mut store = state.store.lock().unwrap();
    ops::project::rename(&mut store, &id, &name)
}

#[tauri::command]
pub fn project_archive(state: State<AppState>, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::project::archive(&mut store, &id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCreate {
    pub business_id: String,
    pub project_id: Option<String>,
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
    pub id: String,
    pub title: String,
    pub priority: i64,
    pub due_date: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskMove {
    pub id: String,
    pub status: String,
    pub sort_order: f64,
}

#[tauri::command]
pub fn task_list(
    state: State<AppState>,
    business_id: String,
    project_id: Option<String>,
) -> Result<Vec<Task>> {
    let store = state.store.lock().unwrap();
    ops::task::list(&store, &business_id, project_id.as_deref())
}

#[tauri::command]
pub fn task_create(state: State<AppState>, input: TaskCreate) -> Result<Task> {
    let mut store = state.store.lock().unwrap();
    ops::task::create(&mut store, &input.business_id, input.project_id.as_deref(), &input.title, input.priority)
}

#[tauri::command]
pub fn task_update(state: State<AppState>, input: TaskUpdate) -> Result<Task> {
    let mut store = state.store.lock().unwrap();
    ops::task::update(
        &mut store,
        &input.id,
        &input.title,
        input.priority,
        input.due_date.as_deref(),
        input.description.as_deref(),
    )
}

#[tauri::command]
pub fn task_move(state: State<AppState>, input: TaskMove) -> Result<Task> {
    let mut store = state.store.lock().unwrap();
    ops::task::move_task(&mut store, &input.id, &input.status, input.sort_order)
}

#[tauri::command]
pub fn task_archive(state: State<AppState>, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::task::archive(&mut store, &id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentCreate {
    pub business_id: String,
    pub project_id: Option<String>,
    pub folder_id: Option<String>,
    pub title: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockCreate {
    pub document_id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub content: String,
    pub sort_order: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockUpdate {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub content: String,
}

#[tauri::command]
pub fn document_list(state: State<AppState>, business_id: String) -> Result<Vec<Document>> {
    let store = state.store.lock().unwrap();
    ops::document::list_by_business(&store, &business_id)
}

#[tauri::command]
pub fn document_create(state: State<AppState>, input: DocumentCreate) -> Result<Document> {
    let mut store = state.store.lock().unwrap();
    ops::document::create(&mut store, &input.business_id, input.project_id.as_deref(), input.folder_id.as_deref(), &input.title)
}

#[tauri::command]
pub fn document_move(state: State<AppState>, id: String, folder_id: Option<String>) -> Result<Document> {
    let mut store = state.store.lock().unwrap();
    ops::document::set_folder(&mut store, &id, folder_id.as_deref())
}

#[tauri::command]
pub fn document_rename(state: State<AppState>, id: String, title: String) -> Result<Document> {
    let mut store = state.store.lock().unwrap();
    ops::document::rename(&mut store, &id, &title)
}

#[tauri::command]
pub fn document_archive(state: State<AppState>, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::document::archive(&mut store, &id)
}

#[tauri::command]
pub fn document_get(state: State<AppState>, id: String) -> Result<Document> {
    let store = state.store.lock().unwrap();
    ops::document::get(&store, &id)
}

#[tauri::command]
pub fn document_set_body(state: State<AppState>, id: String, body: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::document::set_body(&mut store, &id, &body)
}

#[tauri::command]
pub fn block_list(state: State<AppState>, document_id: String) -> Result<Vec<Block>> {
    let store = state.store.lock().unwrap();
    ops::document::list_blocks(&store, &document_id)
}

#[tauri::command]
pub fn block_create(state: State<AppState>, input: BlockCreate) -> Result<Block> {
    let mut store = state.store.lock().unwrap();
    ops::document::create_block(&mut store, &input.document_id, &input.type_, &input.content, input.sort_order)
}

#[tauri::command]
pub fn block_update(state: State<AppState>, input: BlockUpdate) -> Result<Block> {
    let mut store = state.store.lock().unwrap();
    ops::document::update_block(&mut store, &input.id, &input.type_, &input.content)
}

#[tauri::command]
pub fn block_delete(state: State<AppState>, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::document::delete_block(&mut store, &id)
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
    pub task_id: String,
    pub label_id: String,
}

#[tauri::command]
pub fn label_list(state: State<AppState>) -> Result<Vec<Label>> {
    let store = state.store.lock().unwrap();
    ops::label::list(&store)
}

#[tauri::command]
pub fn label_create(state: State<AppState>, input: LabelCreate) -> Result<Label> {
    let mut store = state.store.lock().unwrap();
    ops::label::create(&mut store, &input.name, input.color.as_deref())
}

#[tauri::command]
pub fn label_assign(state: State<AppState>, input: LabelAssign) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::label::assign(&mut store, &input.task_id, &input.label_id)
}

#[tauri::command]
pub fn label_unassign(state: State<AppState>, input: LabelAssign) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::label::unassign(&mut store, &input.task_id, &input.label_id)
}

#[tauri::command]
pub fn task_label_map(state: State<AppState>, business_id: String) -> Result<Vec<TaskLabelView>> {
    let store = state.store.lock().unwrap();
    ops::label::map_for_business(&store, &business_id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliverableCreate {
    pub business_id: String,
    pub project_id: Option<String>,
    pub folder_id: Option<String>,
    pub title: String,
    pub kind: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliverableVersionAdd {
    pub id: String,
    pub note: Option<String>,
    pub file_path: Option<String>,
}

#[tauri::command]
pub fn deliverable_list(state: State<AppState>, business_id: String) -> Result<Vec<Deliverable>> {
    let store = state.store.lock().unwrap();
    ops::deliverable::list_by_business(&store, &business_id)
}

#[tauri::command]
pub fn deliverable_create(state: State<AppState>, input: DeliverableCreate) -> Result<Deliverable> {
    let mut store = state.store.lock().unwrap();
    ops::deliverable::create(&mut store, &input.business_id, input.project_id.as_deref(), input.folder_id.as_deref(), &input.title, &input.kind)
}

#[tauri::command]
pub fn deliverable_move(state: State<AppState>, id: String, folder_id: Option<String>) -> Result<Deliverable> {
    let mut store = state.store.lock().unwrap();
    ops::deliverable::set_folder(&mut store, &id, folder_id.as_deref())
}

#[tauri::command]
pub fn deliverable_set_status(state: State<AppState>, id: String, status: String) -> Result<Deliverable> {
    let mut store = state.store.lock().unwrap();
    ops::deliverable::update_status(&mut store, &id, &status)
}

#[tauri::command]
pub fn deliverable_add_version(state: State<AppState>, input: DeliverableVersionAdd) -> Result<Deliverable> {
    let mut store = state.store.lock().unwrap();
    ops::deliverable::add_version(&mut store, &input.id, input.note.as_deref(), input.file_path.as_deref())
}

#[tauri::command]
pub fn deliverable_versions(state: State<AppState>, deliverable_id: String) -> Result<Vec<DeliverableVersion>> {
    let store = state.store.lock().unwrap();
    ops::deliverable::list_versions(&store, &deliverable_id)
}

#[tauri::command]
pub fn deliverable_archive(state: State<AppState>, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::deliverable::archive(&mut store, &id)
}

/// 파일 업로드(다중). 프론트가 다이얼로그로 고른 경로들을 받아 앱 데이터 폴더로 복사하고
/// 각각 산출물 행을 생성한다. 개별 파일 실패는 건너뛰고 성공분만 반환한다.
#[tauri::command]
pub fn deliverable_upload(
    app: tauri::AppHandle,
    state: State<AppState>,
    business_id: String,
    project_id: Option<String>,
    folder_id: Option<String>,
    paths: Vec<String>,
) -> Result<Vec<Deliverable>> {
    use tauri::Manager;
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| crate::error::AppError::Invalid("앱 데이터 폴더를 찾을 수 없음".into()))?;
    let mut store = state.store.lock().unwrap();
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
        let d = match ops::deliverable::create_file(
            &mut store,
            &business_id,
            project_id.as_deref(),
            folder_id.as_deref(),
            &filename,
            &filename,
            size,
        ) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let dest_dir = data_dir.join("deliverables").join(&d.id);
        if std::fs::create_dir_all(&dest_dir).is_err() {
            let _ = ops::deliverable::delete(&mut store, &d.id);
            continue;
        }
        let dest = dest_dir.join(&filename);
        if std::fs::copy(src, &dest).is_err() {
            let _ = ops::deliverable::delete(&mut store, &d.id);
            let _ = std::fs::remove_dir(&dest_dir);
            continue;
        }
        let dest_str = dest.to_string_lossy().to_string();
        if ops::deliverable::set_file_path(&mut store, &d.id, &dest_str).is_err() {
            continue;
        }
        created.push(ops::deliverable::get(&store, &d.id)?);
    }
    Ok(created)
}

#[tauri::command]
pub fn deliverable_rename(state: State<AppState>, id: String, title: String) -> Result<Deliverable> {
    let mut store = state.store.lock().unwrap();
    ops::deliverable::rename(&mut store, &id, &title)
}

// ---- 폴더(산출물·문서 분류) ----

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderCreate {
    pub business_id: String,
    pub kind: String, // document | deliverable
    pub parent_id: Option<String>,
    pub name: String,
}

#[tauri::command]
pub fn folder_list(state: State<AppState>, business_id: String) -> Result<Vec<Folder>> {
    let store = state.store.lock().unwrap();
    ops::folder::list_by_business(&store, &business_id)
}

#[tauri::command]
pub fn folder_create(state: State<AppState>, input: FolderCreate) -> Result<Folder> {
    let mut store = state.store.lock().unwrap();
    ops::folder::create(&mut store, &input.business_id, &input.kind, input.parent_id.as_deref(), &input.name)
}

#[tauri::command]
pub fn folder_rename(state: State<AppState>, id: String, name: String) -> Result<Folder> {
    let mut store = state.store.lock().unwrap();
    ops::folder::rename(&mut store, &id, &name)
}

#[tauri::command]
pub fn folder_delete(state: State<AppState>, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::folder::delete(&mut store, &id)
}

// ---- 메모(사업별 Keep식) ----

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoCreate {
    pub business_id: String,
    pub title: String,
    pub body: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoUpdate {
    pub id: String,
    pub title: String,
    pub body: String,
}

#[tauri::command]
pub fn memo_list(state: State<AppState>, business_id: String) -> Result<Vec<Memo>> {
    let store = state.store.lock().unwrap();
    ops::memo::list_by_business(&store, &business_id)
}

#[tauri::command]
pub fn memo_create(state: State<AppState>, input: MemoCreate) -> Result<Memo> {
    let mut store = state.store.lock().unwrap();
    ops::memo::create(&mut store, &input.business_id, &input.title, &input.body)
}

#[tauri::command]
pub fn memo_update(state: State<AppState>, input: MemoUpdate) -> Result<Memo> {
    let mut store = state.store.lock().unwrap();
    ops::memo::update(&mut store, &input.id, &input.title, &input.body)
}

#[tauri::command]
pub fn memo_set_color(state: State<AppState>, id: String, color: Option<String>) -> Result<Memo> {
    let mut store = state.store.lock().unwrap();
    ops::memo::set_color(&mut store, &id, color.as_deref())
}

#[tauri::command]
pub fn memo_set_pinned(state: State<AppState>, id: String, pinned: bool) -> Result<Memo> {
    let mut store = state.store.lock().unwrap();
    ops::memo::set_pinned(&mut store, &id, pinned)
}

#[tauri::command]
pub fn memo_archive(state: State<AppState>, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::memo::archive(&mut store, &id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCreate {
    pub business_id: String,
    pub project_id: Option<String>,
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
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub auth_type: String,
    pub key_path: Option<String>,
}

#[tauri::command]
pub fn server_list(state: State<AppState>, business_id: String) -> Result<Vec<Server>> {
    let local = state.local.lock().unwrap();
    ops::server::list_by_business(&local, &business_id)
}

#[tauri::command]
pub fn server_create(state: State<AppState>, input: ServerCreate) -> Result<Server> {
    let mut local = state.local.lock().unwrap();
    ops::server::create(
        &mut local,
        &input.business_id,
        input.project_id.as_deref(),
        &input.name,
        &input.host,
        input.port,
        &input.username,
        &input.auth_type,
        input.key_path.as_deref(),
    )
}

#[tauri::command]
pub fn server_update(state: State<AppState>, input: ServerUpdate) -> Result<Server> {
    let mut local = state.local.lock().unwrap();
    ops::server::update(
        &mut local,
        &input.id,
        &input.name,
        &input.host,
        input.port,
        &input.username,
        &input.auth_type,
        input.key_path.as_deref(),
    )
}

#[tauri::command]
pub fn server_archive(state: State<AppState>, id: String) -> Result<()> {
    // 보관 시 키체인 시크릿도 제거
    let _ = secrets::delete(&secrets::ref_for_server(&id));
    let mut local = state.local.lock().unwrap();
    ops::server::archive(&mut local, &id)
}

/// 서버 비밀값(비밀번호/패스프레이즈)을 OS 키체인에 저장. LocalStore엔 참조 키만 기록.
#[tauri::command]
pub fn server_set_secret(state: State<AppState>, id: String, secret: String) -> Result<()> {
    let r = secrets::ref_for_server(&id);
    secrets::set(&r, &secret)?;
    let mut local = state.local.lock().unwrap();
    ops::server::set_secret_ref(&mut local, &id, Some(&r))
}

/// 키체인에 저장된 비밀값 제거 + 참조 해제.
#[tauri::command]
pub fn server_clear_secret(state: State<AppState>, id: String) -> Result<()> {
    secrets::delete(&secrets::ref_for_server(&id))?;
    let mut local = state.local.lock().unwrap();
    ops::server::set_secret_ref(&mut local, &id, None)
}

/// 비밀값 저장 여부(참조 키 존재).
#[tauri::command]
pub fn server_has_secret(state: State<AppState>, id: String) -> Result<bool> {
    let local = state.local.lock().unwrap();
    Ok(ops::server::get(&local, &id)?.secret_ref.is_some())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetCreate {
    pub server_id: String,
    pub name: String,
    pub command: String,
}

#[tauri::command]
pub fn snippet_list(state: State<AppState>, server_id: String) -> Result<Vec<Snippet>> {
    let local = state.local.lock().unwrap();
    ops::snippet::list_by_server(&local, &server_id)
}

#[tauri::command]
pub fn snippet_create(state: State<AppState>, input: SnippetCreate) -> Result<Snippet> {
    let mut local = state.local.lock().unwrap();
    ops::snippet::create(&mut local, &input.server_id, &input.name, &input.command)
}

#[tauri::command]
pub fn snippet_delete(state: State<AppState>, id: String) -> Result<()> {
    let mut local = state.local.lock().unwrap();
    ops::snippet::delete(&mut local, &id)
}

// ---- SSH 터미널 ----

#[tauri::command]
pub fn ssh_connect(
    app: tauri::AppHandle,
    state: State<AppState>,
    term: State<crate::terminal::TerminalManager>,
    id: String,
) -> Result<()> {
    let server = {
        let local = state.local.lock().unwrap();
        ops::server::get(&local, &id)?
    };
    crate::terminal::connect(&app, &term, &server)?;
    let mut local = state.local.lock().unwrap();
    let _ = ops::server::touch_last_used(&mut local, &id);
    Ok(())
}

#[tauri::command]
pub fn ssh_write(term: State<crate::terminal::TerminalManager>, id: String, data: String) -> Result<()> {
    crate::terminal::write(&term, &id, &data)
}

#[tauri::command]
pub fn ssh_resize(
    term: State<crate::terminal::TerminalManager>,
    id: String,
    rows: u16,
    cols: u16,
) -> Result<()> {
    crate::terminal::resize(&term, &id, rows, cols)
}

#[tauri::command]
pub fn ssh_disconnect(term: State<crate::terminal::TerminalManager>, id: String) -> Result<()> {
    crate::terminal::disconnect(&term, &id)
}

/// SFTP 디렉터리 나열 (키 기반 인증).
#[tauri::command]
pub fn sftp_list(
    app: tauri::AppHandle,
    state: State<AppState>,
    id: String,
    path: String,
) -> Result<Vec<crate::sftp::SftpEntry>> {
    let server = {
        let local = state.local.lock().unwrap();
        ops::server::get(&local, &id)?
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
pub fn ssh_host_status(app: tauri::AppHandle, state: State<AppState>, id: String) -> Result<bool> {
    let server = {
        let local = state.local.lock().unwrap();
        ops::server::get(&local, &id)?
    };
    let kh = crate::hostkey::known_hosts_path(&app)
        .ok_or_else(|| crate::error::AppError::Invalid("앱 데이터 경로 오류".into()))?;
    Ok(crate::hostkey::is_known(&kh, &server.host, server.port))
}

/// ssh-keyscan 으로 호스트 공개키·지문을 가져온다(아직 신뢰하지 않음). 사용자 확인용.
#[tauri::command]
pub fn ssh_scan_host(state: State<AppState>, id: String) -> Result<HostScanResult> {
    let server = {
        let local = state.local.lock().unwrap();
        ops::server::get(&local, &id)?
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
    let store = state.store.lock().unwrap();
    ops::template::list(&store)
}

#[tauri::command]
pub fn template_create(state: State<AppState>, input: TemplateCreate) -> Result<Template> {
    let mut store = state.store.lock().unwrap();
    ops::template::create(&mut store, &input.name, &input.kind, &input.payload)
}

#[tauri::command]
pub fn template_delete(state: State<AppState>, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::template::delete(&mut store, &id)
}

#[tauri::command]
pub fn template_apply_project(state: State<AppState>, template_id: String, business_id: String) -> Result<String> {
    let mut store = state.store.lock().unwrap();
    ops::template::apply_project(&mut store, &template_id, &business_id)
}

#[tauri::command]
pub fn template_apply_document(
    state: State<AppState>,
    template_id: String,
    business_id: String,
    project_id: Option<String>,
) -> Result<String> {
    let mut store = state.store.lock().unwrap();
    ops::template::apply_document(&mut store, &template_id, &business_id, project_id.as_deref())
}

// ---- 반복 태스크 ----

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringCreate {
    pub business_id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub priority: i64,
    pub interval_days: i64,
    pub next_run: String,
}

#[tauri::command]
pub fn recurring_list(state: State<AppState>, business_id: String) -> Result<Vec<RecurringTask>> {
    let store = state.store.lock().unwrap();
    ops::recurring::list_by_business(&store, &business_id)
}

#[tauri::command]
pub fn recurring_create(state: State<AppState>, input: RecurringCreate) -> Result<RecurringTask> {
    let mut store = state.store.lock().unwrap();
    ops::recurring::create(
        &mut store,
        &input.business_id,
        input.project_id.as_deref(),
        &input.title,
        input.priority,
        input.interval_days,
        &input.next_run,
    )
}

#[tauri::command]
pub fn recurring_set_active(state: State<AppState>, id: String, active: bool) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::recurring::set_active(&mut store, &id, active)
}

#[tauri::command]
pub fn recurring_delete(state: State<AppState>, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::recurring::delete(&mut store, &id)
}

/// today(YYYY-MM-DD) 기준 도래한 반복 태스크 생성. 생성 수 반환.
#[tauri::command]
pub fn recurring_generate(state: State<AppState>, today: String) -> Result<usize> {
    let mut store = state.store.lock().unwrap();
    ops::recurring::generate_due(&mut store, &today)
}

#[tauri::command]
pub fn search(state: State<AppState>, query: String) -> Result<Vec<SearchHit>> {
    let store = state.store.lock().unwrap();
    ops::search::search(&store, &query)
}

#[tauri::command]
pub fn trash_list(state: State<AppState>) -> Result<Vec<TrashItem>> {
    let store = state.store.lock().unwrap();
    ops::trash::list_archived(&store)
}

#[tauri::command]
pub fn trash_restore(state: State<AppState>, kind: String, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::trash::restore(&mut store, &kind, &id)
}

#[tauri::command]
pub fn trash_purge(state: State<AppState>, kind: String, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    // 산출물은 영구삭제 시 복사 보관된 물리 파일도 함께 제거한다.
    let file_path = if kind == "deliverable" {
        ops::deliverable::file_path_of(&store, &id).ok().flatten()
    } else {
        None
    };
    ops::trash::purge(&mut store, &kind, &id)?;
    if let Some(path) = file_path {
        let p = std::path::Path::new(&path);
        let _ = std::fs::remove_file(p);
        if let Some(parent) = p.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    }
    Ok(())
}

