import { Music2, Pause, Play, SkipBack, SkipForward } from "lucide-react";

import { useMusicStore } from "../../stores/musicStore";
import { useMusicSync } from "../../hooks/useMusicSync";
import { artistOf } from "../../lib/music";

// Isi dari kartu "Music Player" di Dashboard: lagu sekarang dan tombol kendali.
export function MusicPreview() {
  useMusicSync();

  const player = useMusicStore((s) => s.player);
  const hasLibrary = useMusicStore((s) => s.library.tracks.length > 0);
  const togglePause = useMusicStore((s) => s.togglePause);
  const next = useMusicStore((s) => s.next);
  const previous = useMusicStore((s) => s.previous);

  const { track } = player;
  const playing = player.status === "playing";

  return (
    <>
      <div className="music-preview">
        <div className="album-placeholder">
          <Music2 size={25} />
        </div>

        <div className="track-info">
          <strong>{track ? track.title : "No track selected"}</strong>
          <span>
            {track
              ? artistOf(track)
              : hasLibrary
                ? "Pick a song in Music"
                : "Add your music folder"}
          </span>
        </div>
      </div>

      <div className="music-controls">
        <button
          className="control-button"
          onClick={() => void previous()}
          disabled={!track}
          aria-label="Previous"
        >
          <SkipBack size={16} />
        </button>

        <button
          className="play-button"
          onClick={() => void togglePause()}
          disabled={!track}
          aria-label={playing ? "Pause" : "Play"}
        >
          {playing ? <Pause size={15} /> : <Play size={15} />}
        </button>

        <button
          className="control-button"
          onClick={() => void next()}
          disabled={!track}
          aria-label="Next"
        >
          <SkipForward size={16} />
        </button>
      </div>
    </>
  );
}
