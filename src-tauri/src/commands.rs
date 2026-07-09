use crate::error::{AppError, Result};
use crate::secrets;
use crate::store::model::{
    Block, Business, Deliverable, DeliverableVersion, Document, Folder, Label, Memo, Project,
    RecurringTask, Server, Snippet, Task, Template,
};
use crate::store::layout::{self, DELIVERABLES_AREA};
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
    let sr = store.root.clone();
    let b = ops::business::create(&mut store, &input.name, &input.type_, input.color.as_deref())?;
    mirror_scaffold_business(&store, &sr, &b.id)?;
    Ok(b)
}

#[tauri::command]
pub fn business_update(state: State<AppState>, input: BusinessUpdate) -> Result<Business> {
    let mut store = state.store.lock().unwrap();
    let sr = store.root.clone();
    // 이름이 바뀌면 디스크 폴더도 rename(메타 갱신 전에).
    mirror_rename_business(&store, &sr, &input.id, &input.name)?;
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
    let sr = store.root.clone();
    mirror_rename_business(&store, &sr, &id, &name)?;
    ops::business::rename(&mut store, &id, &name)
}

#[tauri::command]
pub fn business_archive(state: State<AppState>, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    let sr = store.root.clone();
    ops::business::archive(&mut store, &id)?;
    mirror_trash_business(&store, &sr, &id)
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
    let sr = store.root.clone();
    move_deliverable_file(&mut store, &sr, &id, folder_id.as_deref())
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
    let sr = store.root.clone();
    ops::deliverable::archive(&mut store, &id)?;
    // 물리 파일도 휴지통으로 이동 → iCloud/Finder 에서도 사라진다.
    mirror_trash_deliverable(&store, &sr, &id)
}

/// 디스크 미러와 산출물 메타를 재조정한다(외부 Finder 추가/삭제 반영). 프론트가 뷰 진입 시 호출.
#[tauri::command]
pub fn deliverable_reconcile(state: State<AppState>) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    let sr = store.root.clone();
    reconcile_deliverables(&mut store, &sr).map(|_| ())
}

fn deliverable_files_root(store_root: &Path) -> PathBuf {
    store_root.join("files").join("deliverables")
}

/// store_root(`.../Work Vault/.projectManger`)의 부모 = 앱 Vault 루트(`.../Work Vault`).
fn vault_root_of(store_root: &Path) -> PathBuf {
    store_root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| store_root.to_path_buf())
}

/// 사업 폴더명(정규화). 사업이 없으면 에러.
fn business_component(store: &Store, business_id: &str) -> Result<String> {
    let b = store.businesses.get(business_id).ok_or(AppError::NotFound)?;
    Ok(layout::sanitize_component(&b.name, &b.id))
}

/// 폴더 체인(루트→리프) 이름을 정규화해 반환(최대 2단계).
fn folder_chain_components(store: &Store, folder_id: Option<&str>) -> Result<Vec<String>> {
    let Some(fid) = folder_id else { return Ok(vec![]) };
    let f = store.folders.get(fid).ok_or(AppError::NotFound)?;
    let mut chain = vec![layout::sanitize_component(&f.name, &f.id)];
    if let Some(pid) = &f.parent_id {
        if let Some(parent) = store.folders.get(pid) {
            chain.insert(0, layout::sanitize_component(&parent.name, &parent.id));
        }
    }
    Ok(chain)
}

/// 앱 Vault 루트 기준 산출물 디렉터리 상대경로(파일명 제외).
fn deliverable_dir_rel(store: &Store, business_id: &str, folder_id: Option<&str>) -> Result<PathBuf> {
    let mut p = PathBuf::from(business_component(store, business_id)?).join(DELIVERABLES_AREA);
    for c in folder_chain_components(store, folder_id)? {
        p = p.join(c);
    }
    Ok(p)
}

/// 절대 산출물 디렉터리.
fn deliverable_dir_abs(
    store: &Store,
    store_root: &Path,
    business_id: &str,
    folder_id: Option<&str>,
) -> Result<PathBuf> {
    Ok(vault_root_of(store_root).join(deliverable_dir_rel(store, business_id, folder_id)?))
}

/// deliverable file_path(상대=vault 기준, 또는 절대 하위호환)를 실제 절대경로로 해석.
fn resolve_deliverable_path(store_root: &Path, stored: &str) -> Result<PathBuf> {
    let p = PathBuf::from(stored);
    let abs = if p.is_absolute() { p } else { vault_root_of(store_root).join(p) };
    let canon = abs
        .canonicalize()
        .map_err(|_| AppError::Invalid("산출물 파일을 찾을 수 없습니다".into()))?;
    if !canon.is_file() {
        return Err(AppError::Invalid("산출물 파일을 찾을 수 없습니다".into()));
    }
    Ok(canon)
}

/// 확장자 추출("보고서.pdf" -> "pdf", "noext" -> "").
fn ext_of(file_name: &str) -> String {
    Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string()
}

/// 정규화된 base(=title)와 파일 ext 로 최종 파일명 결정.
/// title 이 이미 ".ext" 로 끝나면 그대로(이중 확장자 방지), 아니면 append.
fn compose_file_name(base: &str, ext: &str) -> String {
    if ext.is_empty() {
        return base.to_string();
    }
    let suffix = format!(".{}", ext.to_lowercase());
    if base.to_lowercase().ends_with(&suffix) {
        base.to_string()
    } else {
        format!("{base}.{ext}")
    }
}

/// 사업의 산출물 루트 `{사업}/산출물`(절대).
fn business_deliverables_root_abs(store: &Store, store_root: &Path, business_id: &str) -> Result<PathBuf> {
    Ok(vault_root_of(store_root)
        .join(business_component(store, business_id)?)
        .join(DELIVERABLES_AREA))
}

/// 사업 생성 시 `{사업}/산출물/` 스캐폴딩.
fn mirror_scaffold_business(store: &Store, store_root: &Path, business_id: &str) -> Result<()> {
    let dir = business_deliverables_root_abs(store, store_root, business_id)?;
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Invalid(format!("사업 폴더 생성 실패: {e}")))?;
    Ok(())
}

