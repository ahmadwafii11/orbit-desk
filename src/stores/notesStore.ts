import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type { Note } from "../types";

// Jeda setelah ketikan terakhir sebelum note disimpan ke disk.
const SAVE_DELAY_MS = 600;

const NOTES_CHANGED_EVENT = "notes-changed";

export type LoadState = "idle" | "loading" | "ready" | "error";
export type SaveState = "saved" | "saving" | "error";

interface NotesState {
    notes: Note[];
    selectedId: string | null;
    loadState: LoadState;
    saveState: SaveState;
    error: string | null;

    load: () => Promise<void>;
    reload: () => Promise<void>;
    select: (id: string) => void;
    create: () => Promise<void>;
    updateContent: (id: string, content: string) => void;
    togglePin: (id: string) => Promise<void>;
    remove: (id: string) => Promise<void>;
    flush: () => Promise<void>; // Menyimpan semua perubahan yang masih menunggu sekarang juga.
}

// Urutan yang sama dengan Rust: 
// Disematkan terlebih dahulu, lalu yang terbaru diedit.
function sortNotes(notes: Note[]): Note[] {
  return [...notes].sort(
    (a, b) => Number(b.pinned) - Number(a.pinned) || b.updatedAt - a.updatedAt,
  );
}

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const useNotesStore = create<NotesState>((set, get) => {
  const dirty = new Set<string>(); // Catatan yang punya perubahan belum tersimpan.
  const timers = new Map<string, number>(); // Timer autosave per catatan (debounce).
  
  // Semua penyimpanan dijalankan berurutan. 
  // Tanpa ini, dua panggilan save_note yang berdekatan bisa selesai terbalik di sisi Rust sehingga isi yang lebih lama menimpa yang lebih baru.
  let saveQueue: Promise<void> = Promise.resolve();

  const clearTimer = (id: string) => {
    const timer = timers.get(id);
    if (timer !== undefined) {
      window.clearTimeout(timer);
      timers.delete(id);
    }
  };

  const markSavedIfIdle = () => {
    if (dirty.size === 0 && timers.size === 0) set({ saveState: "saved" });
  };

  const persist = (id: string): Promise<void> => {
    saveQueue = saveQueue.then(async () => {
      if (!dirty.has(id)) return;

      // Membaca isi terbaru saat giliran ini tiba, bukan saat dijadwalkan.
      const note = get().notes.find((n) => n.id === id);
      dirty.delete(id);

      if (!note) {
        markSavedIfIdle();
        return;
      }

      try {
        const saved = await invoke<Note>("save_note", {
          id,
          content: note.content,
        });

        // Mengambil hanya waktu edit dari server. 
        // Isi tidak ditimpa, karena pengguna mungkin sudah mengetik lagi selama menunggu.
        set((state) => ({
          notes: sortNotes(
            state.notes.map((n) =>
              n.id === saved.id ? { ...n, updatedAt: saved.updatedAt } : n,
            ),
          ),
        }));
        markSavedIfIdle();
      } catch (error) {
        dirty.add(id);
        set({ saveState: "error", error: errorText(error) });
      }
    });

    return saveQueue;
  };

  return {
    notes: [],
    selectedId: null,
    loadState: "idle",
    saveState: "saved",
    error: null,

    load: async () => {
      const { loadState } = get();
      if (loadState === "loading" || loadState === "ready") return;

      set({ loadState: "loading" });

      try {
        const notes = await invoke<Note[]>("list_notes");
        set({
          notes,
          selectedId: notes[0]?.id ?? null,
          loadState: "ready",
          error: null,
        });
      } catch (error) {
        set({ loadState: "error", error: errorText(error) });
      }
    },

    // Mengambil ulang daftar dari Rust 
    // (Misalnya setelah window lain mengubah catatan), tanpa menimpa ketikan yang belum tersimpan di jendela ini.
    reload: async () => {
      try {
        const fresh = await invoke<Note[]>("list_notes");
        const local = new Map(get().notes.map((n) => [n.id, n]));
        const freshIds = new Set(fresh.map((n) => n.id));

        // Catatan yang dihapus dari jendela lain tidak perlu disimpan lagi.
        for (const id of [...dirty]) {
          if (!freshIds.has(id)) {
            dirty.delete(id);
            clearTimer(id);
          }
        }

        const merged = fresh.map((n) =>
          dirty.has(n.id)
            ? { ...n, content: local.get(n.id)?.content ?? n.content }
            : n,
        );

        const { selectedId } = get();

        set({
          notes: merged,
          selectedId:
            selectedId && freshIds.has(selectedId)
              ? selectedId
              : (merged[0]?.id ?? null),
          loadState: "ready",
          error: null,
        });
      } catch (error) {
        set({ error: errorText(error) });
      }
    },

    select: (id) => {
      const { selectedId, notes } = get();
      if (id === selectedId) return;

      set({ selectedId: id });

      // Note baru yang ditinggalkan dalam keadaan kosong dibuang, agar daftar tidak penuh "New Note".
      const previous = notes.find((n) => n.id === selectedId);
      if (previous && previous.content.trim() === "") {
        void get().remove(previous.id);
      }
    },

    create: async () => {
      // Sudah terdapat catatan kosong yang sedang dibuka: pakai itu saja.
      const current = get().notes.find((n) => n.id === get().selectedId);
      if (current && current.content.trim() === "") return;

      try {
        const note = await invoke<Note>("create_note");
        set((state) => ({
          notes: sortNotes([note, ...state.notes]),
          selectedId: note.id,
        }));
      } catch (error) {
        set({ error: errorText(error) });
      }
    },

    updateContent: (id, content) => {
      set((state) => ({
        notes: state.notes.map((n) =>
          n.id === id ? { ...n, content, updatedAt: Date.now() } : n,
        ),
        saveState: "saving",
      }));

      dirty.add(id);
      clearTimer(id);
      timers.set(
        id,
        window.setTimeout(() => {
          timers.delete(id);
          void persist(id);
        }, SAVE_DELAY_MS),
      );
    },

    togglePin: async (id) => {
      const note = get().notes.find((n) => n.id === id);
      if (!note) return;

      try {
        const saved = await invoke<Note>("set_note_pinned", {
          id,
          pinned: !note.pinned,
        });

        set((state) => ({
          notes: sortNotes(
            state.notes.map((n) =>
              n.id === id ? { ...n, pinned: saved.pinned } : n,
            ),
          ),
        }));
      } catch (error) {
        set({ error: errorText(error) });
      }
    },

    remove: async (id) => {
      clearTimer(id);
      dirty.delete(id);

      try {
        await invoke("delete_note", { id });
      } catch (error) {
        set({ error: errorText(error) });
        await get().reload(); // Keadaan lokal mungkin tidak sinkron dengan Rust; mengambil ulang.
        return;
      }

      set((state) => {
        const notes = state.notes.filter((n) => n.id !== id);

        return {
          notes,
          selectedId:
            state.selectedId === id ? (notes[0]?.id ?? null) : state.selectedId,
        };
      });

      markSavedIfIdle();
    },

    flush: async () => {
      timers.forEach((timer) => window.clearTimeout(timer));
      timers.clear();

      [...dirty].forEach((id) => {
        void persist(id);
      });

      await saveQueue;
    },
  };
});

// ---------------------------------------------------------------------------
// Sinkronisasi antar window
// ---------------------------------------------------------------------------

function ownWindowLabel(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    return "";
  }
}

let syncUsers = 0;
let unlistenPromise: Promise<() => void> | null = null;

/**
 * Mulai mendengarkan perubahan catatan dari window lain. 
 * Aman dipanggil dari beberapa komponen sekaligus: 
 * Listener hanya dipasang sekali dan dilepas saat pemakai terakhir berhenti. 
 * Mengembalikan fungsi berhenti.
 */
export function startNotesSync(): () => void {
  syncUsers += 1;

  if (syncUsers === 1) {
    unlistenPromise = listen<{ source: string }>(NOTES_CHANGED_EVENT, (event) => {
      // Perubahan dari jendela ini sendiri sudah tercermin di state.
      if (event.payload.source !== ownWindowLabel()) {
        void useNotesStore.getState().reload();
      }
    }).catch(() => () => {});
  }

  return () => {
    syncUsers -= 1;

    if (syncUsers === 0 && unlistenPromise) {
      void unlistenPromise.then((unlisten) => unlisten());
      unlistenPromise = null;
    }
  };
}