use crate::error::{AppError, Result};
use crate::secrets;
use crate::store::model::{
    Block, Business, Deliverable, DeliverableVersion, Document, Folder, Label, Memo, Project,
    RecurringTask, Server, Snippet, Task, Template,
};
use crate::store::ops;
use crate::store::ops::label::TaskLabelView;
use crate::store::ops::search::SearchHit;
use crate::store::ops::trash::TrashItem;
use crate::store::Store;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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
    pub start_date: Option<String>,
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
        input.start_date.as_deref(),
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentAsset {
    pub id: String,
    pub document_id: String,
    pub file_name: String,
    pub file_path: String,
    pub url: String,
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
pub fn document_set_editor_body(
    state: State<AppState>,
    id: String,
    body: String,
    editor_body: String,
    editor_body_format: String,
    collaboration_state: Option<String>,
) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::document::set_editor_body(
        &mut store,
        &id,
        &body,
        Some(&editor_body),
        Some(&editor_body_format),
        collaboration_state.as_deref(),
    )
}

#[tauri::command]
pub fn document_asset_upload(
    state: State<AppState>,
    document_id: String,
    file_name: String,
    bytes: Vec<u8>,
) -> Result<DocumentAsset> {
    let store = state.store.lock().unwrap();
    let document = ops::document::get(&store, &document_id)?;
    let file_name = Path::new(&file_name)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::Invalid("이미지 파일 이름을 확인할 수 없습니다".into()))?
        .to_string();

    if !is_supported_document_image(&file_name) {
        return Err(AppError::Invalid("지원하지 않는 이미지 형식입니다".into()));
    }
    if bytes.is_empty() {
        return Err(AppError::Invalid("빈 이미지 파일은 업로드할 수 없습니다".into()));
    }

    let asset_id = crate::store::ids::new_id();
    let dest_dir = document_asset_files_root(&store.root, &document.id).join(&asset_id);
    std::fs::create_dir_all(&dest_dir)
        .map_err(|_| AppError::Invalid("문서 이미지 폴더를 만들 수 없습니다".into()))?;
    let dest = dest_dir.join(&file_name);
    std::fs::write(&dest, bytes)
        .map_err(|_| AppError::Invalid("문서 이미지를 저장할 수 없습니다".into()))?;

    let file_path = dest.to_string_lossy().to_string();
    Ok(DocumentAsset {
        id: asset_id,
        document_id,
        file_name,
        url: file_path.clone(),
        file_path,
    })
}

#[tauri::command]
pub fn document_show_export_folder(app: tauri::AppHandle, state: State<AppState>, id: String) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;

    let exported = {
        let store = state.store.lock().unwrap();
        let document = ops::document::get(&store, &id)?;
        export_document_markdown(&store.root, &document)?
    };
    if !exported.markdown_path.is_file() {
        return Err(AppError::Invalid("문서 Markdown 파일을 찾을 수 없습니다".into()));
    }
    app.opener()
        .open_path(exported.folder_path.to_string_lossy().to_string(), None::<String>)
        .map_err(|_| AppError::Invalid("문서 export 폴더를 열 수 없습니다".into()))
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

fn deliverable_files_root(store_root: &Path) -> PathBuf {
    store_root.join("files").join("deliverables")
}

fn document_asset_files_root(store_root: &Path, document_id: &str) -> PathBuf {
    store_root.join("files").join("documents").join(document_id).join("assets")
}

fn document_export_current_root(store_root: &Path, document_id: &str) -> PathBuf {
    store_root.join("files").join("documents").join(document_id).join("exports").join("current")
}

pub(crate) struct DocumentExportPaths {
    pub folder_path: PathBuf,
    pub markdown_path: PathBuf,
}

pub(crate) fn export_document_markdown(store_root: &Path, document: &Document) -> Result<DocumentExportPaths> {
    let folder_path = document_export_current_root(store_root, &document.id);
    std::fs::create_dir_all(&folder_path)
        .map_err(|_| AppError::Invalid("문서 export 폴더를 만들 수 없습니다".into()))?;
    let markdown_path = folder_path.join("README.md");
    std::fs::write(&markdown_path, &document.body)
        .map_err(|_| AppError::Invalid("문서 Markdown 파일을 저장할 수 없습니다".into()))?;
    Ok(DocumentExportPaths { folder_path, markdown_path })
}