/// 사업 폴더 rename(메타는 호출측에서 rename).
fn mirror_rename_business(store: &Store, store_root: &Path, business_id: &str, new_name: &str) -> Result<()> {
    let vault = vault_root_of(store_root);
    let old_abs = vault.join(business_component(store, business_id)?);
    if !old_abs.exists() {
        return Ok(());
    }
    let old_name = old_abs.file_name().and_then(|x| x.to_str()).map(|s| s.to_string());
    let existing: Vec<String> = std::fs::read_dir(&vault)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|n| Some(n) != old_name.as_ref())
                .filter(|n| n != ".projectManger")
                .collect()
        })
        .unwrap_or_default();
    let name = layout::unique_name(&existing, &layout::sanitize_component(new_name, business_id));
    std::fs::rename(&old_abs, vault.join(&name))
        .map_err(|e| AppError::Invalid(format!("사업 폴더 이름변경 실패: {e}")))?;
    Ok(())
}

/// 사업 폴더째 휴지통 이동.
fn mirror_trash_business(store: &Store, store_root: &Path, business_id: &str) -> Result<()> {
    let vault = vault_root_of(store_root);
    let abs = vault.join(business_component(store, business_id)?);
    if !abs.exists() {
        return Ok(());
    }
    let trash = store_root.join("trash");
    std::fs::create_dir_all(&trash).map_err(|e| AppError::Invalid(format!("휴지통 생성 실패: {e}")))?;
    let dest = trash.join(format!("biz__{business_id}"));
    std::fs::rename(&abs, &dest).map_err(|e| AppError::Invalid(format!("사업 폴더 휴지통 이동 실패: {e}")))?;
    Ok(())
}

/// 파일이 로컬에 있거나 iCloud 오프로드 스텁(`.{name}.icloud`)이 있으면 "존재"로 본다.
fn deliverable_file_exists_offload_safe(abs: &Path) -> bool {
    if abs.exists() {
        return true;
    }
    if let (Some(dir), Some(name)) = (abs.parent(), abs.file_name().and_then(|n| n.to_str())) {
        if dir.join(format!(".{name}.icloud")).exists() {
            return true;
        }
    }
    false
}

/// `{사업}/산출물` 트리(최대 2단계 폴더)를 훑어 실제 파일을 수집.
/// 각 항목: (앱Vault 기준 상대경로, 폴더명 체인, 파일명, 크기). 숨김(.*)은 제외.
fn collect_area_files(dir: &Path, chain: &[String], biz_comp: &str, out: &mut Vec<(String, Vec<String>, String, i64)>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.filter_map(|e| e.ok()) {
        let Ok(name) = e.file_name().into_string() else { continue };
        if name.starts_with('.') || name.starts_with("~$") || name.starts_with(".~") {
            continue; // .DS_Store / .icloud 스텁 / Office 임시 잠금(~$...) / 기타 숨김
        }
        let path = e.path();
        if path.is_dir() {
            if chain.len() >= 2 {
                continue; // 폴더는 2단계까지
            }
            let mut c = chain.to_vec();
            c.push(name);
            collect_area_files(&path, &c, biz_comp, out);
        } else if path.is_file() {
            let size = std::fs::metadata(&path).map(|m| m.len() as i64).unwrap_or(0);
            let mut rel = PathBuf::from(biz_comp).join(DELIVERABLES_AREA);
            for c in chain {
                rel = rel.join(c);
            }
            rel = rel.join(&name);
            out.push((rel.to_string_lossy().to_string(), chain.to_vec(), name, size));
        }
    }
}

/// 이름으로 산출물 폴더를 찾고 없으면 만든다(정규화 이름 비교).
fn find_or_create_deliverable_folder(store: &mut Store, business_id: &str, parent_id: Option<&str>, name: &str) -> Result<String> {
    let existing = store.folders.list().into_iter().find(|f| {
        f.business_id == business_id
            && f.kind == "deliverable"
            && f.archived_at.is_none()
            && f.parent_id.as_deref() == parent_id
            && layout::sanitize_component(&f.name, &f.id) == name
    });
    if let Some(f) = existing {
        return Ok(f.id);
    }
    let f = ops::folder::create(store, business_id, "deliverable", parent_id, name)?;
    Ok(f.id)
}

/// 폴더명 체인(0~2) → 폴더 id(없으면 생성). 빈 체인이면 None(미분류).
fn resolve_or_create_deliverable_folder(store: &mut Store, business_id: &str, chain: &[String]) -> Result<Option<String>> {
    if chain.is_empty() {
        return Ok(None);
    }
    let root_id = find_or_create_deliverable_folder(store, business_id, None, &chain[0])?;
    if chain.len() == 1 {
        return Ok(Some(root_id));
    }
    let sub_id = find_or_create_deliverable_folder(store, business_id, Some(&root_id), &chain[1])?;
    Ok(Some(sub_id))
}

/// 디스크 미러 ↔ 산출물 메타 재조정(스캔 기반).
/// - 디스크에 있는데 메타에 없는 파일 → import(상태 기본, 폴더는 경로대로 생성/연결)
/// - 메타에 있는데 디스크에서 진짜 사라진 파일 → archive(오프로드 스텁은 존재로 간주)
/// 반환: (import 건수, 제거 건수).
pub(crate) fn reconcile_deliverables(store: &mut Store, store_root: &Path) -> Result<(usize, usize)> {
    let vault = vault_root_of(store_root);
    let mut imported = 0usize;
    let mut removed = 0usize;
    let businesses: Vec<_> = store
        .businesses
        .list()
        .into_iter()
        .filter(|b| b.archived_at.is_none())
        .collect();

    for b in businesses {
        let biz_comp = layout::sanitize_component(&b.name, &b.id);
        let area = vault.join(&biz_comp).join(DELIVERABLES_AREA);
        let mut disk: Vec<(String, Vec<String>, String, i64)> = Vec::new();
        if area.is_dir() {
            collect_area_files(&area, &[], &biz_comp, &mut disk);
        }
        let metas: Vec<_> = store
            .deliverables
            .list()
            .into_iter()
            .filter(|d| d.business_id == b.id && d.archived_at.is_none() && d.kind == "file")
            .collect();
        let meta_paths: std::collections::HashSet<String> =
            metas.iter().filter_map(|d| d.file_path.clone()).collect();

        // 1) import new disk files
        for (rel, chain, fname, size) in &disk {
            if !meta_paths.contains(rel) {
                let folder_id = resolve_or_create_deliverable_folder(store, &b.id, chain)?;
                let d = ops::deliverable::create_file(store, &b.id, None, folder_id.as_deref(), fname, fname, *size)?;
                ops::deliverable::set_file_path(store, &d.id, rel)?;
                imported += 1;
            }
        }
        // 2) archive metadata whose file truly vanished (offload-safe)
        for d in &metas {
            if let Some(fp) = &d.file_path {
                if !deliverable_file_exists_offload_safe(&vault.join(fp)) {
                    ops::deliverable::archive(store, &d.id)?;
                    removed += 1;
                }
            }
        }
    }
    Ok((imported, removed))
}

