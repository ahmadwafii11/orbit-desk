import { create } from "zustand";

import type { ModuleId } from "../types";

interface AppState {
    activeModule: ModuleId;
    setActiveModule: (id: ModuleId) => void;

    isPlaying: boolean;
    togglePlaying: () => void;

    note: string;
    setNote: (value: string) => void;
}

export const useAppStore = create<AppState>((set) => ({
    activeModule: "dashboard",
    setActiveModule: (id) => set({ activeModule: id }),

    isPlaying: false,
    togglePlaying: () => set((s) => ({ isPlaying: !s.isPlaying })),

    note: "",
    setNote: (value) => set({ note: value}),
}));