use std::sync::Arc;
use serde::Serialize;
use tauri::{
    AppHandle,
    Emitter,
    State,
    WebviewWindow,
};
use tauri_plugin_opener::OpenerExt;

use crate::core::error::AppError;
use crate::models::storage::{
    AddReport,
    FileEntry,
    StorageMode,
};
use crate::services::storage_service::StorageService;

const STORAGE_CHANGED_EVENT: &str = "storage-changed";

/// `source` = window yang mengubah; 
/// Window tersebut mengabaikan event-nya sendiri.
#[derive(Clone, Serialize)]
struct StorageChanged {
    source: String,
}

fn notify_changed(window: &WebviewWindow) {
    let payload = StorageChanged {
        source: window.label().to_string(),
    };
    let _ = window.emit(STORAGE_CHANGED_EVENT, payload);
}

#[tauri::command]
pub async fn list_storage(
    service: State<'_, Arc<StorageService>>,
) -> Result<Vec<FileEntry>, AppError> {
    let service = Arc::clone(&service);
    // Memeriksa keberadaan file = akses disk; 
    // Jangan di thread async utama.
    tauri::async_runtime::spawn_blocking(move || service.list())
        .await
        .map_err(|e| AppError::Storage(e.to_string()))?
}

#[tauri::command]
pub async fn add_storage_paths(
    window: WebviewWindow,
    service: State<'_, Arc<StorageService>>,
    paths: Vec<String>,
    mode: StorageMode,
) -> Result<AddReport, AppError> {
    let service = Arc::clone(&service);
    // Menyalin file bisa lama: jalankan di thread blocking.
    let report = tauri::async_runtime::spawn_blocking(move || service.add_paths(paths, mode))
        .await
        .map_err(|e| AppError::Storage(e.to_string()))??;

    if !report.added.is_empty() {
        notify_changed(&window);
    }
    Ok(report)
}

#[tauri::command]
pub async fn remove_storage_item(
    window: WebviewWindow,
    service: State<'_, Arc<StorageService>>,
    id: String,
) -> Result<(), AppError> {
    let service = Arc::clone(&service);
    tauri::async_runtime::spawn_blocking(move || service.remove(&id))
        .await
        .map_err(|e| AppError::Storage(e.to_string()))??;

    notify_changed(&window);
    Ok(())
}

#[tauri::command]
pub async fn open_storage_item(
    app: AppHandle,
    service: State<'_, Arc<StorageService>>,
    id: String,
) -> Result<(), AppError> {
    let path = service.existing_path(&id)?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|e| AppError::Open(e.to_string()))
}

#[tauri::command]
pub async fn reveal_storage_item(
    app: AppHandle,
    service: State<'_, Arc<StorageService>>,
    id: String,
) -> Result<(), AppError> {
    let path = service.existing_path(&id)?;
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|e| AppError::Open(e.to_string()))
}