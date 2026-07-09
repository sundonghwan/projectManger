mod aibridge;
mod commands;
mod config;
mod error;
mod hostkey;
mod secrets;
mod sftp;
mod store;
mod terminal;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir().expect("app_data_dir 를 찾을 수 없음");
            std::fs::create_dir_all(&dir).ok();
            let _ = config::relocate_into_work_vault(&dir);
            let store_root = config::store_root(&dir);
            let mut store = store::Store::open(store_root)
                .unwrap_or_else(|_| store::Store::open(dir.join(".projectManger")).expect("기본 Store 초기화 실패"));
            let _ = commands::migrate_legacy_deliverable_files(&dir, &mut store);
            let _ = commands::migrate_deliverables_to_disk_layout(&mut store);
            // reconcile 은 자동 실행하지 않는다 — 사용자가 산출물 화면의 '새로고침'을 누를 때만 수행.
            let local_root = dir.join("local");
            let local = store::local::LocalStore::open(local_root).expect("LocalStore 초기화 실패");
            app.manage(commands::AppState {
                store: Mutex::new(store),
                local: Mutex::new(local),
            });
            app.manage(terminal::TerminalManager::default());
            app.manage(crate::aibridge::AiBridge::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::business_list,
            commands::business_create,
            commands::business_update,
            commands::business_rename,
            commands::business_archive,
            commands::project_list,
            commands::project_create,
            commands::project_update,
            commands::project_rename,
            commands::project_archive,
            commands::task_list,
            commands::task_create,
            commands::task_update,
            commands::task_move,
            commands::task_archive,
            commands::document_list,
            commands::document_create,
            commands::document_move,
            commands::document_rename,
            commands::document_archive,
            commands::document_get,
            commands::document_set_body,
            commands::document_set_editor_body,
            commands::document_asset_upload,
            commands::document_show_export_folder,
            commands::block_list,
            commands::block_create,
            commands::block_update,
            commands::block_delete,
            commands::label_list,
            commands::label_create,
            commands::label_assign,
            commands::label_unassign,
            commands::task_label_map,
            commands::deliverable_list,
            commands::deliverable_create,
            commands::deliverable_set_status,
            commands::deliverable_add_version,
            commands::deliverable_versions,
            commands::deliverable_archive,
            commands::deliverable_reconcile,
            commands::deliverable_upload,
            commands::deliverable_upload_files,
            commands::deliverable_open,
            commands::deliverable_show_in_folder,
            commands::deliverable_rename,
            commands::deliverable_move,
            commands::folder_list,
            commands::folder_create,
            commands::folder_rename,
            commands::folder_delete,
            commands::memo_list,
            commands::memo_create,
            commands::memo_update,
            commands::memo_set_color,
            commands::memo_set_pinned,
            commands::memo_archive,
            commands::server_list,
            commands::server_create,
            commands::server_update,
            commands::server_archive,
            commands::server_set_secret,
            commands::server_clear_secret,
            commands::server_has_secret,
            commands::snippet_list,
            commands::snippet_create,
            commands::snippet_delete,
            commands::ssh_connect,
            commands::ssh_write,
            commands::ssh_resize,
            commands::ssh_disconnect,
            commands::ssh_host_status,
            commands::ssh_scan_host,
            commands::ssh_trust_host,
            commands::sftp_list,
            commands::template_list,
            commands::template_create,
            commands::template_delete,
            commands::template_apply_project,
            commands::template_apply_document,
            commands::recurring_list,
            commands::recurring_create,
            commands::recurring_set_active,
            commands::recurring_delete,
            commands::recurring_generate,
            commands::search,
            commands::trash_list,
            commands::trash_restore,
            commands::trash_purge,
            commands::vault_path,
            commands::vault_set,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
