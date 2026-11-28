use std::sync::Arc;
use tauri::State;

use crate::core::error::AppError;
use crate::models::weather::{
    PlacesState, 
    WeatherResult, 
    WilayahHit, 
    WilayahInfo, 
    WilayahItem,
};
use crate::services::weather_service::WeatherService;

type Service<'a> = State<'a, Arc<WeatherService>>;

async fn blocking<T, F>(service: &Service<'_>, work: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce(Arc<WeatherService>) -> Result<T, AppError> + Send + 'static,
{
    let service = Arc::clone(service);
    tauri::async_runtime::spawn_blocking(move || work(service))
        .await
        .map_err(|e| AppError::Network(e.to_string()))?
}

#[tauri::command]
pub async fn weather_places(service: Service<'_>) -> Result<PlacesState, AppError> {
    service.places()
}

#[tauri::command]
pub async fn weather_add_place(
    service: Service<'_>,
    code: String,
) -> Result<PlacesState, AppError> {
    service.add_place(&code)
}

#[tauri::command]
pub async fn weather_remove_place(
    service: Service<'_>,
    code: String,
) -> Result<PlacesState, AppError> {
    blocking(&service, move |s| s.remove_place(&code)).await
}

#[tauri::command]
pub async fn weather_set_default(
    service: Service<'_>,
    code: String,
) -> Result<PlacesState, AppError> {
    service.set_default(&code)
}

// `code` kosong = lokasi bawaan. 
// `force` = abaikan cache yang masih segar.
#[tauri::command]
pub async fn weather_get(
    service: Service<'_>,
    code: Option<String>,
    force: bool,
) -> Result<WeatherResult, AppError> {
    blocking(&service, move |s| s.get(code.as_deref(), force)).await // Permintaan jaringan: jangan di thread async utama.
}

#[tauri::command]
pub async fn wilayah_info(service: Service<'_>) -> Result<WilayahInfo, AppError> {
    Ok(service.wilayah_info())
}

#[tauri::command]
pub async fn wilayah_children(
    service: Service<'_>,
    parent: String,
) -> Result<Vec<WilayahItem>, AppError> {
    Ok(service.wilayah_children(&parent))
}

#[tauri::command]
pub async fn wilayah_search(
    service: Service<'_>,
    query: String,
) -> Result<Vec<WilayahHit>, AppError> {
    Ok(service.wilayah_search(&query))
}