// Mode widget: membuka modul sebagai jendela kecil di desktop.
// Setiap widget adalah jendela Tauri yang terpisah dengan label `widget-<modul>`.
// Frontend mengenali mode widget dari label itu (lihat di `src/main.tsx`)
use tauri::{
    AppHandle, 
    Manager, 
    WebviewUrl, 
    WebviewWindow,
    WebviewWindowBuilder,
};

use crate::core::error::AppError;

// Jarak widget dari tepi kanan-atas layar saat pertama dibuka (pixel logic).
const SCREEN_MARGIN_X: f64 = 24.0;
const SCREEN_MARGIN_Y: f64 = 56.0;

// Ukuran awal dan posisi sebyah widget.
struct WidgetSpec {
    width: f64,
    height: f64,
    offset_x: f64, // Gesekan ke kiri dari tepi kanan layar agar beberapa widget yang dibuka bersamaan tidak saling menumpuk
    offset_y: f64, // Geseran ke bawah dari tepi atas, untuk widget yang ditumpuk di bawah widget lain.
}

// Modul yang tidak terdapat disini tidak dapat dibuka sebagai widget.
fn widget_spec(module: &str) -> Option<WidgetSpec> {
    match module {
        "system" => Some(WidgetSpec { 
            width: 288.0, 
            height: 236.0, 
            offset_x: 0.0,
            offset_y: 0.0, 
        }),

        // Di sebelah kiri widget System (lebar 280 + jarak 16).
        "notes" => Some(WidgetSpec { 
            width: 320.0, 
            height: 380.0, 
            offset_x: 296.0, 
            offset_y: 0.0,
        }),
        
        // Di sebelah kiri widget Notes (296 + lebar 320 + jarak 16).
        "storage" => Some(WidgetSpec {
            width: 320.0,
            height: 380.0,
            offset_x: 632.0,
            offset_y: 0.0,
        }),
        // Di bawah widget System (tinggi 236 + jarak 16), satu kolom dengannya.
        "music" => Some(WidgetSpec {
            width: 280.0,
            height: 224.0,
            offset_x: 0.0,
            offset_y: 252.0,
        }),
        _=> None,
    }
}

// Posisi pojok kanan-atas monitor utama, atau `None` jika tidak terbaca.
// (Window kemudian ditaruh di posisi default sistem).
fn top_right_position(app: &AppHandle, spec: &WidgetSpec) -> Option<(f64, f64)> {
    let monitor = app.primary_monitor().ok().flatten()?;
    let scale = monitor.scale_factor();

    let screen_x = monitor.position().x as f64 / scale;
    let screen_y = monitor.position().y as f64 / scale;
    let screen_width = monitor.size().width as f64 / scale;

    Some((
        screen_x + screen_width - spec.width - SCREEN_MARGIN_X - spec.offset_x,
        screen_y + SCREEN_MARGIN_Y + spec.offset_y,
    ))
}

// Membuka widget untuk `module`
// Jika sudah terbuka, cukup ditampilkan lagi.
pub fn open(app: &AppHandle, module: &str) -> Result<(), AppError> {
    let spec = widget_spec(module).ok_or_else(|| AppError::UnknownWidget(module.to_string()))?;

    let label = format!("widget-{module}");

    if let Some(existing) = app.get_webview_window(&label) {
        existing.show()?;
        return Ok(());
    }

    let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title(format!("Orbit Desk – {module}"))
        .inner_size(spec.width, spec.height)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_bottom(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
        .focused(false);

    if let Some((x, y)) = top_right_position(app, &spec) {
        builder = builder.position(x, y);
    }

    builder.build()?;
    Ok(())
}

/// Memindahkan widget ke lapisan atas 
/// (`floating = true`, mengambang di atas window lain) atau kembali ke lapisan bawah (menempel di desktop).
/// Dua mode ini saling meniadakan, jadi yang lama dimatikan lebih dulu.
pub fn set_layer(window: &WebviewWindow, floating: bool) -> Result<(), AppError> {
    if floating {
        window.set_always_on_bottom(false)?;
        window.set_always_on_top(true)?;
    } else {
        window.set_always_on_top(false)?;
        window.set_always_on_bottom(true)?;
    }

    Ok(())
}