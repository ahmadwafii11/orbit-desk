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
    Audio(String), // Pemutar audio gagal (perangkat tidak ada, format tidak didukung, dll.)
    TrackNotFound(String), // Lagu dengan id berikut tidak terdapat di storage
    FolderNotFound, // Folder musik tidak ada
    FolderAlreadyAdded, // Folder musik berikut sudah terdapat di storage
    Network(String),      // Gagal terhubung ke layanan jaringan / API BMKG
    WeatherParse(String), // Gagal melakukan parsing respons data cuaca dari BMKG
    PlaceNotFound(String), // Kode wilayah / lokasi tidak ditemukan
    TooManyPlaces(usize), // Jumlah lokasi yang tersimpan sudah mencapai batas maksimal
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
            Self::Audio(message) => write!(f, "{message}"),
            Self::TrackNotFound(id) => write!(f, "Lagu tidak ditemukan: {id}"),
            Self::FolderNotFound => write!(f, "Folder tidak ditemukan"),
            Self::FolderAlreadyAdded => write!(f, "Folder sudah ada di perpustakaan"),
            Self::Network(message) => write!(f, "Tidak bisa menghubungi BMKG: {message}"),
            Self::WeatherParse(message) => write!(f, "Data BMKG tidak dikenali: {message}"),
            Self::PlaceNotFound(code) => write!(f, "Lokasi tidak ditemukan: {code}"),
            Self::TooManyPlaces(max) => write!(f, "Maksimal {max} lokasi tersimpan"),
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