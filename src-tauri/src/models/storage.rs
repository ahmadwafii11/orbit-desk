// Tipe data modul Storage. Cermin dari `src/modules/storage/types.ts`
use serde::{
    Deserialize,
    Serialize,
};

// Cara sebuah item disimpan
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StorageMode {
    Copy, // File disalin ke folder data app. Tetap ada walaupun file aslinya dipindah
    Link, // Hanya mencatat lokasi aslinya. Tidak menggunakan ruang disk tambahan
}

// Item seperti yang disimpan di `index.json`
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoredFile {
    pub id: String,
    pub name: String, // Nama file atau folder untuk ditampilkan
    pub mode: StorageMode,
    #[serde(default)]
    pub is_dir: bool,
    pub location: String, // `Link`: path asli. `Copy`: path relatif terhadap folder salinan. (`<id>/<nama>`), agar data tetap valid jika folder data app pindah
    #[serde(default)]
    pub size: u64, // Ukuran dalam byte. 0 untuk folder
    pub added_at: i64, // Milidetik semenjak epoch Unix
}

/// Item seperti yang dikirim ke frontend: ditambah path absolut dan status apakah filenya masih ada di disk.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    #[serde(flatten)]
    pub file: StoredFile,
    pub path: String, /// Path absolut yang bisa dibuka.
    pub missing: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct SkippedItem {
    pub path: String,
    pub reason: String,
}

/// Hasil menambahkan beberapa item sekaligus: 
/// Sebagian bisa berhasil dan sebagian dilewati (sudah ada, terlalu besar, tidak ditemukan, dst.).
#[derive(Serialize, Debug)]
pub struct AddReport {
    pub added: Vec<FileEntry>,
    pub skipped: Vec<SkippedItem>,
}