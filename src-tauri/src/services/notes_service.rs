// Logic Notes
// Seluruh daftar disimpan di memori (di balik Mutex) dan ditulis ke disk melalui `NotesRepository` setiap kali terdapat perubahan.
// Satu instance dipakai bersama oleh semua window (utama maupun widget).
// Jadi semuanya dapat selalu melihat data yang sama.
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, 
        MutexGuard,
    },
    time::{
        SystemTime,
        UNIX_EPOCH
    },
};

use crate::core::error::AppError;
use crate::models::notes::Note;
use crate::repositories::notes_repository::NotesRepository;

// Batas ukuran satu notes.
// Seluruh file ditulis ulang di setiap autosave.
// Jadi catatan (notes) raksasa (semisal salah paste) akan membuat app menjadi tersendat.
const MAX_NOTE_BYTES: usize = 1024 * 1024;

pub struct NotesService{
    repo: NotesRepository,
    notes: Mutex<Vec<Note>>,
    counter: AtomicU64,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn find_mut<'a>(notes: &'a mut [Note], id: &str) -> Result<&'a mut Note, AppError> {
    notes
        .iter_mut()
        .find(|note| note.id == id)
        .ok_or_else(|| AppError::NoteNotFound(id.to_string()))
}

impl  NotesService {
    pub fn new(repo: NotesRepository) -> Result<Self, AppError> {
        let notes = repo.load()?;

        Ok(Self {
            repo,
            notes: Mutex::new(notes),
            counter: AtomicU64::new(0),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Vec<Note>>, AppError> {
        self.notes.lock().map_err(|e| AppError::Lock(e.to_string()))
    }

    // Id unik :
    // Waktu pembuatan + penghitung, tanpa memerlukan crate tambahan.
    fn next_id(&self, now: i64) -> String {
        let sequence = self.counter.fetch_add(1, Ordering::Relaxed);
        format!("{now:x}-{sequence:x}")
    }

    // Semua notes
    // Yang disematkan terlebih dahulu, lalu yang terbaru diedit.
    pub fn list(&self) -> Result<Vec<Note>, AppError> {
        let mut sorted = self.lock()?.clone();

        sorted.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });

        Ok(sorted)
    }

    pub fn create(&self) -> Result<Note, AppError> {
        let mut notes = self.lock()?;
        let now = now_ms();

        let note = Note {
            id: self.next_id(now),
            content: String::new(),
            pinned: false,
            created_at: now,
            updated_at: now,
        };

        notes.push(note.clone());
        self.repo.save(&notes)?;

        Ok(note)
    }

    pub fn save_content(&self, id: &str, content: String) -> Result<Note, AppError> {
        if content.len() > MAX_NOTE_BYTES {
            return Err(AppError::NoteTooLarge(MAX_NOTE_BYTES));
        }

        let mut notes = self.lock()?;
        
        let note = find_mut(&mut notes, id)?;
        note.content = content;
        note.updated_at = now_ms();
        let updated = note.clone();

        self.repo.save(&notes)?;

        Ok(updated)
    }

    // Menyematkan bukan mengedit note
    // Jadi `updated_at` tidak mengalami perubahan
    pub fn set_pinned(&self, id: &str, pinned: bool) -> Result<Note, AppError> {
        let mut notes = self.lock()?;

        let note = find_mut(&mut notes, id)?;
        note.pinned = pinned;
        let updated = note.clone();

        self.repo.save(&notes)?;

        Ok(updated)
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let mut notes = self.lock()?;

        let before = notes.len();
        notes.retain(|note| note.id != id);

        if notes.len() == before {
            return Err(AppError::NoteNotFound(id.to_string()));
        }

        self.repo.save(&notes)
    }
}