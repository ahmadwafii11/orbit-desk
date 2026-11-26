//! Satu file JSON di disk, dengan penulisan atomik (tmp + rename)
//! File yang rusak akan dipindahkan ke `<nama>.corrupt-<waktu>.json`
use std::{fs, io::ErrorKind, path::PathBuf};
use serde::{de::DeserializeOwned, Serialize};

use crate::core::{error::AppError, time::now_ms};

pub struct JsonFile {
    path: PathBuf,
}

fn storage_error(context: &str, error: impl std::fmt::Display) -> AppError {
    AppError::Storage(format!("{context}: {error}"))
}

impl JsonFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    // File yang belum ada atau rusak akan diisikan dengan nilai bawaan
    pub fn load<T: DeserializeOwned + Default>(&self) -> Result<T, AppError> {
        let raw = match fs::read_to_string(&self.path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(T::default()),
            Err(error) => {
                return Err(storage_error(
                    &format!("gagal membaca {}", self.file_name()),
                    error,
                ))
            }
        };

        match serde_json::from_str::<T>(&raw) {
            Ok(value) => Ok(value),
            Err(error) => {
                self.quarantine(&error)?;
                Ok(T::default())
            }
        }
    }

    pub fn save<T: Serialize>(&self, value: &T) -> Result<(), AppError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| storage_error("gagal membuat folder data", e))?;
        }

        let json =
            serde_json::to_string(value).map_err(|e| storage_error("gagal menyusun JSON", e))?;

        let temp_path = self.path.with_extension("json.tmp");
        fs::write(&temp_path, json)
            .map_err(|e| storage_error("gagal menulis file sementara", e))?;
        fs::rename(&temp_path, &self.path)
            .map_err(|e| storage_error(&format!("gagal mengganti {}", self.file_name()), e))
    }

    fn quarantine(&self, reason: &serde_json::Error) -> Result<(), AppError> {
        let stem = self
            .path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let backup = self
            .path
            .with_file_name(format!("{stem}.corrupt-{}.json", now_ms()));

        eprintln!(
            "{} tidak bisa dibaca ({reason}); disimpan sebagai {}",
            self.file_name(),
            backup.display()
        );

        fs::rename(&self.path, &backup).map_err(|e| {
            storage_error(
                &format!("gagal mengamankan {} yang rusak", self.file_name()),
                e,
            )
        })
    }
}