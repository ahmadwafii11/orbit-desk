import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Pin } from "lucide-react";

// Membuka modul `moduleId` sebagai widget desktop terpisah.
export function PinWidgetButton({ moduleId }: { moduleId: string }) {
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState(false);

  const open = async () => {
    setBusy(true);
    setFailed(false);

    try {
      await invoke("open_widget", { module: moduleId });
    } catch (error) {
      console.error(error);
      setFailed(true);
    } finally {
      setBusy(false);
    }
  };

  return (
    <button
      className="pin-button"
      onClick={open}
      disabled={busy}
      title="Show this module as a desktop widget"
    >
      <Pin size={13} />
      {failed ? "Failed to open" : "Pin to desktop"}
    </button>
  );
}
