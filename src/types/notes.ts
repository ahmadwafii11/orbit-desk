// Cermin dari struct Note di src-tauri/src/models/notes.rs
export interface Note {
    id: string;
    content: string;
    pinned: boolean;
    createdAt: number; // Milidetik sejak epoch Unix.
    updatedAt: number; // Milidetik sejak epoch Unix. Hanya dapat berubah saat diedit.
}