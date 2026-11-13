// Mode widget: membuka modul sebagai jendela kecil di desktop.
// Setiap widget adalah jendela Tauri yang terpisah dengan label `widget-<modul>`.
// Frontend mengenali mode widget dari label itu (lihat di `src/main.tsx`)
use tauri::{
    AppHandle,
    Manager,
    WebviewUrl,
    WebviewWindowBuilder,
};

use crate::core::error::AppError;

// Jarak widget dari tepi kanan-atas layar saat pertama dibuka (pixel logic).
const SCREEN_MARGIN_X: f64 = 24.0;
const SCREEN_MARGIN_Y: f64 = 56.0;

// Ukuran awal (lebar, tinggi) per modul.
// Modul yang tidak terdapat disini tidak dapat dibuka sebagai widget.
fn widget_size(module: &str) -> Option<(f64, f64)> {
    match module {
        "system" => Some((288.0, 236.0)),
        _=> None,
    }
}

// Posisi pojok kanan-atas monitor utama, atau `None` jika tidak terbaca.
// (Window kemudian ditaruh di posisi default sistem).
fn top_right_position(app: &AppHandle, width: f64) -> Option<(f64, f64)> {
    let monitor = app.primary_monitor().ok().flatten()?;
    let scale = monitor.scale_factor();

    let screen_x = monitor.position().x as f64 / scale;
    let screen_y = monitor.position().y as f64 / scale;
    let screen_width = monitor.size().width as f64 / scale;

    Some((
        screen_x + screen_width - width - SCREEN_MARGIN_X,
        screen_y + SCREEN_MARGIN_Y,
    ))
}

// Membuka widget untuk `module`
// Jika sudah terbuka, cukup ditampilkan lagi.
pub fn open(app: &AppHandle, module: &str) -> Result<(), AppError> {
    let (width, height) =
        widget_size(module).ok_or_else(|| AppError::UnknownWidget(module.to_string()))?;

    let label = format!("widget-{module}");

    if let Some(existing) = app.get_webview_window(&label) {
        existing.show()?;
        return Ok(());
    }

    let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title(format!("Orbit Desk – {module}"))
        .inner_size(width, height)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_bottom(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
        .focused(false);

    if let Some((x, y)) = top_right_position(app, width) {
        builder = builder.position(x, y);
    }

    builder.build()?;
    Ok(())
}