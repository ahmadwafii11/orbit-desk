//! Bertugas untuk pemutar audio. 
//! Seluruh objek rodio hidup di satu thread khusus: 
//! Handle perangkat audio tidak boleh berpindah thread di semua platform, 
//! dan dengan begitu memutar musik tidak pernah memblokir UI.
//!
//! Thread ini hanya tahu memutar satu file
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

// Seberapa sering thread memeriksa apakah lagu sudah selesai dan memperbarui posisi.
const TICK: Duration = Duration::from_millis(200);

pub enum EngineEvent {
    Ended(u64), // Lagu dengan generasi ini selesai diputar sampai habis.
}

enum Command {
    Play {
        path: PathBuf,
        volume: f32,
        reply: mpsc::Sender<Result<u64, String>>,
    },
    Pause,
    Resume,
    Stop,
    Seek {
        position: Duration,
        reply: mpsc::Sender<Result<(), String>>,
    },
    Volume(f32),
}

#[derive(Clone, Copy, Default)]
pub struct EngineStatus {
    pub position_ms: u64,
}

pub struct AudioEngine {
    commands: mpsc::Sender<Command>,
    status: Arc<Mutex<EngineStatus>>,
}

fn lock(status: &Mutex<EngineStatus>) -> MutexGuard<'_, EngineStatus> {
    status.lock().unwrap_or_else(|e| e.into_inner())
}

impl AudioEngine {
    /// Mulai thread audio. 
    /// Perangkat audio baru dibuka saat lagu pertama diputar, jadi app tetap jalan di komputer tanpa kartu suara.
    pub fn start() -> (Self, mpsc::Receiver<EngineEvent>) {
        let (commands, command_rx) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        let status = Arc::new(Mutex::new(EngineStatus::default()));

        let thread_status = Arc::clone(&status);
        thread::Builder::new()
            .name("orbit-audio".into())
            .spawn(move || run(command_rx, event_tx, thread_status))
            .expect("gagal memulai thread audio");

        (Self { commands, status }, events)
    }

    fn send(&self, command: Command) -> Result<(), String> {
        self.commands
            .send(command)
            .map_err(|_| "Pemutar audio berhenti bekerja".to_string())
    }

    /// Memutar `path` dari awal. 
    /// Mengembalikan nomor generasi lagu ini.
    pub fn play(&self, path: &Path, volume: f32) -> Result<u64, String> {
        let (reply, answer) = mpsc::channel();
        self.send(Command::Play {
            path: path.to_path_buf(),
            volume,
            reply,
        })?;

        answer
            .recv()
            .map_err(|_| "Pemutar audio berhenti bekerja".to_string())?
    }

    pub fn pause(&self) {
        let _ = self.send(Command::Pause);
    }

    pub fn resume(&self) {
        let _ = self.send(Command::Resume);
    }

    pub fn stop(&self) {
        let _ = self.send(Command::Stop);
    }

    pub fn set_volume(&self, volume: f32) {
        let _ = self.send(Command::Volume(volume));
    }

    pub fn seek(&self, position: Duration) -> Result<(), String> {
        let (reply, answer) = mpsc::channel();
        self.send(Command::Seek { position, reply })?;

        answer
            .recv()
            .map_err(|_| "Pemutar audio berhenti bekerja".to_string())?
    }

    pub fn position_ms(&self) -> u64 {
        lock(&self.status).position_ms
    }
}

/// Catat posisi pemutar sekarang. 
/// Akan dipanggil sebelum membalas perintah Play dan Seek, agar keadaan yang dibaca `MusicService` sesudahnya sudah terbaru.
fn sync_position(status: &Mutex<EngineStatus>, player: &Option<Player>) {
    lock(status).position_ms = player
        .as_ref()
        .map(|p| p.get_pos().as_millis() as u64)
        .unwrap_or(0);
}

fn open_player(
    device: &mut Option<MixerDeviceSink>,
    path: &Path,
    volume: f32,
) -> Result<Player, String> {
    let file = File::open(path).map_err(|e| format!("Gagal membuka file: {e}"))?;
    let source =
        Decoder::try_from(file).map_err(|e| format!("Format audio tidak didukung: {e}"))?;

    if device.is_none() {
        let opened = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| format!("Tidak ada perangkat audio: {e}"))?;
        *device = Some(opened);
    }

    let mixer = match device.as_ref() {
        Some(device) => device.mixer(),
        None => return Err("Tidak ada perangkat audio".to_string()),
    };

    let player = Player::connect_new(mixer);
    player.set_volume(volume);
    player.append(source);
    Ok(player)
}

fn run(
    commands: mpsc::Receiver<Command>,
    events: mpsc::Sender<EngineEvent>,
    status: Arc<Mutex<EngineStatus>>,
) {
    let mut device: Option<MixerDeviceSink> = None; // Handle perangkat harus tetap hidup selama ada yang diputar.
    let mut player: Option<Player> = None;
    let mut generation: u64 = 0;
    let mut active = false; // Terdapat lagu yang sedang berjalan dan belum dilaporkan selesai.

    loop {
        match commands.recv_timeout(TICK) {
            Ok(Command::Play {
                path,
                volume,
                reply,
            }) => {
                // Lagu yang lama akan dihentikan lebih dulu,
                // Walaupun lagu yang baru gagal dibuka.
                if let Some(old) = player.take() {
                    old.stop();
                }
                active = false;

                match open_player(&mut device, &path, volume) {
                    Ok(new_player) => {
                        generation += 1;
                        player = Some(new_player);
                        active = true;
                        sync_position(&status, &player);
                        let _ = reply.send(Ok(generation));
                    }
                    Err(message) => {
                        let _ = reply.send(Err(message));
                    }
                }
            }
            Ok(Command::Pause) => {
                if let Some(p) = &player {
                    p.pause();
                }
            }
            Ok(Command::Resume) => {
                if let Some(p) = &player {
                    p.play();
                }
            }
            Ok(Command::Stop) => {
                if let Some(p) = player.take() {
                    p.stop();
                }
                active = false;
            }
            Ok(Command::Volume(volume)) => {
                if let Some(p) = &player {
                    p.set_volume(volume);
                }
            }
            Ok(Command::Seek { position, reply }) => {
                let result = match &player {
                    Some(p) => p
                        .try_seek(position)
                        .map_err(|e| format!("Tidak bisa melompat di lagu ini: {e:?}")),
                    None => Ok(()),
                };
                sync_position(&status, &player);
                let _ = reply.send(result);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break, // Pemilik engine sudah dilepas: app sedang berhenti.
        }

        sync_position(&status, &player);

        if active && player.as_ref().is_some_and(|p| p.empty()) {
            active = false;
            let _ = events.send(EngineEvent::Ended(generation));
        }
    }
}