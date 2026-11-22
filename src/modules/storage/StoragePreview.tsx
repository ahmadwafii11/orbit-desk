import { useStorageStore } from "../../stores/storageStore";
import { useStorageSync } from "../../hooks/useStorageSync";

import { formatBytes } from "../../lib/format";
import { copiedBytes } from "../../lib/storage";


/** Isi kartu "Temporary Storage" di Dashboard. */
export function StoragePreview() {
  useStorageSync();

  const items = useStorageStore((s) => s.items);

  const copies = items.filter((i) => i.mode === "copy").length;
  const description =
    items.length === 0
      ? "Add your important files"
      : copies > 0
        ? `${formatBytes(copiedBytes(items))} in ${copies} ${copies === 1 ? "copy" : "copies"}`
        : "All links, no extra space";

  return (
    <>
      <h2>
        {items.length} <small>{items.length === 1 ? "item" : "items"}</small>
      </h2>

      <p className="widget-description">{description}</p>
    </>
  );
}