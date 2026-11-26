import { useState } from "react";
import type { ChangeEvent } from "react";

import { useMusicStore } from "../../stores/musicStore";
import { usePlaybackPosition } from "../../hooks/usePlaybackPosition";
import { formatClock } from "../../lib/music";

// Bilah posisi lagu. 
// Seek hanya dikirim saat slider dilepas, bukan tiap geseran.
export function SeekBar() {
  const duration = useMusicStore((s) => s.player.track?.durationMs ?? 0);
  const seek = useMusicStore((s) => s.seek);
  const position = usePlaybackPosition();

  // Nilai sementara selama slider digeser.
  const [draft, setDraft] = useState<number | null>(null);

  const shown = draft ?? position;

  const commit = () => {
    if (draft === null) return;
    void seek(draft);
    setDraft(null);
  };

  return (
    <>
      <div className="seek-bar">
        <span>{formatClock(shown)}</span>

        <input
          type="range"
          min={0}
          max={Math.max(duration, 1)}
          step={500}
          value={Math.min(shown, Math.max(duration, 1))}
          disabled={duration === 0}
          onChange={(e: ChangeEvent<HTMLInputElement>) =>
            setDraft(Number(e.target.value))
          }
          onPointerUp={commit}
          onKeyUp={commit}
          onBlur={commit}
          aria-label="Seek"
          style={{
            ["--fill" as string]: `${duration > 0 ? (shown / duration) * 100 : 0}%`,
          }}
        />

        <span>{formatClock(duration)}</span>
      </div>
    </>
  );
}
