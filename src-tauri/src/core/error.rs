use std::fmt::{
    Display,
    Formatter,
    Result,
};
use serde::{
    Serialize,
    Serializer,
};

// Error yang dikembalikan command Tauri ke frontend.
// Dikirim ke Javascript sebagai string biasa. Jadi, di sisi frotend `catch (e)` tetap menerima teks yang sama seperti sebelumnya.
#[derive(Debug)]
pub enum AppError {
    // Mutex state beracun (thread lain panik saat memegang kunci).
    Lock(String),
    
    // Gagal membuat atau mengelola window (jendela).
    Window(String),

    // Modul diminta dibuka sebagai widget, tetapi tidak terdaftar.
    UnknownWidget(String),
}

impl Display for AppError {
    fn  fmt (&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Lock(message) => write!(f, "Gagal mengunci state: {message}"),
            Self::Window(message) => write!(f, "Gagal mengelola jendela: {message}"),
            Self::UnknownWidget(module) => write!(f, "Widget tidak dikenal: {module}"),
        }
    }
}

impl std::error::Error for AppError {}
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S,) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(error: tauri::Error) -> Self {
        Self::Window(error.to_string())
    }
}