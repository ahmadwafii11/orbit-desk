import { Music2 } from "lucide-react";

import { useMusicStore } from "../../stores/musicStore";
import type { Track } from "../../types";
import { artistOf, formatClock } from "../../lib/music";

interface TrackListProps {
  tracks: Track[]; // Lagu yang tampil (setelah pencarian, dan sudah dipotong jika banyak).
  queueIds: string[]; // Seluruh urutan hasil pencarian: antrean saat sebuah lagu diklik.
}

export function TrackList({ tracks, queueIds }: TrackListProps) {
  const currentId = useMusicStore((s) => s.player.track?.id ?? null);
  const status = useMusicStore((s) => s.player.status);
  const play = useMusicStore((s) => s.play);

  return (
    <>
      <ul className="track-list">
        {tracks.map((track) => {
          const current = track.id === currentId;

          return (
            <li key={track.id}>
              <button
                className="track-row"
                data-current={current}
                onClick={() => void play(track.id, queueIds)}
                title={track.path}
              >
                <span className="track-row-icon">
                  <Music2 size={14} />
                </span>

                <span className="track-row-text">
                  <strong>{track.title}</strong>
                  <span>
                    {artistOf(track)}
                    {track.album ? ` · ${track.album}` : ""}
                  </span>
                </span>

                <span className="track-row-time">
                  {current && status === "playing"
                    ? "Playing"
                    : formatClock(track.durationMs)}
                </span>
              </button>
            </li>
          );
        })}
      </ul>
    </>
  );
}
