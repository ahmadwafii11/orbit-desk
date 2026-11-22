/** Cermin dari `src-tauri/src/models/storage.rs`. */
export type StorageMode = "copy" | "link";

export interface StorageEntry {
  id: string;
  name: string;
  mode: StorageMode;
  isDir: boolean;
  location: string; /* Link: path asli. Copy: path relatif di folder data app. */
  size: number; /* Byte. 0 untuk folder. */
  addedAt: number; /* Milidetik sejak epoch Unix. */
  path: string; /* Path absolut yang bisa dibuka. */
  missing: boolean; /* File sudah tidak ada di disk (dipindah atau dihapus di luar app). */
}

export interface AddReport {
  added: StorageEntry[];
  skipped: { path: string; reason: string }[];
}