/// 산출물 개별 휴지통 디렉터리: `.projectManger/trash/deliverable/{id}`.
fn deliverable_trash_dir(store_root: &Path, id: &str) -> PathBuf {
    store_root.join("trash").join("deliverable").join(id)
}

/// 산출물 삭제(휴지통) 시 물리 파일을 휴지통으로 이동. 메타 file_path 는 유지(복원 시 재계산).
fn mirror_trash_deliverable(store: &Store, store_root: &Path, id: &str) -> Result<()> {
    let d = ops::deliverable::get(store, id)?;
    let Some(rel) = d.file_path.as_deref() else { return Ok(()) };
    let src = vault_root_of(store_root).join(rel);
    if !src.is_file() {
        return Ok(());
    }
    let trash_dir = deliverable_trash_dir(store_root, id);
    std::fs::create_dir_all(&trash_dir)
        .map_err(|e| AppError::Invalid(format!("휴지통 생성 실패: {e}")))?;
    let fname = src.file_name().map(|f| f.to_os_string()).unwrap_or_default();
    std::fs::rename(&src, trash_dir.join(fname))
        .map_err(|e| AppError::Invalid(format!("파일 휴지통 이동 실패: {e}")))?;
    Ok(())
}

/// 복원 시 휴지통의 파일을 현재 사업/폴더 기준 미러 경로로 되돌린다.
fn mirror_restore_deliverable(store: &mut Store, store_root: &Path, id: &str) -> Result<()> {
    let d = ops::deliverable::get(store, id)?;
    let trash_dir = deliverable_trash_dir(store_root, id);
    let found = std::fs::read_dir(&trash_dir)
        .ok()
        .and_then(|rd| rd.filter_map(|e| e.ok()).map(|e| e.path()).find(|p| p.is_file()));
    let Some(src) = found else { return Ok(()) };
    let dir_abs = deliverable_dir_abs(store, store_root, &d.business_id, d.folder_id.as_deref())?;
    std::fs::create_dir_all(&dir_abs)
        .map_err(|e| AppError::Invalid(format!("복원 폴더 생성 실패: {e}")))?;
    let fname = src.file_name().and_then(|f| f.to_str()).unwrap_or_default().to_string();
    let existing: Vec<String> = std::fs::read_dir(&dir_abs)
        .map(|rd| rd.filter_map(|e| e.ok()).filter_map(|e| e.file_name().into_string().ok()).collect())
        .unwrap_or_default();
    let name = layout::unique_name(&existing, &fname);
    std::fs::rename(&src, dir_abs.join(&name))
        .map_err(|e| AppError::Invalid(format!("파일 복원 실패: {e}")))?;
    let _ = std::fs::remove_dir(&trash_dir); // 빈 휴지통 폴더 정리
    let rel = deliverable_dir_rel(store, &d.business_id, d.folder_id.as_deref())?.join(&name);
    ops::deliverable::set_file_path(store, id, &rel.to_string_lossy())?;
    Ok(())
}

/// 영구삭제 시 휴지통의 물리 파일 제거.
fn mirror_purge_deliverable(store_root: &Path, id: &str) {
    let _ = std::fs::remove_dir_all(deliverable_trash_dir(store_root, id));
}

/// 폴더 생성 시 디스크 디렉터리 생성(빈 폴더라도). deliverable 종류만 미러링.
fn mirror_create_folder(store: &Store, store_root: &Path, folder_id: &str) -> Result<()> {
    let f = store.folders.get(folder_id).ok_or(AppError::NotFound)?;
    if f.kind != "deliverable" {
        return Ok(());
    }
    let dir = deliverable_dir_abs(store, store_root, &f.business_id, Some(folder_id))?;
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Invalid(format!("폴더 생성 실패: {e}")))?;
    Ok(())
}

/// 폴더 이름변경 시 디스크 디렉터리 rename(메타는 호출측에서 rename). deliverable 종류만.
fn mirror_rename_folder(store: &Store, store_root: &Path, folder_id: &str, new_name: &str) -> Result<()> {
    let f = store.folders.get(folder_id).ok_or(AppError::NotFound)?;
    if f.kind != "deliverable" {
        return Ok(());
    }
    let old_abs = deliverable_dir_abs(store, store_root, &f.business_id, Some(folder_id))?;
    if !old_abs.exists() {
        return Ok(());
    }
    let parent = old_abs.parent().map(Path::to_path_buf).unwrap_or_else(|| vault_root_of(store_root));
    let old_name = old_abs.file_name().and_then(|x| x.to_str()).map(|s| s.to_string());
    let existing: Vec<String> = std::fs::read_dir(&parent)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|n| Some(n) != old_name.as_ref())
                .collect()
        })
        .unwrap_or_default();
    let name = layout::unique_name(&existing, &layout::sanitize_component(new_name, folder_id));
    std::fs::rename(&old_abs, parent.join(&name))
        .map_err(|e| AppError::Invalid(format!("폴더 이름변경 실패: {e}")))?;
    Ok(())
}

/// 폴더째 휴지통(`.projectManger/trash/{folderId}__{name}`)으로 이동. deliverable 종류만.
fn mirror_trash_folder(store: &Store, store_root: &Path, folder_id: &str) -> Result<()> {
    let f = store.folders.get(folder_id).ok_or(AppError::NotFound)?;
    if f.kind != "deliverable" {
        return Ok(());
    }
    let abs = deliverable_dir_abs(store, store_root, &f.business_id, Some(folder_id))?;
    if !abs.exists() {
        return Ok(());
    }
    let trash = store_root.join("trash");
    std::fs::create_dir_all(&trash).map_err(|e| AppError::Invalid(format!("휴지통 생성 실패: {e}")))?;
    let safe = layout::sanitize_component(&f.name, folder_id);
    let dest = trash.join(format!("{folder_id}__{safe}"));
    std::fs::rename(&abs, &dest).map_err(|e| AppError::Invalid(format!("폴더 휴지통 이동 실패: {e}")))?;
    Ok(())
}

