import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

// Menebak tipe gambar dari byte awal; 
// Webview membutuhkan tipe yang benar untuk Blob
function sniffMime(bytes: Uint8Array): string {
    if (bytes[0] === 0x89 && bytes[1] === 0x50) return "image/png";
    if (bytes[0] === 0xff && bytes[1] === 0xd8) return "image/jpeg";
    if (bytes[0] === 0x47 && bytes[1] === 0x49) return "image/gif";
    if (bytes[0] === 0x52 && bytes[1] === 0x49) return "image/webp";
    return "image/jpeg";
}

/**
 * Cover lagu `trackId` sebagai URL gambar, atau null jika lagu itu tidak punya cover. 
 * URL dilepas otomatis saat lagu berganti.
 */
export function useCover(trackId: string | null): string | null {
    const [url, setUrl] = useState<string | null>(null);

    useEffect(() => {
        if (!trackId) {
            setUrl(null);
            return;
        }

        let cancelled = false;
        let created: string | null = null;

        invoke<ArrayBuffer | number[]>("music_cover", { id: trackId })
            .then((raw) => {
                if (cancelled) return;

                const bytes = raw instanceof ArrayBuffer ? new Uint8Array(raw) : new Uint8Array(raw);
                if (bytes.length === 0) {
                    setUrl(null);
                    return;
                }

                created = URL.createObjectURL(new Blob([bytes], { type: sniffMime(bytes) }));
                setUrl(created);
            })
            .catch(() => {
                if (!cancelled) setUrl(null);
            });

        return () => {
            cancelled = true;
            if (created) URL.revokeObjectURL(created);
        };
    }, [trackId]);

    return url;
}