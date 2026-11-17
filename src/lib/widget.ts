import { getCurrentWindow } from "@tauri-apps/api/window";

const WIDGET_LABEL_PREFIX = "widget-";

// Mengembalikan id modul jika window ini adalah widget, atau null jika ini merupakan window utama.
// - Di Tauri: dibaca dari label window (`widget-system` -> "system").
// - Di browser biasa (`pnpm dev`): dibaca dari `?widget=system`, agar widget bisa dilihat tanpa membuka Tauri.
export function detectWidgetModule(): string | null {
  const fromQuery = new URLSearchParams(window.location.search).get("widget");
  if (fromQuery) return fromQuery;

  try {
    const label = getCurrentWindow().label;
    if (label.startsWith(WIDGET_LABEL_PREFIX)) {
      return label.slice(WIDGET_LABEL_PREFIX.length);
    }
  } catch {
    // Tidak berjalan di dalam Tauri.
  }

  return null;
}