/// 산출물 title 변경 + 디스크 파일도 rename(확장자 유지, 충돌 시 접미).
fn rename_deliverable_file(
    store: &mut Store,
    store_root: &Path,
    id: &str,
    title: &str,
) -> Result<Deliverable> {
    let before = ops::deliverable::get(store, id)?;
    let updated = ops::deliverable::rename(store, id, title)?;
    if let Some(old_rel) = before.file_path.as_deref() {
        let vault = vault_root_of(store_root);
        let old_abs = vault.join(old_rel);
        if old_abs.is_file() {
            let ext = old_abs
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            let dir_abs = old_abs.parent().map(Path::to_path_buf).unwrap_or(vault.clone());
            let base = layout::sanitize_component(&updated.title, &updated.id);
            let desired = compose_file_name(&base, &ext);
            let old_name = old_abs.file_name().and_then(|f| f.to_str()).map(|s| s.to_string());
            let existing: Vec<String> = std::fs::read_dir(&dir_abs)
                .map(|rd| {
                    rd.filter_map(|e| e.ok())
                        .filter_map(|e| e.file_name().into_string().ok())
                        .filter(|n| Some(n) != old_name.as_ref())
                        .collect()
                })
                .unwrap_or_default();
            let name = layout::unique_name(&existing, &desired);
            let new_abs = dir_abs.join(&name);
            std::fs::rename(&old_abs, &new_abs)
                .map_err(|e| AppError::Invalid(format!("파일 이름변경 실패: {e}")))?;
            let new_rel = Path::new(old_rel)
                .parent()
                .map(|p| p.join(&name))
                .unwrap_or_else(|| PathBuf::from(&name));
            ops::deliverable::set_file_path(store, id, &new_rel.to_string_lossy())?;
        }
    }
    ops::deliverable::get(store, id)
}

/// 산출물 folder_id 변경 + 디스크 파일을 새 폴더 디렉터리로 이동(충돌 시 접미).
fn move_deliverable_file(
    store: &mut Store,
    store_root: &Path,
    id: &str,
    folder_id: Option<&str>,
) -> Result<Deliverable> {
    let before = ops::deliverable::get(store, id)?;
    let updated = ops::deliverable::set_folder(store, id, folder_id)?;
    if let Some(old_rel) = before.file_path.as_deref() {
        let vault = vault_root_of(store_root);
        let old_abs = vault.join(old_rel);
        if old_abs.is_file() {
            let file_name = old_abs
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or_default()
                .to_string();
            let new_dir_abs =
                deliverable_dir_abs(store, store_root, &updated.business_id, folder_id)?;
            std::fs::create_dir_all(&new_dir_abs)
                .map_err(|e| AppError::Invalid(format!("대상 폴더 생성 실패: {e}")))?;
            let existing: Vec<String> = std::fs::read_dir(&new_dir_abs)
                .map(|rd| {
                    rd.filter_map(|e| e.ok())
                        .filter_map(|e| e.file_name().into_string().ok())
                        .collect()
                })
                .unwrap_or_default();
            let name = layout::unique_name(&existing, &file_name);
            let new_abs = new_dir_abs.join(&name);
            std::fs::rename(&old_abs, &new_abs)
                .map_err(|e| AppError::Invalid(format!("파일 이동 실패: {e}")))?;
            let new_rel =
                deliverable_dir_rel(store, &updated.business_id, folder_id)?.join(&name);
            ops::deliverable::set_file_path(store, id, &new_rel.to_string_lossy())?;
        }
    }
    ops::deliverable::get(store, id)
}

