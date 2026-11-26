import { useEffect, useState } from "react";

import { useMusicStore } from "../stores/musicStore";

/**
 * Posisi lagu sekarang dalam ms. 
 * Rust hanya mengirim posisi saat keadaan berubah, jadi di antara event posisi dihitung di sini dari waktu yang berlalu. 
 * Timer hanya berjalan saat lagu diputar dan komponen tampil.
 */
export function usePlaybackPosition(): number {
    const player = useMusicStore((s) => s.player);
    const receivedAt = useMusicStore((s) => s.playerAt);
    const [now, setNow] = useState(() => Date.now());

    const playing = player.status === "playing";

    useEffect(() => {
        if (!playing) return;

        setNow(Date.now());
        const timer = window.setInterval(() => setNow(Date.now()), 250);

        return () => window.clearInterval(timer);
    }, [playing, receivedAt]);

    if (!playing) return player.positionMs;

    const elapsed = Math.max(0, now - receivedAt);
    const duration = player.track?.durationMs ?? 0;
    const position = player.positionMs + elapsed;

    return duration > 0 ? Math.min(position, duration) : position;
}