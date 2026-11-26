import { useEffect } from "react";

import { startMusicSync, useMusicStore } from "../stores/musicStore";


interface MusicSyncOptions {
  library?: boolean; // Memuat daftar semua lagu. Mematikan untuk tampilan yang hanya butuh keadaan pemutar (widget), agar daftar ribuan lagu tidak ikut menempati memori window tersebut.
}


/**
 * Pasang di komponen mana pun yang menampilkan pemutar. 
 * Memuat data saat pertama kali (atau mengambil ulang jika sudah ada tapi mungkin usang),
 * lalu mendengarkan perubahan dari jendela lain.
 */
export function useMusicSync({ library = true }: MusicSyncOptions = {}) {
  useEffect(() => {
    const state = useMusicStore.getState();

    if (!library) {
      void state.loadPlayer();
    } else {
      void (state.loadState === "ready" ? state.reloadLibrary() : state.load());
    }

    return startMusicSync();
  }, [library]);
}