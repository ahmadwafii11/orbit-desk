mod core;
mod commands;
mod models;
mod services;
mod repositories;

use services::{
    system_monitor::SystemMonitor,
    notes_service::NotesService,
};
use repositories::notes_repository::NotesRepository;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(SystemMonitor::new())
        .setup(|app| {
            // Folder data app
            // Misalnya ~/.local/share/<identifier>/ di Linux.
            let data_dir = app.path().app_data_dir()?;
            let repo = NotesRepository::new(data_dir.join("notes.json"));
            app.manage(NotesService::new(repo)?);
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");  
}