mod core;
mod commands;
mod models;
mod services;
mod repositories;

#[cfg(test)]
mod tests;

use std::{
    sync::Arc,
    path::PathBuf,
};

use services::{
    system_monitor::SystemMonitor,
    notes_service::NotesService,
    storage_service::StorageService,
    music_service::MusicService,
    weather_service::WeatherService,
    wilayah_index::WilayahIndex,
};
use repositories::{
    notes_repository::NotesRepository,
    storage_repository::StorageRepository,
    json_file::JsonFile,
};
use tauri::{
    Emitter, 
    Manager,
    path::BaseDirectory, 
};

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
            let (music, engine_events) = MusicService::new(
                JsonFile::new(data_dir.join("music-library.json")),
                JsonFile::new(data_dir.join("music-settings.json")),
            )?;
            let music = Arc::new(music);
            app.manage(Arc::clone(&music));

            // Lagu selesai -> lanjut ke berikutnya lalu memberi tahukan ke semua window
            // Thread ini akan berhenti sendiri jika engine dilepas
            let handle = app.handle().clone();
            std::thread::Builder::new()
                .name("orbit-music-events".into())
                .spawn(move || {
                    for event in engine_events {
                        if music.handle_engine_event(event) {
                            if let Ok(snapshot) = music.snapshot() {
                                let _ = handle.emit(commands::music::STATE_EVENT, snapshot);
                            }
                        }
                    }
                })?;

            // Data wilayah: saat dev dibaca langsung dari folder proyek (tanpa build ulang), saat rilis dari resource yang dibundel.
            let wilayah_dir: PathBuf = if cfg!(debug_assertions) {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/region_data")
            } else {
                app.path()
                    .resolve("resources/region_data", BaseDirectory::Resource)?
            };
            let weather = WeatherService::new(
                WilayahIndex::load_dir(&wilayah_dir),
                JsonFile::new(data_dir.join("weather-places.json")),
                JsonFile::new(data_dir.join("weather-cache.json")),
            )?;
            app.manage(Arc::new(weather));

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
            commands::music::music_library,
            commands::music::music_state,
            commands::music::music_add_folder,
            commands::music::music_remove_folder,
            commands::music::music_rescan,
            commands::music::music_cover,
            commands::music::music_play,
            commands::music::music_toggle_pause,
            commands::music::music_stop,
            commands::music::music_next,
            commands::music::music_previous,
            commands::music::music_seek,
            commands::music::music_set_volume,
            commands::music::music_set_shuffle,
            commands::music::music_set_repeat,
            commands::weather::weather_places,
            commands::weather::weather_add_place,
            commands::weather::weather_remove_place,
            commands::weather::weather_set_default,
            commands::weather::weather_get,
            commands::weather::wilayah_info,
            commands::weather::wilayah_children,
            commands::weather::wilayah_search,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");  
}