fn is_supported_document_image(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
}

fn legacy_deliverable_files_root(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("deliverables")
}

fn resolve_deliverable_open_path(
    app_data_dir: &Path,
    store_root: &Path,
    stored_path: &str,
) -> Result<PathBuf> {
    let allowed_roots = [
        deliverable_files_root(store_root),
        legacy_deliverable_files_root(app_data_dir),
    ];
    let allowed_roots: Vec<PathBuf> = allowed_roots
        .iter()
        .filter_map(|root| root.canonicalize().ok())
        .collect();
    if allowed_roots.is_empty() {
        return Err(AppError::Invalid("산출물 저장 폴더를 찾을 수 없습니다".into()));
    }
    let file = Path::new(stored_path)
        .canonicalize()
        .map_err(|_| AppError::Invalid("산출물 파일을 찾을 수 없습니다".into()))?;
    if !file.is_file() || !allowed_roots.iter().any(|root| file.starts_with(root)) {
        return Err(AppError::Invalid("허용되지 않은 산출물 파일 경로입니다".into()));
    }
    Ok(file)
}

pub(crate) fn migrate_legacy_deliverable_files(app_data_dir: &Path, store: &mut Store) -> Result<usize> {
    let legacy_root = legacy_deliverable_files_root(app_data_dir);
    let legacy_root = match legacy_root.canonicalize() {
        Ok(path) => path,
        Err(_) => return Ok(0),
    };
    let target_root = deliverable_files_root(&store.root);
    let mut migrated = 0;

    for deliverable in store.deliverables.list() {
        let Some(file_path) = deliverable.file_path.as_deref() else {
            continue;
        };
        let source = match Path::new(file_path).canonicalize() {
            Ok(path) => path,
            Err(_) => continue,
        };
        if !source.is_file() || !source.starts_with(&legacy_root) {
            continue;
        }
        let Some(file_name) = source.file_name() else {
            continue;
        };
        let target_dir = target_root.join(&deliverable.id);
        std::fs::create_dir_all(&target_dir)
            .map_err(|_| AppError::Invalid("산출물 저장 폴더를 만들 수 없습니다".into()))?;
        let target = target_dir.join(file_name);
        if !target.exists() {
            std::fs::copy(&source, &target)
                .map_err(|_| AppError::Invalid("산출물 파일을 vault로 복사할 수 없습니다".into()))?;
        }
        ops::deliverable::set_file_path(store, &deliverable.id, &target.to_string_lossy())?;
        migrated += 1;
    }

    Ok(migrated)
}

#[tauri::command]
pub fn deliverable_open(app: tauri::AppHandle, state: State<AppState>, id: String) -> Result<()> {
    use tauri::Manager;
    use tauri_plugin_opener::OpenerExt;

    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| AppError::Invalid("앱 데이터 폴더를 찾을 수 없음".into()))?;
    let (stored_path, store_root) = {
        let store = state.store.lock().unwrap();
        (
            ops::deliverable::file_path_of(&store, &id)?
                .ok_or_else(|| AppError::Invalid("파일 경로가 없습니다".into()))?,
            store.root.clone(),
        )
    };
    let path = resolve_deliverable_open_path(&data_dir, &store_root, &stored_path)?;
    app.opener()
        .open_path(path.to_string_lossy().to_string(), None::<String>)
        .map_err(|_| AppError::Invalid("파일을 열 수 없습니다".into()))
}

