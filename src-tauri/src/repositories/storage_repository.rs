use std::{
    fs, 
    io::ErrorKind,
    path::PathBuf,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::core::{
    error:: AppError,
    time::now_ms,
};
use crate::models::storage::StoredFile;

const FORMAT_VERSION: u32 = 1;

#[derive(Deserialize)]
struct IndexFile {
    items: Vec<StoredFile>,
}

#[derive(Serialize)]
struct IndexFileRef<'a> {
    version: u32,
    items: &'a [StoredFile],
}

pub struct StorageRepository {
    root: PathBuf,
}

fn storage_error(context: &str, error: impl std::fmt::Display) -> AppError {
    AppError::Storage(format!("{context}: {error}"))
}

impl StorageRepository {
    /// `root` = folder storage, misalnya `<app_data>/storage`.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn index_path(&self) -> PathBuf {
        self.root.join("index.json")
    }

    /// Folder induk semua salinan.
    pub fn files_dir(&self) -> PathBuf {
        self.root.join("files")
    }

    pub fn load(&self) -> Result<Vec<StoredFile>, AppError> {
        let raw = match fs::read_to_string(self.index_path()) {
            Ok(raw) => raw,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(storage_error("gagal membaca index.json", error)),
        };

        match serde_json::from_str::<IndexFile>(&raw) {
            Ok(file) => Ok(file.items),
            Err(error) => {
                self.quarantine(&error)?;
                Ok(Vec::new())
            }
        }
    }

    pub fn save(&self, items: &[StoredFile]) -> Result<(), AppError> {
        fs::create_dir_all(&self.root)
            .map_err(|e| storage_error("gagal membuat folder storage", e))?;

        let json = serde_json::to_string_pretty(&IndexFileRef {
            version: FORMAT_VERSION,
            items,
        })
        .map_err(|e| storage_error("gagal menyusun JSON", e))?;

        let temp_path = self.root.join("index.json.tmp");
        fs::write(&temp_path, json)
            .map_err(|e| storage_error("gagal menulis file sementara", e))?;
        fs::rename(&temp_path, self.index_path())
            .map_err(|e| storage_error("gagal mengganti index.json", e))
    }

    fn quarantine(&self, reason: &serde_json::Error) -> Result<(), AppError> {
        let backup = self.root.join(format!("index.corrupt-{}.json", now_ms()));

        eprintln!(
            "index.json tidak bisa dibaca ({reason}); disimpan sebagai {}",
            backup.display()
        );

        fs::rename(self.index_path(), &backup)
            .map_err(|e| storage_error("gagal mengamankan index.json yang rusak", e))
    }
}