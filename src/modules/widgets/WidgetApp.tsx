import type { ComponentType } from "react";

import { NotesDesktopWidget } from "./NotesDesktopWidget";
import { StorageDesktopWidget } from "./StorageDesktopWidget";
import { SystemDesktopWidget } from "./SystemDesktopWidget";

// Untuk menambah widget baru: buat komponennya, daftarkan di sini, lalu
// tambahkan ukurannya di `widget_spec` (src-tauri/src/services/widget_window.rs).
const widgets: Record<string, ComponentType> = {
  system: SystemDesktopWidget,
  notes: NotesDesktopWidget,
  storage: StorageDesktopWidget,
};

export function WidgetApp({ moduleId }: { moduleId: string }) {
  const Widget = widgets[moduleId];

  if (!Widget) {
    return <div className="desktop-widget">Unknown widget: {moduleId}</div>;
  }

  return <Widget />;
}