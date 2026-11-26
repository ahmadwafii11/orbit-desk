//! Bertugas untuk memindai folder musik serta membaca metadata lagu (judul, artis, album, durasi, cover) dengan `lofty`.
use std::{
    fs,
    path::{Path, PathBuf},
};
use lofty::{file::AudioFile, file::TaggedFileExt, picture::PictureType, tag::Accessor};

use crate::models::music::Track;

// Format file lagu yang dapat diputar oleh decoder bawaan rodio (symphonia).
const EXTENSIONS: &[&str] = &["mp3", "flac", "ogg", "wav", "m4a"];

// Batas pengaman agar salah memilih folder (misalnya seluruh disk) tidak membuat app menggantung.
const MAX_DEPTH: usize = 10;
const MAX_FILES: usize = 20_000;

// FNV-1a 64 bit: id yang stabil dari path, tanpa menggunakan crate tambahan.
fn hash_path(path: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in path.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn collect(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    if depth > MAX_DEPTH || found.len() >= MAX_FILES {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        if found.len() >= MAX_FILES {
            return;
        }

        // `file_type` tidak mengikuti symlink, jadi tidak ada risiko berputar-putar.
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();

        if file_type.is_dir() {
            collect(&path, depth + 1, found);
        } else if file_type.is_file() && is_audio(&path) {
            found.push(path);
        }
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

// Membaca metadata dalam satu file. 
// Tag yang tidak terbaca tidak menggagalkan: lagu tetap masuk dengan nama file sebagai judul.
pub fn read_track(path: &Path) -> Option<Track> {
    let path_str = path.to_str()?.to_string();
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let mut title = None;
    let mut artist = None;
    let mut album = None;
    let mut duration_ms = 0;

    if let Ok(tagged) = lofty::read_from_path(path) {
        duration_ms = tagged.properties().duration().as_millis() as u64;

        if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
            title = non_empty(tag.title().map(|v| v.into_owned()));
            artist = non_empty(tag.artist().map(|v| v.into_owned()));
            album = non_empty(tag.album().map(|v| v.into_owned()));
        }
    }

    Some(Track {
        id: hash_path(&path_str),
        path: path_str,
        title: title.unwrap_or(stem),
        artist,
        album,
        duration_ms,
    })
}

// Semua lagu akan di `root` (rekursif), terurut artis, album, lalu judul.
pub fn scan_folder(root: &Path) -> Vec<Track> {
    let mut files = Vec::new();
    collect(root, 0, &mut files);

    let mut tracks: Vec<Track> = files.iter().filter_map(|p| read_track(p)).collect();
    sort_tracks(&mut tracks);
    tracks
}

pub fn sort_tracks(tracks: &mut [Track]) {
    let key = |t: &Track| {
        (
            t.artist.clone().unwrap_or_default().to_lowercase(),
            t.album.clone().unwrap_or_default().to_lowercase(),
            t.title.to_lowercase(),
        )
    };

    tracks.sort_by_cached_key(key);
}

// Gambar cover yang tertanam di file (cover depan lebih diutamakan).
pub fn read_cover(path: &Path) -> Option<Vec<u8>> {
    let tagged = lofty::read_from_path(path).ok()?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag())?;
    let pictures = tag.pictures();

    pictures
        .iter()
        .find(|p| p.pic_type() == PictureType::CoverFront)
        .or_else(|| pictures.first())
        .map(|p| p.data().to_vec())
}