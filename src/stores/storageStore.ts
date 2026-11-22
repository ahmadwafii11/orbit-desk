import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";

import type { AddReport, StorageEntry, StorageMode } from "../types/storage";

const STORAGE_CHANGED_EVENT = "storage-changed";
const MODE_KEY = "orbit.storage.addMode";

export type LoadState = "idle" | "loading" | "ready" | "error";

interface StorageState {
  items: StorageEntry[];
  loadState: LoadState;
  addMode: StorageMode; // Mode untuk file yang ditambahkan berikutnya. Folder selalu tautan.
  adding: boolean;
  error: string | null;
  notice: string | null; // Info non-fatal, misalnya sebagian file dilewati saat menambah.

  load: () => Promise<void>;
  reload: () => Promise<void>;
  setAddMode: (mode: StorageMode) => void;
  addPaths: (paths: string[]) => Promise<void>;
  pick: (kind: "files" | "folder") => Promise<void>;
  remove: (id: string) => Promise<void>;
  open: (id: string) => Promise<void>;
  reveal: (id: string) => Promise<void>;
  dismissMessages: () => void;
}

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function sortItems(items: StorageEntry[]): StorageEntry[] {
  return [...items].sort((a, b) => b.addedAt - a.addedAt);
}

function readMode(): StorageMode {
  try {
    return localStorage.getItem(MODE_KEY) === "link" ? "link" : "copy";
  } catch {
    return "copy";
  }
}

function writeMode(mode: StorageMode) {
  try {
    localStorage.setItem(MODE_KEY, mode);
  } catch {
    // Penyimpanan browser tidak tersedia: preferensi hanya berlaku sesi ini.
  }
}

/** Hasil dialog bisa berupa satu path, banyak path, atau null (dibatalkan). */
function toPaths(result: string | string[] | null): string[] {
  if (result === null) return [];
  return Array.isArray(result) ? result : [result];
}

function summarizeSkipped(skipped: AddReport["skipped"]): string | null {
  if (skipped.length === 0) return null;

  const reasons = [...new Set(skipped.map((s) => s.reason))].join("; ");
  return `${skipped.length} ${skipped.length === 1 ? "item" : "items"} skipped: ${reasons}`;
}

export const useStorageStore = create<StorageState>((set, get) => ({
  items: [],
  loadState: "idle",
  addMode: readMode(),
  adding: false,
  error: null,
  notice: null,

  load: async () => {
    const { loadState } = get();
    if (loadState === "loading" || loadState === "ready") return;

    set({ loadState: "loading" });

    try {
      const items = await invoke<StorageEntry[]>("list_storage");
      set({ items, loadState: "ready", error: null });
    } catch (error) {
      set({ loadState: "error", error: errorText(error) });
    }
  },

  // Tidak menghapus `error`: reload sering dipanggil tepat setelah sebuah aksi gagal, dan pesannya tidak boleh hilang sebelum sempat dibaca.
  reload: async () => {
    try {
      const items = await invoke<StorageEntry[]>("list_storage");
      set({ items, loadState: "ready" });
    } catch (error) {
      set({ error: errorText(error) });
    }
  },

  setAddMode: (mode) => {
    writeMode(mode);
    set({ addMode: mode });
  },

  addPaths: async (paths) => {
    const unique = [...new Set(paths)].filter((p) => p !== "");
    if (unique.length === 0) return;

    set({ adding: true, error: null, notice: null });

    try {
      const report = await invoke<AddReport>("add_storage_paths", {
        paths: unique,
        mode: get().addMode,
      });

      set((state) => ({
        items: sortItems([...report.added, ...state.items]),
        notice: summarizeSkipped(report.skipped),
      }));
    } catch (error) {
      set({ error: errorText(error) });
    } finally {
      set({ adding: false });
    }
  },

  pick: async (kind) => {
    try {
      const result = await open({
        multiple: true,
        directory: kind === "folder",
      });
      await get().addPaths(toPaths(result));
    } catch (error) {
      set({ error: errorText(error) });
    }
  },

  remove: async (id) => {
    try {
      await invoke("remove_storage_item", { id });
      set((state) => ({ items: state.items.filter((i) => i.id !== id) }));
    } catch (error) {
      set({ error: errorText(error) });
      // Daftar lokal mungkin sudah usang (mis. dihapus dari jendela lain).
      await get().reload();
    }
  },

  open: async (id) => {
    try {
      await invoke("open_storage_item", { id });
    } catch (error) {
      set({ error: errorText(error) });
      // Penyebab paling umum: file hilang. Segarkan status "missing".
      await get().reload();
    }
  },

  reveal: async (id) => {
    try {
      await invoke("reveal_storage_item", { id });
    } catch (error) {
      set({ error: errorText(error) });
      await get().reload();
    }
  },

  dismissMessages: () => set({ error: null, notice: null }),
}));

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
 * Dengarkan perubahan dari window lain. 
 * Aman dipanggil dari beberapa komponen: 
 * listener dipasang sekali dan dilepas saat pemakai terakhir berhenti. 
 * Mengembalikan fungsi berhenti.
 */
export function startStorageSync(): () => void {
  syncUsers += 1;

  if (syncUsers === 1) {
    unlistenPromise = listen<{ source: string }>(STORAGE_CHANGED_EVENT, (event) => {
      if (event.payload.source !== ownWindowLabel()) {
        void useStorageStore.getState().reload();
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