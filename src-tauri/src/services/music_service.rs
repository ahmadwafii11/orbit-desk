//! Berguna untuk mengatur Logika Music: perpustakaan (folder + lagu), antrean, dan kendali pemutar.
//! Satu instance dipakai bersama semua window (halaman Music, widget, dan Dashboard), jadi semuanya selalu melihat pemutar yang sama.
use std::{
    path::Path,
    sync::{mpsc::Receiver, Mutex, MutexGuard},
};

use crate::core::{error::AppError, time::now_ms};
use crate::models::music::{
    LibraryDto, 
    LibraryFile, 
    MusicSettings, 
    PlayerSnapshot, 
    PlayerStatus, 
    RepeatMode, 
    Track,
};
use crate::repositories::json_file::JsonFile;
use crate::services::{
    audio_engine::{AudioEngine, EngineEvent},
    library_scanner,
    play_queue::PlayQueue,
};

/// Apabila menekan "previous" setelah lewat detik ini = ulangi lagu, bukan mundur dengan memutar lagu sebelumnya.
const RESTART_THRESHOLD_MS: u64 = 3_000;

struct Inner {
    folders: Vec<String>,
    tracks: Vec<Track>,
    settings: MusicSettings,
    queue: PlayQueue,
    engine: AudioEngine,
    current: Option<Track>,
    status: PlayerStatus,
    generation: u64, /// Generasi lagu yang sedang diputar, berguna untuk mengabaikan laporan "selesai" dari lagu yang sudah diganti.
    error: Option<String>,
}

pub struct MusicService {
    inner: Mutex<Inner>,
    library_file: JsonFile,
    settings_file: JsonFile,
}

impl Inner {
    // Memutar lagu yang sedang ditunjuk sama antrean music.
    fn start_current(&mut self) -> Result<(), AppError> {
        let id = self
            .queue
            .current()
            .map(str::to_string)
            .ok_or_else(|| AppError::TrackNotFound(String::new()))?;

        let track = self
            .tracks
            .iter()
            .find(|t| t.id == id)
            .cloned()
            .ok_or_else(|| AppError::TrackNotFound(id))?;

        match self
            .engine
            .play(Path::new(&track.path), self.settings.volume)
        {
            Ok(generation) => {
                self.generation = generation;
                self.current = Some(track);
                self.status = PlayerStatus::Playing;
                self.error = None;
                Ok(())
            }
            Err(message) => {
                self.current = Some(track);
                self.status = PlayerStatus::Stopped;
                self.error = Some(message.clone());
                Err(AppError::Audio(message))
            }
        }
    }

    fn stop(&mut self) {
        self.engine.stop();
        self.status = PlayerStatus::Stopped;
    }

    fn snapshot(&self) -> PlayerSnapshot {
        PlayerSnapshot {
            status: self.status,
            track: self.current.clone(),
            position_ms: if self.status == PlayerStatus::Stopped {
                0
            } else {
                self.engine.position_ms()
            },
            volume: self.settings.volume,
            shuffle: self.settings.shuffle,
            repeat: self.settings.repeat,
            error: self.error.clone(),
        }
    }
}

fn merge_sorted(mut tracks: Vec<Track>) -> Vec<Track> {
    tracks.sort_by(|a, b| a.id.cmp(&b.id));
    tracks.dedup_by(|a, b| a.id == b.id);
    library_scanner::sort_tracks(&mut tracks);
    tracks
}

fn is_under_any(path: &str, folders: &[String]) -> bool {
    folders.iter().any(|f| Path::new(path).starts_with(f))
}

impl MusicService {
    pub fn new(
        library_file: JsonFile,
        settings_file: JsonFile,
    ) -> Result<(Self, Receiver<EngineEvent>), AppError> {
        let library: LibraryFile = library_file.load()?;
        let settings: MusicSettings = settings_file.load()?;
        let (engine, events) = AudioEngine::start();

        let inner = Inner {
            folders: library.folders,
            tracks: library.tracks,
            settings: MusicSettings {
                volume: settings.volume.clamp(0.0, 1.0),
                ..settings
            },
            queue: PlayQueue::new(now_ms() as u64),
            engine,
            current: None,
            status: PlayerStatus::Stopped,
            generation: 0,
            error: None,
        };

        Ok((
            Self {
                inner: Mutex::new(inner),
                library_file,
                settings_file,
            },
            events,
        ))
    }

