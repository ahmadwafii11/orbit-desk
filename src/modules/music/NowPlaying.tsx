import { Music2, Volume2, VolumeX } from "lucide-react";
import type { ChangeEvent } from "react";

import { useMusicStore } from "../../stores/musicStore";
import { PlayerControls } from "./PlayerControls";
import { SeekBar } from "./SeekBar";
import { useCover } from "../../hooks/useCover";
import { artistOf } from "../../lib/music";

export function NowPlaying() {
  const track = useMusicStore((s) => s.player.track);
  const volume = useMusicStore((s) => s.player.volume);
  const playerError = useMusicStore((s) => s.player.error);
  const setVolume = useMusicStore((s) => s.setVolume);

  const cover = useCover(track?.id ?? null);

  return (
    <>
      <div className="widget music-detail">
        <div className="large-album">
          {cover ? (
            <img src={cover} alt="" draggable={false} />
          ) : (
            <Music2 size={45} strokeWidth={1.2} />
          )}
        </div>

        <p className="widget-label">NOW PLAYING</p>
        <h2>{track ? track.title : "No track selected"}</h2>
        <p className="subtitle">
          {track
            ? `${artistOf(track)}${track.album ? ` · ${track.album}` : ""}`
            : "Pick a song from your library below."}
        </p>

        {playerError && <p className="music-error">{playerError}</p>}

        <SeekBar />
        <PlayerControls />

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
      </div>
    </>
  );
}
