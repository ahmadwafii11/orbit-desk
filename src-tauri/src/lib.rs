mod core;
mod commands;
mod models;
mod services;
mod repositories;

use std::sync::Arc;

use services::{
    system_monitor::SystemMonitor,
    notes_service::NotesService,
    storage_service::StorageService,
};
use repositories::{
    notes_repository::NotesRepository,
    storage_repository::StorageRepository,
};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(SystemMonitor::new())
        .setup(|app| {
            // Folder data app
            // Misalnya ~/.local/share/<identifier>/ di Linux.
            let data_dir = app.path().app_data_dir()?;
            let repo = NotesRepository::new(data_dir.join("notes.json"));
            app.manage(NotesService::new(repo)?);
            let storage = StorageRepository::new(data_dir.join("storage"));
            app.manage(Arc::new(StorageService::new(storage)?));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::get_system_info,
            commands::system::get_system_stats,
            commands::widget::open_widget,
            commands::widget::close_widget,
            commands::widget::set_widget_layer,
            commands::notes::list_notes,
            commands::notes::create_note,
            commands::notes::save_note,
            commands::notes::set_note_pinned,
            commands::notes::delete_note,
            commands::storage::list_storage,
            commands::storage::add_storage_paths,
            commands::storage::remove_storage_item,
            commands::storage::open_storage_item,
            commands::storage::reveal_storage_item,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");  
}