mod commands;
mod db;
mod error;
mod export;
mod models;
mod repo;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir().expect("app_data_dir 를 찾을 수 없음");
            std::fs::create_dir_all(&dir).ok();
            let db_path = dir.join("projectmanger.sqlite");
            let conn = db::open_at(&db_path).expect("DB 초기화 실패");
            app.manage(commands::AppState {
                db: Mutex::new(conn),
            });
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
            commands::document_rename,
            commands::document_archive,
            commands::block_list,
            commands::block_create,
            commands::block_update,
            commands::block_delete,
            commands::label_list,
            commands::label_create,
            commands::label_assign,
            commands::label_unassign,
            commands::task_label_map,
            commands::search,
            commands::export_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