/// 산출물 파일을 미러 레이아웃에 배치한다. 디렉터리 생성 → `title.ext`(충돌 시 접미) 결정 →
/// writer 로 실제 바이트 쓰기 → 메타의 file_path 를 상대경로로 기록. 상대경로 반환.
fn place_deliverable_file<W>(
    store: &mut Store,
    store_root: &Path,
    id: &str,
    ext: &str,
    writer: W,
) -> Result<PathBuf>
where
    W: FnOnce(&Path) -> std::io::Result<()>,
{
    let d = ops::deliverable::get(store, id)?;
    let dir_abs = deliverable_dir_abs(store, store_root, &d.business_id, d.folder_id.as_deref())?;
    std::fs::create_dir_all(&dir_abs)
        .map_err(|e| AppError::Invalid(format!("산출물 폴더 생성 실패: {e}")))?;
    let base = layout::sanitize_component(&d.title, &d.id);
    let desired = compose_file_name(&base, ext);
    let existing: Vec<String> = std::fs::read_dir(&dir_abs)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();
    let name = layout::unique_name(&existing, &desired);
    let dest_abs = dir_abs.join(&name);
    writer(&dest_abs).map_err(|e| AppError::Invalid(format!("파일 저장 실패: {e}")))?;
    let rel = deliverable_dir_rel(store, &d.business_id, d.folder_id.as_deref())?.join(&name);
    ops::deliverable::set_file_path(store, id, &rel.to_string_lossy())?;
    Ok(rel)
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

/// 신버전 최초 1회: 기존 GUID 격리 파일을 `{사업}/산출물/{카테고리}/title.ext` 미러로 복사하고
/// 메타 경로를 상대경로로 갱신한다. 복사·검증 성공분의 원본만 legacy-backup 으로 이동(무손실).
pub(crate) fn migrate_deliverables_to_disk_layout(store: &mut Store) -> Result<usize> {
    let store_root = store.root.clone();
    let marker = store_root.join(".migrated-v2");
    if marker.exists() {
        return Ok(0);
    }
    let vault = vault_root_of(&store_root);
    let backup_root = store_root.join("legacy-backup");
    let mut migrated = 0usize;

    for d in store.deliverables.list() {
        let Some(old) = d.file_path.clone() else { continue };
        let old_path = PathBuf::from(&old);
        // 멱등: 이미 미러 상대경로이고 파일이 그 자리에 있으면 재배치하지 않는다(중복 " (2)" 방지).
        if !old_path.is_absolute() && vault.join(&old_path).is_file() {
            continue;
        }
        let candidate = if old_path.is_absolute() { old_path.clone() } else { vault.join(&old_path) };
        // 저장된 경로가 유효하면 그대로, 아니면(재배치로 절대경로가 낡음) 현재 store 의
        // GUID 폴더(`files/deliverables/{id}/…`)에서 파일을 찾는다.
        let src_abs = if candidate.is_file() {
            candidate
        } else {
            let guid_dir = deliverable_files_root(&store_root).join(&d.id);
            let found = std::fs::read_dir(&guid_dir)
                .ok()
                .and_then(|rd| rd.filter_map(|e| e.ok()).map(|e| e.path()).find(|p| p.is_file()));
            match found {
                Some(p) => p,
                None => continue, // 접근 불가/오프로드/원본 없음 → 건너뜀
            }
        };
        let ext = src_abs.extension().and_then(|e| e.to_str()).unwrap_or("").to_string();
        // 대상 경로 계산 + 복사(place 규칙과 동일하나 원본은 유지). place 가 메타 경로도 갱신.
        let src_for_copy = src_abs.clone();
        let placed = place_deliverable_file(store, &store_root, &d.id, &ext, move |dest| {
            std::fs::copy(&src_for_copy, dest).map(|_| ())
        });
        if placed.is_err() {
            continue;
        }
        // 원본을 legacy-backup 으로 이동(id 기준). store 내부 파일만 대상.
        if src_abs.starts_with(&store_root) {
            let backup_dir = backup_root.join(&d.id);
            let _ = std::fs::create_dir_all(&backup_dir);
            let fname = src_abs.file_name().map(|f| f.to_os_string()).unwrap_or_default();
            let _ = std::fs::rename(&src_abs, backup_dir.join(fname));
            if let Some(parent) = src_abs.parent() {
                let _ = std::fs::remove_dir(parent); // 빈 GUID 폴더 정리(best-effort)
            }
        }
        migrated += 1;
    }
    let _ = std::fs::write(&marker, b"v2");
    Ok(migrated)
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
    use tauri_plugin_opener::OpenerExt;

    let (stored_path, store_root) = {
        let store = state.store.lock().unwrap();
        (
            ops::deliverable::file_path_of(&store, &id)?
                .ok_or_else(|| AppError::Invalid("파일 경로가 없습니다".into()))?,
            store.root.clone(),
        )
    };
    let path = resolve_deliverable_path(&store_root, &stored_path)?;
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
    use tauri_plugin_opener::OpenerExt;

    let (stored_path, store_root) = {
        let store = state.store.lock().unwrap();
        (
            ops::deliverable::file_path_of(&store, &id)?
                .ok_or_else(|| AppError::Invalid("파일 경로가 없습니다".into()))?,
            store.root.clone(),
        )
    };
    // 파일이 든 폴더를 "여는" 대신 파일 자체를 Finder에서 선택(reveal)한다.
    // iCloud 오프로드 시 폴더가 비어 보이는 문제도 reveal 로 파일을 지정하면 해소된다.
    let file = resolve_deliverable_path(&store_root, &stored_path)?;
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
    let store_root_owned = store.root.clone();
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
        let ext = ext_of(&filename);
        let src_path = src.to_path_buf();
        if place_deliverable_file(&mut store, &store_root_owned, &d.id, &ext, move |dest| {
            std::fs::copy(&src_path, dest).map(|_| ())
        })
        .is_err()
        {
            let _ = ops::deliverable::delete(&mut store, &d.id);
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
        let store_root_owned = store.root.clone();
        let ext = ext_of(&filename);
        let bytes = file.bytes;
        if place_deliverable_file(store, &store_root_owned, &d.id, &ext, move |dest| {
            std::fs::write(dest, bytes)
        })
        .is_err()
        {
            let _ = ops::deliverable::delete(store, &d.id);
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
    let sr = store.root.clone();
    rename_deliverable_file(&mut store, &sr, &id, &title)
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
    let sr = store.root.clone();
    let f = ops::folder::create(&mut store, &input.business_id, &input.kind, input.parent_id.as_deref(), &input.name)?;
    mirror_create_folder(&store, &sr, &f.id)?;
    Ok(f)
}

#[tauri::command]
pub fn folder_rename(state: State<AppState>, id: String, name: String) -> Result<Folder> {
    let mut store = state.store.lock().unwrap();
    let sr = store.root.clone();
    // 디스크 rename 을 먼저(현재 메타의 이름 기준으로 경로 계산) 한 뒤 메타 갱신.
    mirror_rename_folder(&store, &sr, &id, &name)?;
    ops::folder::rename(&mut store, &id, &name)
}

#[tauri::command]
pub fn folder_delete(state: State<AppState>, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    let sr = store.root.clone();
    let f = ops::folder::get(&store, &id)?;
    if f.kind == "deliverable" {
        // 1) 디스크: 폴더째 휴지통으로(메타 아직 존재해야 경로 계산 가능)
        mirror_trash_folder(&store, &sr, &id)?;
        // 2) 메타: 폴더+직계 자식에 속한 산출물을 archive(개별 복구 가능) + 미분류 처리
        let mut folder_ids = vec![id.clone()];
        for child in store.folders.list() {
            if child.parent_id.as_deref() == Some(id.as_str()) {
                folder_ids.push(child.id);
            }
        }
        let dels: Vec<String> = store
            .deliverables
            .list()
            .into_iter()
            .filter(|d| d.folder_id.as_deref().map(|fid| folder_ids.iter().any(|x| x == fid)).unwrap_or(false))
            .map(|d| d.id)
            .collect();
        for did in dels {
            let _ = ops::deliverable::set_folder(&mut store, &did, None);
            let _ = ops::deliverable::archive(&mut store, &did);
        }
    }
    // 3) 폴더 메타 제거(문서 폴더는 기존 동작 그대로).
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
    #[serde(default)]
    pub ai_bridge: bool,
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
    #[serde(default)]
    pub ai_bridge: bool,
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
        input.ai_bridge,
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
        input.ai_bridge,
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
    bridge: State<crate::aibridge::AiBridge>,
    id: String,
) -> Result<()> {
    let server = {
        let local = state.local.lock().unwrap();
        ops::server::get(&local, &id)?
    };
    let ports = if server.ai_bridge { Some(bridge.ensure_started(&app)?) } else { None };
    crate::terminal::connect(&app, &term, &server, ports.as_ref())?;
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

/// 로컬 셸 탭 — 원격 SSH 없이 `claude login`/`cswap` 등을 로컬에서 직접 실행할 수 있도록
/// 로컬 로그인 셸 PTY 세션을 연다. write/resize/disconnect 는 기존 ssh_* 커맨드가
/// 동일한 `TerminalManager` 세션 맵을 id 기준으로 그대로 처리한다.
#[tauri::command]
pub fn local_terminal_open(
    app: tauri::AppHandle,
    term: State<crate::terminal::TerminalManager>,
    id: String,
) -> Result<()> {
    crate::terminal::connect_local(&app, &term, &id)
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
    let sr = store.root.clone();
    ops::trash::restore(&mut store, &kind, &id)?;
    if kind == "deliverable" {
        // 휴지통의 물리 파일을 미러 경로로 되돌린다.
        mirror_restore_deliverable(&mut store, &sr, &id)?;
    }
    Ok(())
}

#[tauri::command]
pub fn trash_purge(state: State<AppState>, kind: String, id: String) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    let sr = store.root.clone();
    ops::trash::purge(&mut store, &kind, &id)?;
    if kind == "deliverable" {
        // 휴지통에 보관된 물리 파일 영구 제거.
        mirror_purge_deliverable(&sr, &id);
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
    fn resolve_open_path_accepts_relative_under_vault() {
        let root = tmp_dir("cmd_resolve_rel");
        let store_root = root.join("Work Vault").join(".projectManger");
        let vault = super::vault_root_of(&store_root);
        let file = vault.join("사업A").join("산출물").join("개요.png");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, b"x").unwrap();

        let rel = std::path::Path::new("사업A").join("산출물").join("개요.png");
        let resolved = super::resolve_deliverable_path(&store_root, &rel.to_string_lossy()).unwrap();
        assert_eq!(resolved, file.canonicalize().unwrap());
    }

    #[test]
    fn migrate_v2_moves_guid_files_to_mirror_and_backs_up() {
        use crate::store::ops::folder;
        let root = tmp_dir("cmd_mig_v2");
        let store_root = root.join("Work Vault").join(".projectManger");
        let mut s = Store::open(store_root.clone()).unwrap();
        let b = business::create(&mut s, "사업A", "si", None).unwrap();
        let cat = folder::create(&mut s, &b.id, "deliverable", None, "설계서").unwrap();
        let d = deliverable::create_file(&mut s, &b.id, None, Some(&cat.id), "개요", "그림1.png", 3).unwrap();
        // 구 GUID 레이아웃으로 파일 배치(절대경로 저장)
        let guid_dir = super::deliverable_files_root(&store_root).join(&d.id);
        std::fs::create_dir_all(&guid_dir).unwrap();
        let guid_file = guid_dir.join("그림1.png");
        std::fs::write(&guid_file, b"png").unwrap();
        deliverable::set_file_path(&mut s, &d.id, &guid_file.to_string_lossy()).unwrap();

        let n = super::migrate_deliverables_to_disk_layout(&mut s).unwrap();
        assert_eq!(n, 1);

        let rel = deliverable::file_path_of(&s, &d.id).unwrap().unwrap();
        assert_eq!(
            rel,
            std::path::Path::new("사업A").join("산출물").join("설계서").join("개요.png").to_string_lossy()
        );
        assert!(super::vault_root_of(&store_root).join(&rel).is_file());
        assert!(store_root.join("legacy-backup").exists());
        assert!(store_root.join(".migrated-v2").is_file());

        // 멱등: 재실행은 0건
        assert_eq!(super::migrate_deliverables_to_disk_layout(&mut s).unwrap(), 0);
    }

    #[test]
    fn migrate_v2_is_idempotent_without_marker_no_duplicate_files() {
        // 마커가 없어도(재실행) 이미 미러에 있는 파일은 다시 배치하지 않아야 한다.
        let root = tmp_dir("cmd_mig_idem");
        let store_root = root.join("Work Vault").join(".projectManger");
        let mut s = Store::open(store_root.clone()).unwrap();
        let b = business::create(&mut s, "사업A", "si", None).unwrap();
        let d = deliverable::create_file(&mut s, &b.id, None, None, "개요", "그림1.png", 3).unwrap();
        let guid = super::deliverable_files_root(&store_root).join(&d.id);
        std::fs::create_dir_all(&guid).unwrap();
        std::fs::write(guid.join("그림1.png"), b"png").unwrap();
        deliverable::set_file_path(&mut s, &d.id, &guid.join("그림1.png").to_string_lossy()).unwrap();

        assert_eq!(super::migrate_deliverables_to_disk_layout(&mut s).unwrap(), 1);
        // 마커 제거 후 재실행 → 0건, 미러엔 파일 1개만(중복 " (2)" 없음)
        std::fs::remove_file(store_root.join(".migrated-v2")).unwrap();
        assert_eq!(super::migrate_deliverables_to_disk_layout(&mut s).unwrap(), 0);
        let dir = super::vault_root_of(&store_root).join("사업A").join("산출물");
        let count = std::fs::read_dir(&dir).unwrap().filter(|e| e.as_ref().unwrap().path().is_file()).count();
        assert_eq!(count, 1, "재실행해도 미러 파일은 1개여야 함");
    }

    #[test]
    fn migrate_v2_finds_file_via_guid_dir_when_stored_path_is_stale() {
        // 재배치로 메타의 절대 file_path 가 낡았지만 실제 파일은 store GUID 폴더에 있는 경우.
        let root = tmp_dir("cmd_mig_stale");
        let store_root = root.join("Work Vault").join(".projectManger");
        let mut s = Store::open(store_root.clone()).unwrap();
        let b = business::create(&mut s, "사업A", "si", None).unwrap();
        let d = deliverable::create_file(&mut s, &b.id, None, None, "개요.png", "그림1.png", 3).unwrap();
        // 실제 파일은 현재 store GUID 폴더에 존재
        let guid_dir = super::deliverable_files_root(&store_root).join(&d.id);
        std::fs::create_dir_all(&guid_dir).unwrap();
        std::fs::write(guid_dir.join("그림1.png"), b"png").unwrap();
        // 그러나 메타 경로는 존재하지 않는 낡은 절대경로
        deliverable::set_file_path(&mut s, &d.id, "/nonexistent/old/.projectManger/files/deliverables/x/그림1.png").unwrap();

        let n = super::migrate_deliverables_to_disk_layout(&mut s).unwrap();
        assert_eq!(n, 1);
        let rel = deliverable::file_path_of(&s, &d.id).unwrap().unwrap();
        assert_eq!(rel, std::path::Path::new("사업A").join("산출물").join("개요.png").to_string_lossy());
        assert!(super::vault_root_of(&store_root).join(&rel).is_file());
    }

    #[test]
    fn business_cascade_scaffold_rename_trash() {
        let root = tmp_dir("cmd_biz_cascade");
        let store_root = root.join("Work Vault").join(".projectManger");
        let mut s = Store::open(store_root.clone()).unwrap();
        let vault = super::vault_root_of(&store_root);
        let b = business::create(&mut s, "사업A", "si", None).unwrap();

        super::mirror_scaffold_business(&s, &store_root, &b.id).unwrap();
        assert!(vault.join("사업A").join("산출물").is_dir());

        super::mirror_rename_business(&s, &store_root, &b.id, "사업B").unwrap();
        assert!(vault.join("사업B").is_dir());
        assert!(!vault.join("사업A").exists());

        business::rename(&mut s, &b.id, "사업B").unwrap();
        super::mirror_trash_business(&s, &store_root, &b.id).unwrap();
        assert!(!vault.join("사업B").exists());
    }

    #[test]
    fn reconcile_imports_new_disk_files_and_removes_missing() {
        use crate::store::ops::folder;
        let root = tmp_dir("cmd_reconcile");
        let store_root = root.join("Work Vault").join(".projectManger");
        let mut s = Store::open(store_root.clone()).unwrap();
        let b = business::create(&mut s, "사업A", "si", None).unwrap();
        let vault = super::vault_root_of(&store_root);

        // 기존 산출물 1개(정상 미러 배치)
        let keep = deliverable::create_file(&mut s, &b.id, None, None, "유지", "keep.txt", 3).unwrap();
        super::place_deliverable_file(&mut s, &store_root, &keep.id, "txt", |p| std::fs::write(p, b"k")).unwrap();

        // 메타엔 있지만 디스크에서 사라진 산출물 1개
        let gone = deliverable::create_file(&mut s, &b.id, None, None, "사라짐", "gone.txt", 3).unwrap();
        deliverable::set_file_path(&mut s, &gone.id, &std::path::Path::new("사업A").join("산출물").join("gone.txt").to_string_lossy()).unwrap();
        // (파일을 만들지 않음 → 디스크에 없음)

        // Finder 로 직접 추가한 것처럼: 카테고리 폴더 + 파일을 디스크에만 생성
        let cat_dir = vault.join("사업A").join("산출물").join("신규폴더");
        std::fs::create_dir_all(&cat_dir).unwrap();
        std::fs::write(cat_dir.join("외부추가.pdf"), b"ext").unwrap();

        // Office 임시 잠금 파일(~$...)은 import 대상이 아니어야 한다.
        std::fs::write(cat_dir.join("~$외부추가.pdf"), b"lock").unwrap();

        let (imported, removed) = super::reconcile_deliverables(&mut s, &store_root).unwrap();
        assert_eq!(imported, 1, "외부추가.pdf 1건만 import(~$ 잠금 제외)");
        assert_eq!(removed, 1, "gone.txt 1건 제거");

        let active = deliverable::list_by_business(&s, &b.id).unwrap();
        let titles: Vec<&str> = active.iter().map(|d| d.title.as_str()).collect();
        assert!(titles.contains(&"유지"));
        assert!(titles.contains(&"외부추가.pdf"));
        assert!(!titles.contains(&"사라짐"));
        // import 된 것은 폴더도 생성/연결됨
        let imported_d = active.iter().find(|d| d.title == "외부추가.pdf").unwrap();
        let fid = imported_d.folder_id.as_deref().unwrap();
        assert_eq!(folder::get(&s, fid).unwrap().name, "신규폴더");
    }

    #[test]
    fn archive_moves_file_to_trash_and_restore_brings_it_back() {
        let root = tmp_dir("cmd_deliv_trash");
        let store_root = root.join("Work Vault").join(".projectManger");
        let mut s = Store::open(store_root.clone()).unwrap();
        let b = business::create(&mut s, "사업A", "si", None).unwrap();
        let d = deliverable::create_file(&mut s, &b.id, None, None, "개요", "그림1.png", 3).unwrap();
        let rel = super::place_deliverable_file(&mut s, &store_root, &d.id, "png", |p| std::fs::write(p, b"x")).unwrap();
        let mirror_abs = super::vault_root_of(&store_root).join(&rel);
        assert!(mirror_abs.is_file());

        // 삭제(휴지통) → 미러에서 사라지고 trash 로 이동
        deliverable::archive(&mut s, &d.id).unwrap();
        super::mirror_trash_deliverable(&s, &store_root, &d.id).unwrap();
        assert!(!mirror_abs.exists(), "미러에서 파일이 사라져야 함(iCloud 반영)");
        assert!(super::deliverable_trash_dir(&store_root, &d.id).join("개요.png").is_file());

        // 복원 → 미러로 되돌아옴
        s.deliverables.get(&d.id).cloned().map(|mut x| { x.archived_at = None; s.deliverables.put(x).unwrap(); });
        super::mirror_restore_deliverable(&mut s, &store_root, &d.id).unwrap();
        assert!(mirror_abs.exists(), "복원 시 미러로 파일이 돌아와야 함");
        assert!(!super::deliverable_trash_dir(&store_root, &d.id).join("개요.png").exists());

        // 영구삭제 → 휴지통 파일 제거
        super::mirror_trash_deliverable(&s, &store_root, &d.id).unwrap();
        super::mirror_purge_deliverable(&store_root, &d.id);
        assert!(!super::deliverable_trash_dir(&store_root, &d.id).exists());
    }

    #[test]
    fn folder_cascade_create_rename_trash_on_disk() {
        use crate::store::ops::folder;
        let root = tmp_dir("cmd_folder_cascade");
        let store_root = root.join("Work Vault").join(".projectManger");
        let mut s = Store::open(store_root.clone()).unwrap();
        let b = business::create(&mut s, "사업A", "si", None).unwrap();
        let vault = super::vault_root_of(&store_root);

        let cat = folder::create(&mut s, &b.id, "deliverable", None, "설계서").unwrap();
        super::mirror_create_folder(&s, &store_root, &cat.id).unwrap();
        assert!(vault.join("사업A").join("산출물").join("설계서").is_dir());

        super::mirror_rename_folder(&s, &store_root, &cat.id, "설계 문서").unwrap();
        folder::rename(&mut s, &cat.id, "설계 문서").unwrap();
        assert!(vault.join("사업A").join("산출물").join("설계 문서").is_dir());
        assert!(!vault.join("사업A").join("산출물").join("설계서").exists());

        super::mirror_trash_folder(&s, &store_root, &cat.id).unwrap();
        assert!(!vault.join("사업A").join("산출물").join("설계 문서").exists());
        assert!(store_root.join("trash").read_dir().unwrap().next().is_some());
    }

    #[test]
    fn rename_deliverable_renames_disk_file_and_updates_path() {
        let root = tmp_dir("cmd_rename_file");
        let store_root = root.join("Work Vault").join(".projectManger");
        let mut s = Store::open(store_root.clone()).unwrap();
        let b = business::create(&mut s, "사업A", "si", None).unwrap();
        let d = deliverable::create_file(&mut s, &b.id, None, None, "옛이름", "그림1.png", 3).unwrap();
        super::place_deliverable_file(&mut s, &store_root, &d.id, "png", |p| std::fs::write(p, b"x")).unwrap();

        super::rename_deliverable_file(&mut s, &store_root, &d.id, "새 이름").unwrap();

        let rel = deliverable::file_path_of(&s, &d.id).unwrap().unwrap();
        assert_eq!(
            std::path::Path::new(&rel).file_name().unwrap().to_str().unwrap(),
            "새 이름.png"
        );
        assert!(super::vault_root_of(&store_root).join(&rel).is_file());
        assert!(!super::vault_root_of(&store_root)
            .join("사업A").join("산출물").join("옛이름.png").exists());
    }

    #[test]
    fn move_deliverable_moves_disk_file_between_folders() {
        use crate::store::ops::folder;
        let root = tmp_dir("cmd_move_file");
        let store_root = root.join("Work Vault").join(".projectManger");
        let mut s = Store::open(store_root.clone()).unwrap();
        let b = business::create(&mut s, "사업A", "si", None).unwrap();
        let cat = folder::create(&mut s, &b.id, "deliverable", None, "설계서").unwrap();
        let d = deliverable::create_file(&mut s, &b.id, None, None, "개요", "그림1.png", 3).unwrap();
        super::place_deliverable_file(&mut s, &store_root, &d.id, "png", |p| std::fs::write(p, b"x")).unwrap();

        super::move_deliverable_file(&mut s, &store_root, &d.id, Some(&cat.id)).unwrap();

        let rel = deliverable::file_path_of(&s, &d.id).unwrap().unwrap();
        assert_eq!(
            rel,
            std::path::Path::new("사업A").join("산출물").join("설계서").join("개요.png").to_string_lossy()
        );
        assert!(super::vault_root_of(&store_root).join(&rel).is_file());
        assert!(!super::vault_root_of(&store_root)
            .join("사업A").join("산출물").join("개요.png").exists());
    }

    #[test]
    fn place_deliverable_file_writes_named_by_title_relative_path() {
        use crate::store::ops::folder;
        let root = tmp_dir("cmd_place");
        let store_root = root.join("Work Vault").join(".projectManger");
        let mut s = Store::open(store_root.clone()).unwrap();
        let b = business::create(&mut s, "사업A", "si", None).unwrap();
        let cat = folder::create(&mut s, &b.id, "deliverable", None, "설계서").unwrap();
        let d = deliverable::create_file(&mut s, &b.id, None, Some(&cat.id), "개요", "그림1.png", 3).unwrap();

        let rel = super::place_deliverable_file(&mut s, &store_root, &d.id, "png", |dest| {
            std::fs::write(dest, b"png")
        })
        .unwrap();

        assert_eq!(rel, std::path::Path::new("사업A").join("산출물").join("설계서").join("개요.png"));
        let abs = super::vault_root_of(&store_root).join(&rel);
        assert!(abs.is_file());
        assert_eq!(
            deliverable::file_path_of(&s, &d.id).unwrap().as_deref(),
            rel.to_str()
        );
    }

    #[test]
    fn place_deliverable_file_does_not_double_extension_when_title_has_ext() {
        let root = tmp_dir("cmd_place_ext");
        let store_root = root.join("Work Vault").join(".projectManger");
        let mut s = Store::open(store_root.clone()).unwrap();
        let b = business::create(&mut s, "사업A", "si", None).unwrap();
        // title 이 이미 확장자 포함(실제 데이터 형태)
        let d = deliverable::create_file(&mut s, &b.id, None, None, "이상탐지 총 로직.png", "그림1.png", 3).unwrap();

        let rel = super::place_deliverable_file(&mut s, &store_root, &d.id, "png", |p| std::fs::write(p, b"x")).unwrap();

        assert_eq!(rel.file_name().unwrap().to_str().unwrap(), "이상탐지 총 로직.png");
    }

    #[test]
    fn deliverable_dir_rel_builds_business_area_folder_chain() {
        use crate::store::ops::folder;
        let mut s = Store::open(tmp_dir("cmd_dir_rel").join(".projectManger")).unwrap();
        let b = business::create(&mut s, "철도청 이상탐지", "si", None).unwrap();
        let root = folder::create(&mut s, &b.id, "deliverable", None, "설계서").unwrap();
        let sub = folder::create(&mut s, &b.id, "deliverable", Some(&root.id), "1차").unwrap();

        let unfiled = super::deliverable_dir_rel(&s, &b.id, None).unwrap();
        assert_eq!(unfiled, std::path::Path::new("철도청 이상탐지").join("산출물"));

        let in_root = super::deliverable_dir_rel(&s, &b.id, Some(&root.id)).unwrap();
        assert_eq!(in_root, std::path::Path::new("철도청 이상탐지").join("산출물").join("설계서"));

        let in_sub = super::deliverable_dir_rel(&s, &b.id, Some(&sub.id)).unwrap();
        assert_eq!(in_sub, std::path::Path::new("철도청 이상탐지").join("산출물").join("설계서").join("1차"));
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
        // 새 미러 레이아웃: 앱 Vault 루트 기준 상대경로로 저장.
        let rel = created[0].file_path.as_deref().unwrap();
        assert_eq!(rel, std::path::Path::new("폴라리스AI").join("산출물").join("report.pdf").to_str().unwrap());
        let abs = super::vault_root_of(&store_root).join(rel);
        assert_eq!(std::fs::read(abs).unwrap(), b"pdf");
    }
}
