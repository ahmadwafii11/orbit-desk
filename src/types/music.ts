// Cermin dari `src-tauri/src/models/music.rs`
export interface Track {
    id: string;
    path: string;
    title: string;
    artist: string | null;
    album: string | null;
    durationMs: number;
}

export type RepeatMode = "off" | "all" | "one";
export type PlayerStatus = "stopped" | "playing" | "paused";

export interface PlayerSnapshot {
    status: PlayerStatus;
    track: Track | null;
    positionMs: number;
    volume: number; // 0 sampai 1
    shuffle: boolean;
    repeat: RepeatMode;
    error: string | null; // Pesan galat terakhir, misalnya file tidak bisa diputar
}

export interface Library {
    folders: string[];
    tracks: Track[];
}