import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

import type { Library, PlayerSnapshot, RepeatMode } from "../types";

const STATE_EVENT = "music-state-changed";
const LIBRARY_EVENT = "music-library-changed";

// Jeda setelah slider volume berhenti digeser sebelum dikirim ke Rust.
const VOLUME_DELAY_MS = 150;

export type LoadState = "idle" | "loading" | "ready" | "error";

const IDLE_PLAYER: PlayerSnapshot = {
    status: "stopped",
    track: null,
    positionMs: 0,
    volume: 0.8,
    shuffle: false,
    repeat: "off",
    error: null,
};

interface MusicState {
    library: Library;
    loadState: LoadState;
    player: PlayerSnapshot;
    playerAt: number; // Waktu (Date.now) `player` diterima; dasar menghitung posisi lagu.
    scanning: boolean;
    error: string | null;

    load: () => Promise<void>;
    loadPlayer: () => Promise<void>; // Hanya keadaan pemutar, tanpa daftar lagu. Untuk widget yang ringan.
    reloadLibrary: () => Promise<void>;
    addFolder: () => Promise<void>;
    removeFolder: (path: string) => Promise<void>;
    rescan: () => Promise<void>;
    play: (id: string, queueIds: string[]) => Promise<void>; // Putar `id`; `queueIds` = urutan lagu yang sedang tampil di layar.
    togglePause: () => Promise<void>;
    next: () => Promise<void>;
    previous: () => Promise<void>;
    seek: (positionMs: number) => Promise<void>;
    setVolume: (volume: number) => void;
    toggleShuffle: () => Promise<void>;
    cycleRepeat: () => Promise<void>;
    dismissError: () => void;
}

const NEXT_REPEAT: Record<RepeatMode, RepeatMode> = {
    off: "all",
    all: "one",
    one: "off",
};

function errorText(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
}

// Selama slider volume sedang digeser (atau menunggu dikirim), volume dari snapshot yang masuk diabaikan. 
// Tanpa ini, snapshot lama dapat menarik slider mundur di tengah gesekan.
let volumeTimer: number | null = null;
let volumeInFlight = false;

function applySnapshot(snapshot: PlayerSnapshot) {
    const keepLocalVolume = volumeTimer !== null || volumeInFlight;

    useMusicStore.setState((state) => ({
        player: keepLocalVolume
            ? { ...snapshot, volume: state.player.volume }
            : snapshot,
        playerAt: Date.now(),
    }));
}

export const useMusicStore = create<MusicState>((set, get) => {
    // Menjalankan perintah pemutar yang mengembalikan snapshot baru.
    const command = async (name: string, args?: Record<string, unknown>) => {
        try {
            applySnapshot(await invoke<PlayerSnapshot>(name, args));
        } catch (error) {
            set({ error: errorText(error) });
        }
    };

    return {
        library: { folders: [], tracks: [] },
        loadState: "idle",
        player: IDLE_PLAYER,
        playerAt: 0,
        scanning: false,
        error: null,

        load: async () => {
            const { loadState } = get();
            if (loadState === "loading" || loadState === "ready") return;

            set({ loadState: "loading" });

            try {
                const [library, player] = await Promise.all([
                    invoke<Library>("music_library"),
                    invoke<PlayerSnapshot>("music_state"),
                ]);

                set({ library, loadState: "ready", error: null });
                applySnapshot(player);
            } catch (error) {
                set({ loadState: "error", error: errorText(error) });
            }
        },

        loadPlayer: async () => {
            try {
                applySnapshot(await invoke<PlayerSnapshot>("music_state"));
            } catch (error) {
                set({ error: errorText(error) });
            }
        },

        reloadLibrary: async () => {
            try {
                const [library, player] = await Promise.all([
                    invoke<Library>("music_library"),
                    invoke<PlayerSnapshot>("music_state"),
                ]);

                set({ library, loadState: "ready" });
                applySnapshot(player);
            } catch (error) {
                set({ error: errorText(error) });
            }
        },

        addFolder: async () => {
            try {
                const chosen = await open({ directory: true, multiple: false });
                const path = Array.isArray(chosen) ? chosen[0] : chosen;
                if (!path) return;

                set({ scanning: true, error: null });
                const library = await invoke<Library>("music_add_folder", { path });
                set({ library });
            } catch (error) {
                set({ error: errorText(error) });
            } finally {
                set({ scanning: false });
            }
        },

        removeFolder: async (path) => {
            try {
                set({ library: await invoke<Library>("music_remove_folder", { path }) });
            } catch (error) {
                set({ error: errorText(error) });
            }
        },

        rescan: async () => {
            set({ scanning: true, error: null });

            try {
                set({ library: await invoke<Library>("music_rescan") });
            } catch (error) {
                set({ error: errorText(error) });
            } finally {
                set({ scanning: false });
            }
        },

        play: (id, queueIds) => command("music_play", { id, queue: queueIds }),
        togglePause: () => command("music_toggle_pause"),
        next: () => command("music_next"),
        previous: () => command("music_previous"),

        seek: async (positionMs) => {
            // Menggeser tampilan dulu agar slider tidak melompat balik menunggu Rust.
            set((state) => ({
                player: { ...state.player, positionMs },
                playerAt: Date.now(),
            }));

            await command("music_seek", { positionMs: Math.round(positionMs) });
        },

        setVolume: (volume) => {
            const clamped = Math.min(1, Math.max(0, volume));

            set((state) => ({ player: { ...state.player, volume: clamped } }));

            if (volumeTimer !== null) window.clearTimeout(volumeTimer);
            volumeTimer = window.setTimeout(() => {
                volumeTimer = null;
                volumeInFlight = true;

                void invoke<PlayerSnapshot>("music_set_volume", {
                    volume: get().player.volume,
                })
                    .catch((error) => set({ error: errorText(error) }))
                    .finally(() => {
                        volumeInFlight = false;
                    });
            }, VOLUME_DELAY_MS);
        },

        toggleShuffle: () =>
            command("music_set_shuffle", { shuffle: !get().player.shuffle }),

        cycleRepeat: () =>
            command("music_set_repeat", { repeat: NEXT_REPEAT[get().player.repeat] }),

        dismissError: () =>
            set((state) => ({ error: null, player: { ...state.player, error: null } })),
    };
});

// ---------------------------------------------------------------------------
// Sinkronisasi antar window
// ---------------------------------------------------------------------------

let syncUsers = 0;
let unlistenPromise: Promise<() => void> | null = null;

/**
 * Dengarkan perubahan pemutar dan perpustakaan dari jendela mana pun.
 * Aman dipanggil dari beberapa komponen: listener dipasang sekali dan dilepas saat pemakai terakhir berhenti. 
 * Mengembalikan fungsi berhenti.
 */
export function startMusicSync(): () => void {
    syncUsers += 1;

    if (syncUsers === 1) {
        const unlistenState = listen<PlayerSnapshot>(STATE_EVENT, (event) => {
            applySnapshot(event.payload);
        });

        const unlistenLibrary = listen(LIBRARY_EVENT, () => {
            void useMusicStore.getState().reloadLibrary();
        });

        unlistenPromise = Promise.all([unlistenState, unlistenLibrary])
            .then(([a, b]) => () => {
                a();
                b();
            })
            .catch(() => () => { });
    }

    return () => {
        syncUsers -= 1;

        if (syncUsers === 0 && unlistenPromise) {
            void unlistenPromise.then((unlisten) => unlisten());
            unlistenPromise = null;
        }
    };
}