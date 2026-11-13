mod core;
mod commands;
mod models;
mod services;

use services::system_monitor::SystemMonitor;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(SystemMonitor::new())
        .invoke_handler(tauri::generate_handler![
            commands::system::get_system_info,
            commands::system::get_system_stats,
            commands::widget::open_widget,
            commands::widget::close_widget,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");  
}