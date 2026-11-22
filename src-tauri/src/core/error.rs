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
    Lock(String), // Mutex state beracun (thread lain panik saat memegang kunci).
    Window(String), // Gagal membuat atau mengelola window (jendela).
    UnknownWidget(String), // Modul diminta dibuka sebagai widget, tetapi tidak terdaftar.
    Storage(String), // Gagal membaca atau menulis penyimpanan di disk.
    NoteNotFound(String), // Catatan dengna id ini tidak ada (misalnya sudah dihapus dari jendela lain).
    NoteTooLarge(usize), // Isi catatan melebihi batas ukuran (dalam byte).
    FileNotFound, // File atau folder yang diminta tidak tedapat di disk
    InvalidPath, // Path yang diberikan bukan path absolut
    AlreadyAdded, // Item tautan yang sama sudah terdapat di storage
    TooLargeToCopy(u64), // File terlalu besar untuk dicopy (batas copy dalam byte)
    ItemNotFound(String), // Item dengan suatu id tertentu tidak terdapat dalam storage
    Open(String), // Gagal dalam membuka file atau folder dengan aplikasi bawaan dari sistem
}

impl Display for AppError {
    fn  fmt (&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Lock(message) => write!(f, "Gagal mengunci state: {message}"),
            Self::Window(message) => write!(f, "Gagal mengelola jendela: {message}"),
            Self::UnknownWidget(module) => write!(f, "Widget tidak dikenal: {module}"),
            Self::Storage(message) => write!(f, "Gagal mengakses penyimpanan: {message}"),
            Self::NoteNotFound(id) => write!(f, "Catatan tidak ditemukan: {id}"),
            Self::NoteTooLarge(max) => write!(f, "Catatan terlalu besar (maksimal {} KB)", max / 1024),
            Self::FileNotFound => write!(f, "File tidak ditemukan (mungkin telah dipindah atau sudah dihapus)"),
            Self::InvalidPath => write!(f, "Path harus absolut"),
            Self::AlreadyAdded => write!(f, "Sudah terdapat di storage"),
            Self::TooLargeToCopy(max) => write!(f, "Terlalu besar untuk disalin (maksimal {} MB). Tambahkan sebagai tautan", max / (1024 * 1024)),
            Self::ItemNotFound(id) => write!(f, "Item tidak ditemukan: {id}"),
            Self::Open(message) => write!(f, "Gagal membuka: {message}"),

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