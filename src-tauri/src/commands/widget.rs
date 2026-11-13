use tauri::{
    AppHandle, 
    WebviewWindow,
};

use crate::core::error::AppError;
use crate::services::widget_window;

// Membuka widget untuk `module`.
// Dibuat `async` karena membuat window dari command sinkron bisa menyebabkan deadlock di OS Windows.
#[tauri::command]
pub async  fn open_widget(app: AppHandle, module: String) -> Result<(), AppError> {
    widget_window::open(&app, &module)
}

// Menutup window widget yang memanggil command ini.
#[tauri::command]
pub fn close_widget(window: WebviewWindow) -> Result<(), AppError> {
    window.close()?;
    Ok(())
}