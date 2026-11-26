import type { Track } from "../types";

// 83000 -> "1:23", 3723000 -> "1:02:03".
export function formatClock(ms: number): string {
    const totalSeconds = Math.max(0, Math.floor(ms / 1000));
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = String(totalSeconds % 60).padStart(2, "0");

    return hours > 0
        ? `${hours}:${String(minutes).padStart(2, "0")}:${seconds}`
        : `${minutes}:${seconds}`;
}

export function artistOf(track: Track): string {
    return track.artist ?? "Unknown artist";
}

export function matchesQuery(track: Track, query: string): boolean {
    const q = query.trim().toLowerCase();
    if (q === "") return true;

    return [track.title, track.artist, track.album].some((field) =>
        field?.toLowerCase().includes(q),
    );
}

// "/home/me/Music" -> "Music"
export function folderName(path: string): string {
    const parts = path.split(/[/\\]/).filter(Boolean);
    return parts[parts.length - 1] ?? path;
}