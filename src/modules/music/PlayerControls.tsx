import {
  Pause,
  Play,
  Repeat,
  Repeat1,
  Shuffle,
  SkipBack,
  SkipForward,
} from "lucide-react";

import { useMusicStore } from "../../stores/musicStore";

// Tombol shuffle, previous, play/pause, next, dan repeat.
export function PlayerControls() {
  const player = useMusicStore((s) => s.player);
  const togglePause = useMusicStore((s) => s.togglePause);
  const next = useMusicStore((s) => s.next);
  const previous = useMusicStore((s) => s.previous);
  const toggleShuffle = useMusicStore((s) => s.toggleShuffle);
  const cycleRepeat = useMusicStore((s) => s.cycleRepeat);

  const hasTrack = player.track !== null;
  const playing = player.status === "playing";

  return (
    <>
      <div className="player-controls">
        <button
          className="control-button"
          data-active={player.shuffle}
          onClick={() => void toggleShuffle()}
          title={player.shuffle ? "Shuffle on" : "Shuffle off"}
          aria-pressed={player.shuffle}
        >
          <Shuffle size={16} />
        </button>

        <button
          className="control-button"
          onClick={() => void previous()}
          disabled={!hasTrack}
          title="Previous"
          aria-label="Previous"
        >
          <SkipBack size={20} />
        </button>

        <button
          className="play-button large"
          onClick={() => void togglePause()}
          disabled={!hasTrack}
          title={playing ? "Pause" : "Play"}
          aria-label={playing ? "Pause" : "Play"}
        >
          {playing ? <Pause size={20} /> : <Play size={20} />}
        </button>

        <button
          className="control-button"
          onClick={() => void next()}
          disabled={!hasTrack}
          title="Next"
          aria-label="Next"
        >
          <SkipForward size={20} />
        </button>

        <button
          className="control-button"
          data-active={player.repeat !== "off"}
          onClick={() => void cycleRepeat()}
          title={`Repeat: ${player.repeat}`}
          aria-label={`Repeat: ${player.repeat}`}
        >
          {player.repeat === "one" ? <Repeat1 size={16} /> : <Repeat size={16} />}
        </button>
      </div>
    </>
  );
}
