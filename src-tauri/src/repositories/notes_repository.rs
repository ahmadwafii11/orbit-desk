// Penyimpanan notes sebagai satu file JSON.
// Penulisan atomik: ditulis ke file sementara lalu di-rename.
// Jadi file tidak pernah setengah tertulis jika app mati ditengah jalan.
// File yang rusak tidak dibuang melainkan dipindahkan ke `notes.corrupt-<waktu>.json` agar masih dapat diperiksa atau dipulihkan lagi secara manual.
use std::{
    fs,
    io::ErrorKind,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH}
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::core::error::AppError;
use crate::models::notes::Note;

// Versi format file, untuk migrasi di masa depan.
const FORMAT_VERSION: u32 = 1;

#[derive(Deserialize)]
struct NotesFile {
    notes: Vec<Note>,
}

#[derive(Serialize)]
struct NotesFileRef<'a>{
    version: u32,
    notes: &'a [Note],
}

pub struct NotesRepository {
    path: PathBuf,
}

fn storage_error(context: &str, error: impl std::fmt::Display) -> AppError {
    AppError::Storage(format!("{context}: {error}"))
}

impl NotesRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    // Membaca semua catatan.
    // Jika file belum ada = daftar kosong.
    pub fn load(&self) -> Result<Vec<Note>, AppError> {
        let raw = match fs::read_to_string(&self.path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(storage_error("gagal membaca notes.json", error)),
        };

        match serde_json::from_str::<NotesFile>(&raw) {
            Ok(file) => Ok(file.notes),
            Err(error) => {
                self.quarantine(&error)?;
                Ok(Vec::new())
            }
        }
    }

    // Mneulis semua catatan ke disk secara atomik.
    pub fn save(&self, notes: &[Note]) -> Result<(), AppError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| storage_error("gagal membuat folder data", e))?;
        }

        let json = serde_json::to_string_pretty(&NotesFileRef {
            version: FORMAT_VERSION,
            notes,
        })
        .map_err(|e| storage_error("gagal menyusun JSON", e))?;
        
        let temp_path = self.path.with_extension("json.tmp");
        fs::write(&temp_path, json)
            .map_err(|e| storage_error("gagal menulis file sementara", e))?;
        fs::rename(&temp_path, &self.path)
            .map_err(|e| storage_error("gagal mengganti notes.json", e))?;

        Ok(())
    }

    // Memindahkan file yang tidak dapat dibaca ke samping, agar tidak tertimpa.
    fn quarantine(&self, reason: &serde_json::Error) -> Result<(), AppError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let backup_path = self
            .path
            .with_file_name(format!("notes.corrupt-{timestamp}.json"));

        eprintln!(
            "notes.json tidak dapat dibaca ({reason}); disimpan sebagai {}",
            backup_path.display()
        );

        fs::rename(&self.path, &backup_path)
            .map_err(|e| storage_error("gagal mengamankan notes.json yang rusak", e))
    }
}