use std::sync::Arc;
use tauri::{ipc::Response, AppHandle, Emitter, State};

use crate::core::error::AppError;
use crate::models::music::{LibraryDto, PlayerSnapshot, RepeatMode};
use crate::services::music_service::MusicService;

pub const STATE_EVENT: &str = "music-state-changed";
const LIBRARY_EVENT: &str = "music-library-changed";

// Menampilkan keadaan terbaru ke semua window. 
// Pengirim juga menerimanya:
// snapshot bersifat idempoten, jadi tidak perlu disaring seperti Notes.
fn publish_state(app: &AppHandle, snapshot: &PlayerSnapshot) {
    let _ = app.emit(STATE_EVENT, snapshot);
}

fn publish_library(app: &AppHandle) {
    let _ = app.emit(LIBRARY_EVENT, ());
}

async fn blocking<T, F>(service: &State<'_, Arc<MusicService>>, work: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce(Arc<MusicService>) -> Result<T, AppError> + Send + 'static,
{
    let service = Arc::clone(service);
    tauri::async_runtime::spawn_blocking(move || work(service))
        .await
        .map_err(|e| AppError::Audio(e.to_string()))?
}

#[tauri::command]
pub async fn music_library(service: State<'_, Arc<MusicService>>) -> Result<LibraryDto, AppError> {
    service.library()
}

#[tauri::command]
pub async fn music_state(
    service: State<'_, Arc<MusicService>>,
) -> Result<PlayerSnapshot, AppError> {
    service.snapshot()
}

#[tauri::command]
pub async fn music_add_folder(
    app: AppHandle,
    service: State<'_, Arc<MusicService>>,
    path: String,
) -> Result<LibraryDto, AppError> {
    // Memindai folder = banyak akses disk; jangan di thread async utama.
    let library = blocking(&service, move |s| s.add_folder(&path)).await?;
    publish_library(&app);
    Ok(library)
}

#[tauri::command]
pub async fn music_remove_folder(
    app: AppHandle,
    service: State<'_, Arc<MusicService>>,
    path: String,
) -> Result<LibraryDto, AppError> {
    let library = service.remove_folder(&path)?;
    publish_library(&app);
    publish_state(&app, &service.snapshot()?);
    Ok(library)
}

#[tauri::command]
pub async fn music_rescan(
    app: AppHandle,
    service: State<'_, Arc<MusicService>>,
) -> Result<LibraryDto, AppError> {
    let library = blocking(&service, |s| s.rescan()).await?;
    publish_library(&app);
    Ok(library)
}

/// Cover sebagai byte mentah (ArrayBuffer di JavaScript); kosong jika tidak ada.
#[tauri::command]
pub async fn music_cover(
    service: State<'_, Arc<MusicService>>,
    id: String,
) -> Result<Response, AppError> {
    let cover = blocking(&service, move |s| s.cover(&id)).await?;
    Ok(Response::new(cover.unwrap_or_default()))
}

#[tauri::command]
pub async fn music_play(
    app: AppHandle,
    service: State<'_, Arc<MusicService>>,
    id: String,
    queue: Vec<String>,
) -> Result<PlayerSnapshot, AppError> {
    // Kalau gagal diputar, keadaan (dengan pesan galat) tetap ditampilkan.
    let result = service.play(&id, queue);
    publish_state(&app, &service.snapshot()?);
    result
}

#[tauri::command]
pub async fn music_toggle_pause(
    app: AppHandle,
    service: State<'_, Arc<MusicService>>,
) -> Result<PlayerSnapshot, AppError> {
    let result = service.toggle_pause();
    publish_state(&app, &service.snapshot()?);
    result
}

#[tauri::command]
pub async fn music_stop(
    app: AppHandle,
    service: State<'_, Arc<MusicService>>,
) -> Result<PlayerSnapshot, AppError> {
    let snapshot = service.stop()?;
    publish_state(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn music_next(
    app: AppHandle,
    service: State<'_, Arc<MusicService>>,
) -> Result<PlayerSnapshot, AppError> {
    let result = service.next();
    publish_state(&app, &service.snapshot()?);
    result
}

#[tauri::command]
pub async fn music_previous(
    app: AppHandle,
    service: State<'_, Arc<MusicService>>,
) -> Result<PlayerSnapshot, AppError> {
    let result = service.previous();
    publish_state(&app, &service.snapshot()?);
    result
}

#[tauri::command]
pub async fn music_seek(
    app: AppHandle,
    service: State<'_, Arc<MusicService>>,
    position_ms: u64,
) -> Result<PlayerSnapshot, AppError> {
    let snapshot = service.seek(position_ms)?;
    publish_state(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn music_set_volume(
    app: AppHandle,
    service: State<'_, Arc<MusicService>>,
    volume: f32,
) -> Result<PlayerSnapshot, AppError> {
    let snapshot = service.set_volume(volume)?;
    publish_state(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn music_set_shuffle(
    app: AppHandle,
    service: State<'_, Arc<MusicService>>,
    shuffle: bool,
) -> Result<PlayerSnapshot, AppError> {
    let snapshot = service.set_shuffle(shuffle)?;
    publish_state(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn music_set_repeat(
    app: AppHandle,
    service: State<'_, Arc<MusicService>>,
    repeat: RepeatMode,
) -> Result<PlayerSnapshot, AppError> {
    let snapshot = service.set_repeat(repeat)?;
    publish_state(&app, &snapshot);
    Ok(snapshot)
}