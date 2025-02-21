use commands::*;
use models::TauriState;
use std::sync::RwLock;
pub mod commands;
pub mod helper;
pub mod loader;
pub mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("Start running Tauri");
    let tauri_state = RwLock::new(TauriState::default());
    tauri::Builder::default()
        .manage(tauri_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            select_file,
            prepare_models,            
            query_file_path,
            query_node,
            request_rename,
            request_delete,
            request_add,
            request_update_algorithm,
            request_can_expand_toggling,
            log
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
