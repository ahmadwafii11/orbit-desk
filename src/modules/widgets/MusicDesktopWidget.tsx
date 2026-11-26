import { Music2 } from "lucide-react";

import { useMusicStore } from "../../stores/musicStore";

import { useCover } from "../../hooks/useCover";
import { useMusicSync } from "../../hooks/useMusicSync";
import { artistOf } from "../../lib/music";

import { PlayerControls } from "../music/PlayerControls";
import { SeekBar } from "../music/SeekBar";
import { VolumeControl } from "../music/VolumeControl";

import { DesktopWidgetFrame } from "../../components/widget/DesktopWidgetFrame";

/**
 * Widget pemutar musik. 
 * Pemutarnya sendiri ada di Rust dan dipakai bersama semua window, jadi widget ini hanya menampilkan keadaan dan mengirim perintah. 
 * Daftar lagu sengaja tidak dimuat di sini.
 */
export function MusicDesktopWidget() {
  useMusicSync({ library: false });

  const track = useMusicStore((s) => s.player.track);
  const playerError = useMusicStore((s) => s.player.error);
  const error = useMusicStore((s) => s.error);

  const cover = useCover(track?.id ?? null);
  const message = error ?? playerError;

  return (
    <DesktopWidgetFrame title="MUSIC" interactive>
      <div className="desktop-music-track">
        <div className="desktop-music-cover">
          {cover ? (
            <img src={cover} alt="" draggable={false} />
          ) : (
            <Music2 size={20} strokeWidth={1.4} />
          )}
        </div>

        <div className="desktop-music-text">
          <strong>{track ? track.title : "Nothing playing"}</strong>
          <span>{track ? artistOf(track) : "Press play to start"}</span>
        </div>
      </div>

      {message && <p className="desktop-music-error">{message}</p>}

      <SeekBar />
      <PlayerControls />
      <VolumeControl />
    </DesktopWidgetFrame>
  );
}