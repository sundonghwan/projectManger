mod commands;
mod db;
mod error;
mod export;
mod hostkey;
mod models;
mod repo;
mod secrets;
mod sftp;
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
            let db_path = dir.join("projectmanger.sqlite");
            let conn = db::open_at(&db_path).expect("DB 초기화 실패");
            app.manage(commands::AppState {
                db: Mutex::new(conn),
            });
            app.manage(terminal::TerminalManager::default());
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
            commands::deliverable_upload,
            commands::deliverable_rename,
            commands::deliverable_move,
            commands::folder_list,
            commands::folder_create,
            commands::folder_rename,
            commands::folder_delete,
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
            commands::export_json,
            commands::import_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
