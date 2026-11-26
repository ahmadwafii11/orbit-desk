//! Tipe data modul Music. Cermin dari `src/types/music.ts`.

use serde::{
    Serialize,
    Deserialize,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    // Hash diperoleh dari path
    // Jadi akan stabil selama file tidak dipindah
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: u64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PlayerStatus {
    Stopped,
    Playing,
    Paused,
}

// Isi dari `music-library.json`.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct LibraryFile {
    #[serde(default)]
    pub folders: Vec<String>,
    #[serde(default)]
    pub tracks: Vec<Track>,
}

// Isi dari `music-settings.json`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(default)]
pub struct MusicSettings {
    pub volume: f32,
    pub shuffle: bool,
    pub repeat: RepeatMode,
}

impl Default for MusicSettings {
    fn default() -> Self {
        Self {
            volume: 0.8,
            shuffle: false,
            repeat: RepeatMode::Off,
        }
    }
}

// Perpustakaan atau library seperti yang dikirimkan ke frontend.
pub type LibraryDto = LibraryFile;

// Keadaan pemutar. 
// Dikirim setiap kali berubah lewat event `music-state-changed`; 
// Frontend akan menghitung sendiri posisi di antara event.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSnapshot {
    pub status: PlayerStatus,
    pub track: Option<Track>,
    pub position_ms: u64,
    pub volume: f32,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub error: Option<String>, // Pesan galat terakhir (misalnya file tidak dapat diputar)
}