#[tauri::command]
pub fn deliverable_show_in_folder(
    app: tauri::AppHandle,
    state: State<AppState>,
    id: String,
) -> Result<()> {
    use tauri::Manager;
    use tauri_plugin_opener::OpenerExt;

    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| AppError::Invalid("앱 데이터 폴더를 찾을 수 없음".into()))?;
    let (stored_path, store_root) = {
        let store = state.store.lock().unwrap();
        (
            ops::deliverable::file_path_of(&store, &id)?
                .ok_or_else(|| AppError::Invalid("파일 경로가 없습니다".into()))?,
            store.root.clone(),
        )
    };
    // 파일이 든 폴더를 "여는" 대신 파일 자체를 Finder에서 선택(reveal)한다.
    // 각 산출물은 고유 id 폴더에 개별 저장되므로 폴더를 열면 원본 파일 하나만 보이고,
    // iCloud 오프로드 시 폴더가 비어 보이는 문제도 reveal 로 파일을 지정하면 해소된다.
    let file = resolve_deliverable_open_path(&data_dir, &store_root, &stored_path)?;
    app.opener()
        .reveal_item_in_dir(&file)
        .map_err(|_| AppError::Invalid("파일을 Finder에서 열 수 없습니다".into()))
}

/// 파일 업로드(다중). 프론트가 다이얼로그로 고른 경로들을 받아 앱 데이터 폴더로 복사하고
/// 각각 산출물 행을 생성한다. 개별 파일 실패는 건너뛰고 성공분만 반환한다.
#[tauri::command]
pub fn deliverable_upload(
    state: State<AppState>,
    business_id: String,
    project_id: Option<String>,
    folder_id: Option<String>,
    paths: Vec<String>,
) -> Result<Vec<Deliverable>> {
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
        let dest_dir = deliverable_files_root(&store.root).join(&d.id);
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliverableUploadFile {
    pub file_name: String,
    pub bytes: Vec<u8>,
}

pub(crate) fn create_deliverables_from_uploaded_files(
    store: &mut Store,
    business_id: &str,
    project_id: Option<&str>,
    folder_id: Option<&str>,
    files: Vec<DeliverableUploadFile>,
) -> Result<Vec<Deliverable>> {
    let mut created = Vec::new();
    for file in files {
        let filename = match Path::new(&file.file_name)
            .file_name()
            .and_then(|f| f.to_str())
            .filter(|value| !value.trim().is_empty())
        {
            Some(f) => f.to_string(),
            None => continue,
        };
        if file.bytes.is_empty() {
            continue;
        }
        let d = match ops::deliverable::create_file(
            store,
            business_id,
            project_id,
            folder_id,
            &filename,
            &filename,
            file.bytes.len() as i64,
        ) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let dest_dir = deliverable_files_root(&store.root).join(&d.id);
        if std::fs::create_dir_all(&dest_dir).is_err() {
            let _ = ops::deliverable::delete(store, &d.id);
            continue;
        }
        let dest = dest_dir.join(&filename);
        if std::fs::write(&dest, file.bytes).is_err() {
            let _ = ops::deliverable::delete(store, &d.id);
            let _ = std::fs::remove_dir(&dest_dir);
            continue;
        }
        let dest_str = dest.to_string_lossy().to_string();
        if ops::deliverable::set_file_path(store, &d.id, &dest_str).is_err() {
            continue;
        }
        created.push(ops::deliverable::get(store, &d.id)?);
    }
    Ok(created)
}

#[tauri::command]
pub fn deliverable_upload_files(
    state: State<AppState>,
    business_id: String,
    project_id: Option<String>,
    folder_id: Option<String>,
    files: Vec<DeliverableUploadFile>,
) -> Result<Vec<Deliverable>> {
    let mut store = state.store.lock().unwrap();
    create_deliverables_from_uploaded_files(
        &mut store,
        &business_id,
        project_id.as_deref(),
        folder_id.as_deref(),
        files,
    )
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

/// 현재 설정된 vault 폴더(없으면 None = 기본 위치).
#[tauri::command]
pub fn vault_path(app: tauri::AppHandle) -> Result<Option<String>> {
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| crate::error::AppError::Invalid(format!("앱 데이터 경로 오류: {e}")))?;
    Ok(crate::config::read_vault_path(&dir))
}

/// vault 폴더를 지정하고 Store를 그 위치(`<path>/.projectManger`)로 다시 연다.
#[tauri::command]
pub fn vault_set(app: tauri::AppHandle, state: State<AppState>, path: String) -> Result<()> {
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| crate::error::AppError::Invalid(format!("앱 데이터 경로 오류: {e}")))?;
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(crate::error::AppError::Invalid("vault 경로가 비어 있습니다".into()));
    }
    if !std::path::Path::new(trimmed).is_dir() {
        return Err(crate::error::AppError::Invalid("선택한 vault 폴더가 존재하지 않습니다".into()));
    }
    let prev = crate::config::read_vault_path(&dir);
    crate::config::write_vault_path(&dir, Some(trimmed))?;
    let new_root = crate::config::store_root(&dir);
    match crate::store::Store::open(new_root) {
        Ok(new_store) => {
            *state.store.lock().unwrap() = new_store;
            Ok(())
        }
        Err(e) => {
            // 롤백: 이전 vault 설정 복원
            let _ = crate::config::write_vault_path(&dir, prev.as_deref());
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::store::ops::{business, deliverable};
    use crate::store::ids::new_id;
    use crate::store::model::Document;
    use crate::store::Store;

    fn tmp_dir(prefix: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("{prefix}_{}", new_id()));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn resolve_deliverable_open_path_accepts_files_under_app_deliverables() {
        let app_data = tmp_dir("cmd_open_app");
        let store_root = tmp_dir("cmd_open_store").join(".projectManger");
        let file_dir = app_data.join("deliverables").join("d1");
        std::fs::create_dir_all(&file_dir).unwrap();
        let file = file_dir.join("report.pdf");
        std::fs::write(&file, b"pdf").unwrap();

        let resolved =
            super::resolve_deliverable_open_path(&app_data, &store_root, file.to_str().unwrap())
                .unwrap();

        assert_eq!(resolved, file.canonicalize().unwrap());
    }

    #[test]
    fn resolve_deliverable_open_path_accepts_files_under_store_deliverables() {
        let app_data = tmp_dir("cmd_open_app");
        let store_root = tmp_dir("cmd_open_store").join(".projectManger");
        let file_dir = store_root.join("files").join("deliverables").join("d1");
        std::fs::create_dir_all(&file_dir).unwrap();
        let file = file_dir.join("report.pdf");
        std::fs::write(&file, b"pdf").unwrap();

        let resolved =
            super::resolve_deliverable_open_path(&app_data, &store_root, file.to_str().unwrap())
                .unwrap();

        assert_eq!(resolved, file.canonicalize().unwrap());
    }

    #[test]
    fn resolve_deliverable_open_path_rejects_files_outside_app_deliverables() {
        let app_data = tmp_dir("cmd_open_app");
        let store_root = tmp_dir("cmd_open_store").join(".projectManger");
        let outside = tmp_dir("cmd_open_outside").join("report.pdf");
        std::fs::write(&outside, b"pdf").unwrap();

        let result =
            super::resolve_deliverable_open_path(&app_data, &store_root, outside.to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn resolve_deliverable_open_path_rejects_missing_files() {
        let app_data = tmp_dir("cmd_open_app");
        let store_root = tmp_dir("cmd_open_store").join(".projectManger");
        let missing = app_data.join("deliverables").join("d1").join("missing.pdf");

        let result =
            super::resolve_deliverable_open_path(&app_data, &store_root, missing.to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn deliverable_files_root_lives_under_store_root() {
        let store_root = tmp_dir("cmd_file_root").join(".projectManger");

        assert_eq!(
            super::deliverable_files_root(&store_root),
            store_root.join("files").join("deliverables")
        );
    }

    #[test]
    fn document_asset_files_root_lives_under_store_root() {
        let store_root = tmp_dir("cmd_document_asset_root").join(".projectManger");

        assert_eq!(
            super::document_asset_files_root(&store_root, "doc-1"),
            store_root.join("files").join("documents").join("doc-1").join("assets")
        );
    }

    #[test]
    fn export_document_markdown_writes_readme_under_current_export_folder() {
        let store_root = tmp_dir("cmd_document_export_root").join(".projectManger");
        let doc = Document {
            id: "doc-1".into(),
            business_id: "biz-1".into(),
            project_id: None,
            folder_id: None,
            title: "기획/초안.md".into(),
            icon: None,
            body: "# 제목\n본문".into(),
            editor_body: Some("[{}]".into()),
            editor_body_format: Some("blocknote-json".into()),
            collaboration_state: Some("old-state".into()),
            blocks: vec![],
            sort_order: 1.0,
            archived_at: None,
            created_at: "2026-07-08T00:00:00.000Z".into(),
            updated_at: "2026-07-08T00:00:00.000Z".into(),
        };

        let exported = super::export_document_markdown(&store_root, &doc).unwrap();

        assert_eq!(exported.folder_path, store_root.join("files").join("documents").join("doc-1").join("exports").join("current"));
        assert_eq!(exported.markdown_path, exported.folder_path.join("README.md"));
        assert_eq!(std::fs::read_to_string(exported.markdown_path).unwrap(), "# 제목\n본문");
    }

    #[test]
    fn is_supported_document_image_accepts_common_images() {
        assert!(super::is_supported_document_image("a.png"));
        assert!(super::is_supported_document_image("a.jpg"));
        assert!(super::is_supported_document_image("a.jpeg"));
        assert!(super::is_supported_document_image("a.gif"));
        assert!(super::is_supported_document_image("a.webp"));
        assert!(!super::is_supported_document_image("a.pdf"));
    }

    #[test]
    fn migrate_legacy_deliverable_files_copies_files_into_store_root() {
        let app_data = tmp_dir("cmd_migrate_app");
        let store_root = tmp_dir("cmd_migrate_store").join(".projectManger");
        let mut store = Store::open(store_root.clone()).unwrap();
        let business_id = business::create(&mut store, "폴라리스AI", "client", None).unwrap().id;
        let legacy = app_data
            .join("deliverables")
            .join("legacy-deliverable")
            .join("report.pdf");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(&legacy, b"pdf").unwrap();
        let d = deliverable::create_file(
            &mut store,
            &business_id,
            None,
            None,
            "report.pdf",
            "report.pdf",
            3,
        )
        .unwrap();
        deliverable::set_file_path(&mut store, &d.id, legacy.to_str().unwrap()).unwrap();

        let migrated = super::migrate_legacy_deliverable_files(&app_data, &mut store).unwrap();

        let updated = deliverable::get(&store, &d.id).unwrap();
        let copied = store_root
            .join("files")
            .join("deliverables")
            .join(&d.id)
            .join("report.pdf");
        assert_eq!(migrated, 1);
        assert_eq!(updated.file_path.as_deref(), copied.to_str());
        assert_eq!(std::fs::read(copied).unwrap(), b"pdf");
        assert!(legacy.exists());
    }

    #[test]
    fn create_deliverables_from_uploaded_files_writes_bytes_under_store_root() {
        let store_root = tmp_dir("cmd_upload_files_store").join(".projectManger");
        let mut store = Store::open(store_root.clone()).unwrap();
        let business_id = business::create(&mut store, "폴라리스AI", "client", None).unwrap().id;

        let created = super::create_deliverables_from_uploaded_files(
            &mut store,
            &business_id,
            None,
            None,
            vec![
                super::DeliverableUploadFile {
                    file_name: "../report.pdf".into(),
                    bytes: b"pdf".to_vec(),
                },
                super::DeliverableUploadFile {
                    file_name: "empty.txt".into(),
                    bytes: Vec::new(),
                },
            ],
        )
        .unwrap();

        assert_eq!(created.len(), 1);
        assert_eq!(created[0].title, "report.pdf");
        assert_eq!(created[0].original_name.as_deref(), Some("report.pdf"));
        assert_eq!(created[0].file_size, Some(3));
        let stored = std::path::PathBuf::from(created[0].file_path.as_deref().unwrap());
        assert_eq!(stored, store_root.join("files").join("deliverables").join(&created[0].id).join("report.pdf"));
        assert_eq!(std::fs::read(stored).unwrap(), b"pdf");
    }
}
