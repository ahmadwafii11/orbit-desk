import { Archive, File, FileCode, FileText, Film, Folder, Image as ImageIcon, Music2, type LucideIcon } from "lucide-react";

import type { StorageEntry } from "../types/storage";

export type Category =
  | "folder"
  | "image"
  | "document"
  | "video"
  | "audio"
  | "archive"
  | "code"
  | "other";

export const CATEGORY_LABELS: Record<Category, string> = {
  folder: "Folders",
  image: "Images",
  document: "Documents",
  video: "Videos",
  audio: "Audio",
  archive: "Archives",
  code: "Code",
  other: "Other",
};

export const CATEGORY_ICONS: Record<Category, LucideIcon> = {
  folder: Folder,
  image: ImageIcon,
  document: FileText,
  video: Film,
  audio: Music2,
  archive: Archive,
  code: FileCode,
  other: File,
};

/** Urutan tampil chip filter. */
export const CATEGORY_ORDER: Category[] = [
  "document",
  "image",
  "video",
  "audio",
  "archive",
  "code",
  "folder",
  "other",
];

const EXTENSIONS: Record<Exclude<Category, "folder" | "other">, string[]> = {
  image: ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "heic", "avif", "ico"],
  document: [
    "pdf", "doc", "docx", "odt", "txt", "md", "rtf",
    "xls", "xlsx", "ods", "csv", "ppt", "pptx", "odp", "epub",
  ],
  video: ["mp4", "mkv", "mov", "avi", "webm"],
  audio: ["mp3", "wav", "flac", "ogg", "m4a", "aac"],
  archive: ["zip", "rar", "7z", "tar", "gz", "xz", "bz2"],
  code: [
    "js", "ts", "tsx", "jsx", "json", "py", "rs", "go", "java", "c", "cpp",
    "h", "html", "css", "sh", "yml", "yaml", "toml", "sql",
  ],
};

const CATEGORY_BY_EXTENSION = new Map<string, Category>(
  Object.entries(EXTENSIONS).flatMap(([category, list]) =>
    list.map((ext) => [ext, category as Category] as const),
  ),
);

/** "Report.Final.PDF" -> "pdf". Tanpa ekstensi (atau ".bashrc") -> "". */
export function extensionOf(name: string): string {
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

export function categoryOf(entry: Pick<StorageEntry, "name" | "isDir">): Category {
  if (entry.isDir) return "folder";
  return CATEGORY_BY_EXTENSION.get(extensionOf(entry.name)) ?? "other";
}

export function matchesQuery(entry: StorageEntry, query: string): boolean {
  const q = query.trim().toLowerCase();
  return q === "" || entry.name.toLowerCase().includes(q);
}

/** Total ukuran salinan saja: tautan tidak memakai ruang disk app. */
export function copiedBytes(items: StorageEntry[]): number {
  return items.reduce((sum, i) => (i.mode === "copy" ? sum + i.size : sum), 0);
}

/** Folder asal untuk ditampilkan, mis. "/home/me/Documents/a.pdf" -> "/home/me/Documents". */
export function parentFolder(path: string): string {
  const index = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return index > 0 ? path.slice(0, index) : path;
}