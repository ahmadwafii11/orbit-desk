import { useEffect } from "react";

import { startStorageSync, useStorageStore } from "../stores/storageStore";

/**
 * Pasang di komponen yang menampilkan storage: 
 * - memuat data, 
 * - mendengarkan perubahan dari window lain 
 * - serta merefresh status "file hilang" setiap kali window kembali difokuskan (file bisa dipindah di luar app).
 */
export function useStorageSync() {
  useEffect(() => {
    const state = useStorageStore.getState();
    void (state.loadState === "ready" ? state.reload() : state.load());

    const stopSync = startStorageSync();

    const onFocus = () => {
      void useStorageStore.getState().reload();
    };
    window.addEventListener("focus", onFocus);

    return () => {
      window.removeEventListener("focus", onFocus);
      stopSync();
    };
  }, []);
}