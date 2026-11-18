import { useEffect } from "react";

import { startNotesSync, useNotesStore } from "../stores/notesStore";

// Pasang di komponen manapun yang akan menampikan note. 
// Memuat data saat pertama kali, mengambil ulang jika data sudah ada tetapi mungkin sudah usang (saat tidak terdapat komponen yang mendengarkan, perubahan dari window lain terlewat)
// Lalu mendengarkan perubahan dari window lain.
export function useNotesSync() {
    useEffect(() => {
        const state = useNotesStore.getState();
        void (state.loadState === "ready" ? state.reload() : state.load());

        return startNotesSync();
    }, []);
}