    fn lock(&self) -> Result<MutexGuard<'_, Inner>, AppError> {
        self.inner.lock().map_err(|e| AppError::Lock(e.to_string()))
    }

    fn save_library(&self, inner: &Inner) -> Result<(), AppError> {
        self.library_file.save(&LibraryFile {
            folders: inner.folders.clone(),
            tracks: inner.tracks.clone(),
        })
    }

    fn save_settings(&self, inner: &Inner) -> Result<(), AppError> {
        self.settings_file.save(&inner.settings)
    }

    // ---- Library ----

    pub fn library(&self) -> Result<LibraryDto, AppError> {
        let inner = self.lock()?;
        Ok(LibraryFile {
            folders: inner.folders.clone(),
            tracks: inner.tracks.clone(),
        })
    }

    /// Menambah folder lalu memindai isi dari foldernya. 
    /// Pemindaian folder dilakukan tanpa memegang kunci, karena dapat makan waktu jika isi file terlalu banyak atau besar.
    pub fn add_folder(&self, path: &str) -> Result<LibraryDto, AppError> {
        let folder = Path::new(path);
        if !folder.is_absolute() {
            return Err(AppError::InvalidPath);
        }
        if !folder.is_dir() {
            return Err(AppError::FolderNotFound);
        }
        if self.lock()?.folders.iter().any(|f| f == path) {
            return Err(AppError::FolderAlreadyAdded);
        }

        let scanned = library_scanner::scan_folder(folder);

        {
            let mut inner = self.lock()?;
            if inner.folders.iter().any(|f| f == path) {
                return Err(AppError::FolderAlreadyAdded);
            }

            inner.folders.push(path.to_string());
            let existing = std::mem::take(&mut inner.tracks);
            inner.tracks = merge_sorted(existing.into_iter().chain(scanned).collect());
            self.save_library(&inner)?;
        }

        self.library()
    }

    // Menghapus folder dari library. 
    // File musiknya akan tidak disentuh.
    pub fn remove_folder(&self, path: &str) -> Result<LibraryDto, AppError> {
        {
            let mut inner = self.lock()?;
            inner.folders.retain(|f| f != path);

            let folders = inner.folders.clone();
            inner.tracks.retain(|t| is_under_any(&t.path, &folders));

            // Lagu yang sedang diputar akan ikut hilang dari library.
            let playing_removed = inner
                .current
                .as_ref()
                .is_some_and(|c| !inner.tracks.iter().any(|t| t.id == c.id));
            if playing_removed {
                inner.stop();
                inner.current = None;
                inner.queue.clear();
            }

            self.save_library(&inner)?;
        }

        self.library()
    }

    // Memindai ulang semua folder: lagu baru akan muncul, file yang telah dihapus akan hilang.
    pub fn rescan(&self) -> Result<LibraryDto, AppError> {
        let folders = self.lock()?.folders.clone();

        let scanned: Vec<Track> = folders
            .iter()
            .flat_map(|f| library_scanner::scan_folder(Path::new(f)))
            .collect();

        {
            let mut inner = self.lock()?;

            // Folder yang dihapus selama pemindaian tidak akan muncul lagi.
            let current_folders = inner.folders.clone();
            inner.tracks = merge_sorted(
                scanned
                    .into_iter()
                    .filter(|t| is_under_any(&t.path, &current_folders))
                    .collect(),
            );
            self.save_library(&inner)?;
        }

        self.library()
    }

    pub fn cover(&self, id: &str) -> Result<Option<Vec<u8>>, AppError> {
        let path = {
            let inner = self.lock()?;
            inner
                .tracks
                .iter()
                .find(|t| t.id == id)
                .map(|t| t.path.clone())
                .ok_or_else(|| AppError::TrackNotFound(id.to_string()))?
        };

        Ok(library_scanner::read_cover(Path::new(&path)))
    }

    // ---- Pemutar ----

    pub fn snapshot(&self) -> Result<PlayerSnapshot, AppError> {
        Ok(self.lock()?.snapshot())
    }

    // Putar `id`; 
    // `queue` = urutan lagu yang sedang ditampilkan di layar (setelah filter/pencarian), yang akan jadi antrean berikutnya.
    pub fn play(&self, id: &str, queue: Vec<String>) -> Result<PlayerSnapshot, AppError> {
        let mut inner = self.lock()?;

        if !inner.tracks.iter().any(|t| t.id == id) {
            return Err(AppError::TrackNotFound(id.to_string()));
        }

        // Antrean hanya berisi lagu yang masih ada di library.
        let known: std::collections::HashSet<&str> =
            inner.tracks.iter().map(|t| t.id.as_str()).collect();
        let queue: Vec<String> = queue
            .into_iter()
            .filter(|q| known.contains(q.as_str()))
            .collect();

        let shuffle = inner.settings.shuffle;
        if !inner.queue.set(queue, id, shuffle) {
            // Lagu yang dipilih tidak ada di daftar yang dikirim: putar sendiri.
            inner.queue.set(vec![id.to_string()], id, shuffle);
        }

        let result = inner.start_current();
        let snapshot = inner.snapshot();
        result.map(|_| snapshot)
    }

    pub fn toggle_pause(&self) -> Result<PlayerSnapshot, AppError> {
        let mut inner = self.lock()?;

        match inner.status {
            PlayerStatus::Playing => {
                inner.engine.pause();
                inner.status = PlayerStatus::Paused;
            }
            PlayerStatus::Paused => {
                inner.engine.resume();
                inner.status = PlayerStatus::Playing;
            }
            PlayerStatus::Stopped => {
                if inner.queue.current().is_some() {
                    inner.start_current()?;
                }
            }
        }

        Ok(inner.snapshot())
    }

    pub fn stop(&self) -> Result<PlayerSnapshot, AppError> {
        let mut inner = self.lock()?;
        inner.stop();
        Ok(inner.snapshot())
    }

    pub fn next(&self) -> Result<PlayerSnapshot, AppError> {
        let mut inner = self.lock()?;
        let repeat = inner.settings.repeat;

        if inner.queue.advance(repeat, false) {
            inner.start_current()?;
        }

        Ok(inner.snapshot())
    }

    pub fn previous(&self) -> Result<PlayerSnapshot, AppError> {
        let mut inner = self.lock()?;
        let repeat = inner.settings.repeat;

        let restart = inner.current.is_some()
            && (inner.engine.position_ms() > RESTART_THRESHOLD_MS
                || inner.status == PlayerStatus::Stopped
                || !inner.queue.retreat(repeat));

        if restart {
            if inner.status != PlayerStatus::Stopped {
                inner
                    .engine
                    .seek(std::time::Duration::ZERO)
                    .map_err(AppError::Audio)?;
            }
        } else if inner.queue.current().is_some() {
            inner.start_current()?;
        }

        Ok(inner.snapshot())
    }

    pub fn seek(&self, position_ms: u64) -> Result<PlayerSnapshot, AppError> {
        let inner = self.lock()?;

        if inner.status != PlayerStatus::Stopped {
            inner
                .engine
                .seek(std::time::Duration::from_millis(position_ms))
                .map_err(AppError::Audio)?;
        }

        Ok(inner.snapshot())
    }

    pub fn set_volume(&self, volume: f32) -> Result<PlayerSnapshot, AppError> {
        let mut inner = self.lock()?;

        inner.settings.volume = volume.clamp(0.0, 1.0);
        inner.engine.set_volume(inner.settings.volume);
        self.save_settings(&inner)?;

        Ok(inner.snapshot())
    }

    pub fn set_shuffle(&self, shuffle: bool) -> Result<PlayerSnapshot, AppError> {
        let mut inner = self.lock()?;

        inner.settings.shuffle = shuffle;
        inner.queue.set_shuffle(shuffle);
        self.save_settings(&inner)?;

        Ok(inner.snapshot())
    }

    pub fn set_repeat(&self, repeat: RepeatMode) -> Result<PlayerSnapshot, AppError> {
        let mut inner = self.lock()?;

        inner.settings.repeat = repeat;
        self.save_settings(&inner)?;

        Ok(inner.snapshot())
    }

    // Dipanggil thread pendengar saat sebuah lagu selesai. 
    // `true` jika keadaan berubah dan perlu ditampilkan ke window.
    pub fn handle_engine_event(&self, event: EngineEvent) -> bool {
        let EngineEvent::Ended(generation) = event;

        let Ok(mut inner) = self.lock() else {
            return false;
        };

        // Lagu sudah diganti atau dijeda sejak laporan ini dikirim.
        if generation != inner.generation || inner.status != PlayerStatus::Playing {
            return false;
        }

        let repeat = inner.settings.repeat;

        // File yang gagal dibuka dilewati, dengan batas agar tidak berputar selamanya kalau semuanya rusak.
        for _ in 0..inner.queue.len().max(1) {
            if !inner.queue.advance(repeat, true) {
                inner.stop();
                return true;
            }
            if inner.start_current().is_ok() {
                return true;
            }
        }

        inner.stop();
        true
    }
}