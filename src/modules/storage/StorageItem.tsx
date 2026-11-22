import { useEffect, useState } from "react";
import { Check, Copy, FolderOpen, Trash2 } from "lucide-react";

import { useStorageStore } from "../../stores/storageStore";

import { formatBytes } from "../../lib/format";
import { formatRelativeTime } from "../../lib/time";
import { CATEGORY_ICONS, categoryOf, parentFolder } from "../../lib/storage";

import type { StorageEntry } from "../../types";

/** Waktu tombol "Copied" / konfirmasi hapus kembali ke keadaan semula. */
const FEEDBACK_MS = 2500;

interface StorageItemProps {
  item: StorageEntry;
}

export function StorageItem({ item }: StorageItemProps) {
  const open = useStorageStore((s) => s.open);
  const reveal = useStorageStore((s) => s.reveal);
  const remove = useStorageStore((s) => s.remove);

  const [copied, setCopied] = useState(false);
  const [confirming, setConfirming] = useState(false);

  useEffect(() => {
    if (!copied) return;
    const timer = window.setTimeout(() => setCopied(false), FEEDBACK_MS);
    return () => window.clearTimeout(timer);
  }, [copied]);

  useEffect(() => {
    if (!confirming) return;
    const timer = window.setTimeout(() => setConfirming(false), FEEDBACK_MS);
    return () => window.clearTimeout(timer);
  }, [confirming]);

  const Icon = CATEGORY_ICONS[categoryOf(item)];

  const copyPath = async () => {
    try {
      await navigator.clipboard.writeText(item.path);
      setCopied(true);
    } catch {
      // Clipboard ditolak oleh webview; tidak ada yang bisa dilakukan.
    }
  };

  const onRemove = () => {
    // Menghapus salinan itu permanen, jadi minta klik kedua.
    if (item.mode === "copy" && !confirming) {
      setConfirming(true);
      return;
    }
    void remove(item.id);
  };

  const origin =
    item.mode === "link" ? parentFolder(item.path) : "Copied to Orbit Desk";

  return (
    <>
      <li className={`storage-item${item.missing ? " is-missing" : ""}`}>
        <button
          className="storage-item-main"
          onClick={() => void open(item.id)}
          disabled={item.missing}
          title={item.missing ? "File not found" : `Open ${item.name}`}
        >
          <span className="storage-item-icon">
            <Icon size={17} strokeWidth={1.6} />
          </span>

          <span className="storage-item-text">
            <strong>{item.name}</strong>
            <span>
              {item.missing && (
                <em className="storage-badge-missing">Missing</em>
              )}
              {!item.isDir && `${formatBytes(item.size)} · `}
              {formatRelativeTime(item.addedAt)} · {origin}
            </span>
          </span>

          <span className={`storage-badge storage-badge-${item.mode}`}>
            {item.mode === "copy" ? "Copy" : "Link"}
          </span>
        </button>

        <div className="storage-item-actions">
          <button
            onClick={() => void reveal(item.id)}
            disabled={item.missing}
            title="Show in folder"
            aria-label="Show in folder"
          >
            <FolderOpen size={14} />
          </button>

          <button
            onClick={() => void copyPath()}
            disabled={item.path === ""}
            title="Copy path"
            aria-label="Copy path"
          >
            {copied ? <Check size={14} /> : <Copy size={14} />}
          </button>

          <button
            className={confirming ? "is-danger" : undefined}
            onClick={onRemove}
            title={
              item.mode === "copy"
                ? "Delete the stored copy (the original is untouched)"
                : "Remove from list (the file itself is untouched)"
            }
            aria-label="Remove"
          >
            {confirming ? <span>Delete?</span> : <Trash2 size={14} />}
          </button>
        </div>
      </li>
    </>
  );
}