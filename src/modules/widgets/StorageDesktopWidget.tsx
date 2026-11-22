import { useEffect } from "react";
import { FolderOpen, Plus, Upload } from "lucide-react";

import { useFileDrop } from "../../hooks/useFileDrop";
import { useStorageSync } from "../../hooks/useStorageSync";

import { formatBytes } from "../../lib/format";
import { formatRelativeTime } from "../../lib/time";
import { CATEGORY_ICONS, categoryOf } from "../../lib/storage";

import { useStorageStore } from "../../stores/storageStore";
import type { StorageEntry } from "../../types";

import { DesktopWidgetFrame } from "../../components/widget/DesktopWidgetFrame";

function addPathsFromDrop(paths: string[]) {
  void useStorageStore.getState().addPaths(paths);
}

function Row({ item }: { item: StorageEntry }) {
  const open = useStorageStore((s) => s.open);
  const reveal = useStorageStore((s) => s.reveal);

  const Icon = CATEGORY_ICONS[categoryOf(item)];

  return (
    <>
      <li className={`desktop-storage-row${item.missing ? " is-missing" : ""}`}>
        <button
          className="desktop-storage-open"
          onClick={() => void open(item.id)}
          disabled={item.missing}
          title={item.missing ? "File not found" : item.path}
        >
          <Icon size={15} strokeWidth={1.6} />

          <span className="desktop-storage-text">
            <strong>{item.name}</strong>
            <span>
              {item.missing
                ? "Missing"
                : `${item.isDir ? "Folder" : formatBytes(item.size)} · ${formatRelativeTime(item.addedAt)}`}
            </span>
          </span>
        </button>

        <button
          className="desktop-storage-reveal"
          onClick={() => void reveal(item.id)}
          disabled={item.missing}
          title="Show in folder"
          aria-label="Show in folder"
        >
          <FolderOpen size={13} />
        </button>
      </li>
    </>
  );
}

/**
 * Widget Storage: daftar file yang disimpan, klik untuk membuka.
 * File dapat ditambah melalui drop ke widget atau tombol "+".
 * Menghapus item hanya terdapat di aplikasi utama.
 */
export function StorageDesktopWidget() {
  useStorageSync();
  const dragging = useFileDrop(addPathsFromDrop);

  const items = useStorageStore((s) => s.items);
  const loadState = useStorageStore((s) => s.loadState);
  const adding = useStorageStore((s) => s.adding);
  const error = useStorageStore((s) => s.error);
  const notice = useStorageStore((s) => s.notice);
  const pick = useStorageStore((s) => s.pick);
  const dismissMessages = useStorageStore((s) => s.dismissMessages);

  const message = error ?? notice;

  // Pesan hilang sendiri; widget tidak punya tombol tutup untuk pesan.
  useEffect(() => {
    if (!message) return;

    const timer = window.setTimeout(dismissMessages, 5000);
    return () => window.clearTimeout(timer);
  }, [message, dismissMessages]);

  return (
    <>
      <DesktopWidgetFrame title="STORAGE" interactive>
        <div className="desktop-storage-toolbar">
          <span className="desktop-storage-count">
            {items.length} {items.length === 1 ? "item" : "items"}
          </span>

          <button
            className="desktop-storage-add"
            onClick={() => void pick("files")}
            disabled={adding}
            title="Add files"
            aria-label="Add files"
          >
            <Plus size={14} />
          </button>
        </div>

        {message && (
          <p className={`desktop-storage-message${error ? " is-error" : ""}`}>
            {message}
          </p>
        )}

        {items.length > 0 ? (
          <ul className="desktop-storage-list">
            {items.map((item) => (
              <Row key={item.id} item={item} />
            ))}
          </ul>
        ) : (
          <div className="desktop-storage-empty">
            <Upload size={22} strokeWidth={1.3} />
            <p>
              {loadState === "error"
                ? "Could not load storage"
                : loadState === "ready"
                  ? "Drop files here to keep them within reach"
                  : "Loading..."}
            </p>
          </div>
        )}

        {dragging && (
          <div className="desktop-storage-drop">
            <Upload size={22} strokeWidth={1.4} />
            <span>Drop to add</span>
          </div>
        )}
      </DesktopWidgetFrame>
    </>
  );
}
