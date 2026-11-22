import { useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

/**
 * Menerima file yang di-drag dari file manager ke window. 
 * Tauri yang menangani drag-drop native 
 * Jadi kita mendapat path asli (bukan objek File milik browser). 
 * Mengembalikan `true` selama ada file di atas window.
 */
export function useFileDrop(onDrop: (paths: string[]) => void): boolean {
  const [hovering, setHovering] = useState(false);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | null = null;

    getCurrentWebviewWindow()
      .onDragDropEvent((event) => {
        const payload = event.payload;

        if (payload.type === "enter" || payload.type === "over") {
          setHovering(true);
        } else if (payload.type === "drop") {
          setHovering(false);
          onDrop(payload.paths);
        } else {
          setHovering(false);
        }
      })
      .then((fn) => {
        // Komponen sudah dilepas sebelum listener siap: langsung lepas.
        if (cancelled) fn();
        else unlisten = fn;
      })
      .catch(() => {});

    return () => {
      cancelled = true;
      unlisten?.();
    };
    // `onDrop` sengaja tidak jadi dependensi: pemanggil memberi fungsi stabil.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return hovering;
}