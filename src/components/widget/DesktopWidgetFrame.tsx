import { useState } from "react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Layers, X } from "lucide-react";

interface DesktopWidgetFrameProps {
  title: string;
  children: ReactNode;
  interactive?: boolean; // Widget yang berisi kontrol (ketik, klik). Hanya header yang menjadi area drag. Jika false (default, untuk widget yang hanya menampilkan data), seluruh permukaan bisa dipakai untuk menggeser.
}

// Bingkai umum widget desktop: 
// - latar semi-transparan 
// - Area drag, 
// - Tombol lapisan (menempel di desktop / mengambang di atas jendela lain), dan tombol close. 
// - Tombol baru muncul saat kursor di atas widget.

// `data-tauri-drag-region` hanya bekerja di elemen yang diberi atribut, bukan anak-anaknya. 
// Oleh karena itu CSS mematikan pointer-events pada anak-anak area drag (kecuali tombol)
// sehingga klik selalu mengenai elemen drag.
export function DesktopWidgetFrame({
  title,
  children,
  interactive = false,
}: DesktopWidgetFrameProps) {
  const [floating, setFloating] = useState(false);

  const toggleLayer = () => {
    const next = !floating;
    setFloating(next);

    invoke("set_widget_layer", { floating: next }).catch((error) => {
      console.error(error);
      setFloating(!next);
    });
  };

  const close = () => {
    invoke("close_widget").catch(console.error);
  };

  return (
    <div
      className="desktop-widget"
      data-interactive={interactive}
      data-tauri-drag-region={interactive ? undefined : true}
    >
      <div className="desktop-widget-header" data-tauri-drag-region>
        <span className="desktop-widget-title">{title}</span>

        <div className="desktop-widget-controls">
          <button
            className="desktop-widget-button"
            onClick={toggleLayer}
            aria-pressed={floating}
            title={floating ? "Send back to desktop" : "Float above other windows"}
          >
            <Layers size={13} />
          </button>

          <button
            className="desktop-widget-button"
            onClick={close}
            title="Close widget"
            aria-label="Close widget"
          >
            <X size={13} />
          </button>
        </div>
      </div>

      {children}
    </div>
  );
}