import { Volume2, VolumeX } from "lucide-react";
import type { ChangeEvent } from "react";

import { useMusicStore } from "../../stores/musicStore";

// Slider volume. 
// Perubahan dikirim ke Rust setelah slider berhenti digeser.
export function VolumeControl() {
    const volume = useMusicStore((s) => s.player.volume);
    const setVolume = useMusicStore((s) => s.setVolume);

    return (
        <>
            <label className="volume-control">
                {volume === 0 ? <VolumeX size={15} /> : <Volume2 size={15} />}

                <input
                    type="range"
                    min={0}
                    max={100}
                    value={Math.round(volume * 100)}
                    onChange={(e: ChangeEvent<HTMLInputElement>) =>
                        setVolume(Number(e.target.value) / 100)
                    }
                    aria-label="Volume"
                    style={{ ["--fill" as string]: `${Math.round(volume * 100)}%` }}
                />
            </label>
        </>
    );
}