import { useMemo, useState, type ChangeEvent } from "react";
import { Folder, FolderPlus, HardDrive, Plus, Search, Upload, X } from "lucide-react";

import { useFileDrop } from "../../hooks/useFileDrop";
import { useStorageSync } from "../../hooks/useStorageSync";

import { formatBytes } from "../../lib/format";
import {
  CATEGORY_LABELS,
  CATEGORY_ORDER,
  categoryOf,
  copiedBytes,
  matchesQuery,
  type Category,
} from "../../lib/storage";

import { useStorageStore } from "../../stores/storageStore";
import { StorageItem } from "./StorageItem";

import { SectionHeading } from "../../components/layout/SectionHeading";
import { PinWidgetButton } from "../../components/widget/PinWidgetButton";

type Filter = "all" | Category;

function addPathsFromDrop(paths: string[]) {
  void useStorageStore.getState().addPaths(paths);
}

export function Storage() {
  useStorageSync();
  const dragging = useFileDrop(addPathsFromDrop);

  const items = useStorageStore((s) => s.items);
  const loadState = useStorageStore((s) => s.loadState);
  const addMode = useStorageStore((s) => s.addMode);
  const adding = useStorageStore((s) => s.adding);
  const error = useStorageStore((s) => s.error);
  const notice = useStorageStore((s) => s.notice);
  const setAddMode = useStorageStore((s) => s.setAddMode);
  const pick = useStorageStore((s) => s.pick);
  const dismissMessages = useStorageStore((s) => s.dismissMessages);

  const [filter, setFilter] = useState<Filter>("all");
  const [query, setQuery] = useState("");

  const counts = useMemo(() => {
    const result = new Map<Category, number>();
    for (const item of items) {
      const category = categoryOf(item);
      result.set(category, (result.get(category) ?? 0) + 1);
    }
    return result;
  }, [items]);

  // Filter yang kategorinya sudah kosong (item terakhirnya dihapus) kembali ke "all".
  const activeFilter: Filter =
    filter !== "all" && !counts.has(filter) ? "all" : filter;

  const visible = useMemo(
    () =>
      items.filter(
        (item) =>
          (activeFilter === "all" || categoryOf(item) === activeFilter) &&
          matchesQuery(item, query),
      ),
    [items, activeFilter, query],
  );

  const copies = items.filter((i) => i.mode === "copy").length;
  const message = error ?? notice;

  return (
    <>
      <SectionHeading
        eyebrow="LOCAL FILE MANAGEMENT"
        title="Storage."
        subtitle="Keep important files within reach."
        action={
          <div className="heading-actions">
            <PinWidgetButton moduleId="storage" />

            <button
              className="secondary-button"
              onClick={() => void pick("folder")}
              disabled={adding}
            >
              <FolderPlus size={15} /> Add folder
            </button>

            <button
              className="primary-button"
              onClick={() => void pick("files")}
              disabled={adding}
            >
              <Plus size={15} /> {adding ? "Adding..." : "Add file"}
            </button>
          </div>
        }
      />

      <div className="widget storage-overview">
        <HardDrive size={22} />

        <div className="storage-overview-text">
          <h2>
            {items.length} {items.length === 1 ? "item" : "items"}
          </h2>
          <p>
            {copies > 0
              ? `${formatBytes(copiedBytes(items))} used by ${copies} stored ${copies === 1 ? "copy" : "copies"}`
              : "Nothing copied yet. Links take no extra space"}
          </p>
        </div>

        <div className="mode-toggle" role="group" aria-label="How files are added">
          <button
            className={addMode === "copy" ? "is-active" : undefined}
            onClick={() => setAddMode("copy")}
            title="Keep a copy inside Orbit Desk. Still available if you move the original"
          >
            Copy
          </button>
          <button
            className={addMode === "link" ? "is-active" : undefined}
            onClick={() => setAddMode("link")}
            title="Only remember where the file is. Uses no extra space"
          >
            Link
          </button>
        </div>
      </div>

      {message && (
        <div className={`storage-message${error ? " is-error" : ""}`}>
          <span>{message}</span>
          <button onClick={dismissMessages} aria-label="Dismiss">
            <X size={13} />
          </button>
        </div>
      )}

      {items.length > 0 && (
        <div className="storage-toolbar">
          <div className="storage-chips">
            <button
              className={activeFilter === "all" ? "is-active" : undefined}
              onClick={() => setFilter("all")}
            >
              All <span>{items.length}</span>
            </button>

            {CATEGORY_ORDER.filter((c) => counts.has(c)).map((category) => (
              <button
                key={category}
                className={activeFilter === category ? "is-active" : undefined}
                onClick={() => setFilter(category)}
              >
                {CATEGORY_LABELS[category]} <span>{counts.get(category)}</span>
              </button>
            ))}
          </div>

          <label className="storage-search">
            <Search size={13} />
            <input
              value={query}
              onChange={(e: ChangeEvent<HTMLInputElement>) => setQuery(e.target.value)}
              placeholder="Search"
              spellCheck={false}
            />
          </label>
        </div>
      )}

      {loadState === "loading" && items.length === 0 ? (
        <div className="empty-state">
          <p>Loading...</p>
        </div>
      ) : items.length === 0 ? (
        <div className="empty-state">
          <Folder size={38} strokeWidth={1.2} />
          <h3>Your storage is empty</h3>
          <p>
            Click Add file, or drop files here, so you never have to hunt for
            them again.
          </p>
        </div>
      ) : visible.length === 0 ? (
        <div className="empty-state">
          <p>No items match.</p>
        </div>
      ) : (
        <ul className="storage-list">
          {visible.map((item) => (
            <StorageItem key={item.id} item={item} />
          ))}
        </ul>
      )}

      {dragging && (
        <div className="storage-drop-overlay">
          <Upload size={30} strokeWidth={1.4} />
          <strong>Drop to add</strong>
          <span>
            Files are {addMode === "copy" ? "copied" : "linked"}. Folders are always linked
          </span>
        </div>
      )}
    </>
  );
}