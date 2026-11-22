use std::{
    fs,
    path::{
        Path, 
        PathBuf,
    },
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
        MutexGuard,
    },
};

use crate::core::{
    error::AppError,
    time::now_ms,
};
use crate::models::storage::{
    AddReport,
    FileEntry,
    SkippedItem,
    StorageMode,
    StoredFile,
};
use crate::repositories::storage_repository::StorageRepository;

// Batas ukuran satu file yang disalin (dicopy). 
// Di atas ini, menggunakan tautan (path folder atau file disimpan)
const MAX_COPY_BYTES: u64 = 1024 * 1024 * 1024;

pub struct StorageService {
    repo: StorageRepository,
    items: Mutex<Vec<StoredFile>>,
    counter: AtomicU64,
}

/// Id dipakai sebagai nama folder (dan indeks bisa saja diedit tangan).
/// Jadi hanya karakter aman yang diterima.
fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

impl StorageService {
    pub fn new(repo: StorageRepository) -> Result<Self, AppError> {
        let items = repo.load()?;

        Ok(Self {
            repo,
            items: Mutex::new(items),
            counter: AtomicU64::new(0),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Vec<StoredFile>>, AppError> {
        self.items.lock().map_err(|e| AppError::Lock(e.to_string()))
    }

    fn next_id(&self) -> String {
        let sequence = self.counter.fetch_add(1, Ordering::Relaxed);
        format!("{:x}-{:x}", now_ms(), sequence)
    }

    /// Path absolut item di disk.
    fn absolute_path(&self, file: &StoredFile) -> Result<PathBuf, AppError> {
        match file.mode {
            StorageMode::Link => Ok(PathBuf::from(&file.location)),
            StorageMode::Copy => {
                let id_part = file.location.split('/').next().unwrap_or("");
                if !is_safe_id(id_part) {
                    return Err(AppError::Storage("lokasi salinan tidak valid".into()));
                }
                Ok(self.repo.files_dir().join(&file.location))
            }
        }
    }

    fn to_entry(&self, file: StoredFile) -> FileEntry {
        match self.absolute_path(&file) {
            Ok(path) => FileEntry {
                missing: !path.exists(),
                path: path.to_string_lossy().into_owned(),
                file,
            },
            Err(_) => FileEntry {
                missing: true,
                path: String::new(),
                file,
            },
        }
    }

    /// List item, terbaru di atas. 
    /// Status `missing` juga akan diperiksa disini.
    pub fn list(&self) -> Result<Vec<FileEntry>, AppError> {
        let mut items = self.lock()?.clone();
        items.sort_by(|a, b| b.added_at.cmp(&a.added_at));

        Ok(items.into_iter().map(|file| self.to_entry(file)).collect())
    }

    /// Menambahkan beberapa path sekaligus. 
    /// Yang gagal dicatat di `skipped`,
    /// yang lain akan tetap diproses.
    pub fn add_paths(&self, paths: Vec<String>, mode: StorageMode) -> Result<AddReport, AppError> {
        let mut added = Vec::new();
        let mut skipped = Vec::new();

        for raw in paths {
            match self.add_one(&raw, mode) {
                Ok(file) => added.push(self.to_entry(file)),
                Err(error) => skipped.push(SkippedItem {
                    path: raw,
                    reason: error.to_string(),
                }),
            }
        }

        Ok(AddReport { added, skipped })
    }

    fn add_one(&self, raw: &str, requested: StorageMode) -> Result<StoredFile, AppError> {
        let path = Path::new(raw);
        if !path.is_absolute() {
            return Err(AppError::InvalidPath);
        }

        let metadata = fs::metadata(path).map_err(|_| AppError::FileNotFound)?;
        let is_dir = metadata.is_dir();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or(AppError::InvalidPath)?
            .to_string();
        let location_str = path.to_str().ok_or(AppError::InvalidPath)?.to_string();

        // Folder selalu berupa tautan: menyalin folder besar bukan tujuan menu ini.
        let mode = if is_dir { StorageMode::Link } else { requested };
        let size = if is_dir { 0 } else { metadata.len() };

        let id = self.next_id();

        let file = match mode {
            StorageMode::Link => StoredFile {
                id,
                name,
                mode,
                is_dir,
                location: location_str,
                size,
                added_at: now_ms(),
            },
            StorageMode::Copy => {
                if size > MAX_COPY_BYTES {
                    return Err(AppError::TooLargeToCopy(MAX_COPY_BYTES));
                }

                let dir = self.repo.files_dir().join(&id);
                fs::create_dir_all(&dir)
                    .map_err(|e| AppError::Storage(format!("gagal membuat folder salinan: {e}")))?;

                if let Err(error) = fs::copy(path, dir.join(&name)) {
                    let _ = fs::remove_dir_all(&dir);
                    return Err(AppError::Storage(format!("gagal menyalin file: {error}")));
                }

                StoredFile {
                    location: format!("{id}/{name}"),
                    id,
                    name,
                    mode,
                    is_dir,
                    size,
                    added_at: now_ms(),
                }
            }
        };

        let result = self.commit_add(&file);
        if result.is_err() && file.mode == StorageMode::Copy {
            let _ = fs::remove_dir_all(self.repo.files_dir().join(&file.id));
        }
        result.map(|_| file)
    }

    fn commit_add(&self, file: &StoredFile) -> Result<(), AppError> {
        let mut items = self.lock()?;

        if file.mode == StorageMode::Link
            && items
                .iter()
                .any(|i| i.mode == StorageMode::Link && i.location == file.location)
        {
            return Err(AppError::AlreadyAdded);
        }

        items.push(file.clone());
        if let Err(error) = self.repo.save(&items) {
            items.pop();
            return Err(error);
        }
        Ok(())
    }

    /// Hapus dari daftar. 
    /// Salinan ikut dihapus; file asli tidak pernah disentuh.
    pub fn remove(&self, id: &str) -> Result<(), AppError> {
        let removed = {
            let mut items = self.lock()?;
            let index = items
                .iter()
                .position(|i| i.id == id)
                .ok_or_else(|| AppError::ItemNotFound(id.to_string()))?;

            let removed = items.remove(index);
            if let Err(error) = self.repo.save(&items) {
                items.insert(index, removed);
                return Err(error);
            }
            removed
        };

        if removed.mode == StorageMode::Copy && is_safe_id(&removed.id) {
            // Gagal menghapus salinan tidak menggagalkan penghapusan item.
            let _ = fs::remove_dir_all(self.repo.files_dir().join(&removed.id));
        }
        Ok(())
    }

    /// Path absolut item yang masih ada di disk, siap dibuka.
    pub fn existing_path(&self, id: &str) -> Result<PathBuf, AppError> {
        let file = self
            .lock()?
            .iter()
            .find(|i| i.id == id)
            .cloned()
            .ok_or_else(|| AppError::ItemNotFound(id.to_string()))?;

        let path = self.absolute_path(&file)?;
        if !path.exists() {
            return Err(AppError::FileNotFound);
        }
        Ok(path)
    }
}