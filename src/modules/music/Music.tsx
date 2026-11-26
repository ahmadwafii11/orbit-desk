import { useMemo, useState } from "react";
import type { ChangeEvent } from "react";
import { FolderPlus, Music2, RefreshCw, Search, X } from "lucide-react";

import { PinWidgetButton } from "../../components/widget/PinWidgetButton";
import { SectionHeading } from "../../components/layout/SectionHeading";

import { useMusicStore } from "../../stores/musicStore";
import { NowPlaying } from "./NowPlaying";
import { TrackList } from "./TrackList";
import { folderName, matchesQuery } from "../../lib/music";
import { useMusicSync } from "../../hooks/useMusicSync";

// Banyak lagu yang digambar sekaligus; sisanya lewat "Show more".
const PAGE_SIZE = 100;

export function Music() {
  useMusicSync();

  const library = useMusicStore((s) => s.library);
  const loadState = useMusicStore((s) => s.loadState);
  const scanning = useMusicStore((s) => s.scanning);
  const error = useMusicStore((s) => s.error);
  const addFolder = useMusicStore((s) => s.addFolder);
  const removeFolder = useMusicStore((s) => s.removeFolder);
  const rescan = useMusicStore((s) => s.rescan);
  const dismissError = useMusicStore((s) => s.dismissError);

  const [query, setQuery] = useState("");
  const [shown, setShown] = useState(PAGE_SIZE);

  const matches = useMemo(
    () => library.tracks.filter((t) => matchesQuery(t, query)),
    [library.tracks, query],
  );
  const queueIds = useMemo(() => matches.map((t) => t.id), [matches]);

  const hasLibrary = library.folders.length > 0;

  return (
    <>
      <SectionHeading
        eyebrow="ENTERTAINMENT"
        title="Music."
        subtitle="Your music, your atmosphere."
        action={
          <div className="heading-actions">
            <PinWidgetButton moduleId="music" />

            {hasLibrary && (
              <button
                className="secondary-button"
                onClick={() => void rescan()}
                disabled={scanning}
                title="Look for new or removed songs"
              >
                <RefreshCw size={14} /> Rescan
              </button>
            )}

            <button
              className="primary-button"
              onClick={() => void addFolder()}
              disabled={scanning}
            >
              <FolderPlus size={15} /> {scanning ? "Scanning..." : "Add folder"}
            </button>
          </div>
        }
      />

      {error && (
        <div className="music-message">
          <span>{error}</span>
          <button onClick={dismissError} aria-label="Dismiss">
            <X size={13} />
          </button>
        </div>
      )}

      <NowPlaying />

      {hasLibrary && (
        <div className="music-folders">
          {library.folders.map((folder) => (
            <span key={folder} className="music-folder-chip" title={folder}>
              {folderName(folder)}
              <button
                onClick={() => void removeFolder(folder)}
                title="Remove folder from library (files are not deleted)"
                aria-label={`Remove ${folderName(folder)}`}
              >
                <X size={11} />
              </button>
            </span>
          ))}
        </div>
      )}

      {library.tracks.length > 0 ? (
        <>
          <div className="music-toolbar">
            <span className="music-count">
              {matches.length} {matches.length === 1 ? "song" : "songs"}
            </span>

            <label className="music-search">
              <Search size={13} />
              <input
                value={query}
                onChange={(e: ChangeEvent<HTMLInputElement>) => {
                  setQuery(e.target.value);
                  setShown(PAGE_SIZE);
                }}
                placeholder="Search title, artist, album"
                spellCheck={false}
              />
            </label>
          </div>

          {matches.length > 0 ? (
            <TrackList tracks={matches.slice(0, shown)} queueIds={queueIds} />
          ) : (
            <div className="empty-state">
              <p>No songs match.</p>
            </div>
          )}

          {matches.length > shown && (
            <button
              className="secondary-button music-more"
              onClick={() => setShown((n) => n + PAGE_SIZE * 2)}
            >
              Show more ({matches.length - shown} left)
            </button>
          )}
        </>
      ) : (
        <div className="empty-state">
          <Music2 size={38} strokeWidth={1.2} />
          <h3>
            {loadState === "loading"
              ? "Loading..."
              : hasLibrary
                ? "No songs found"
                : "Your library is empty"}
          </h3>
          <p>
            {hasLibrary
              ? "Supported formats: MP3, FLAC, OGG, WAV, M4A."
              : "Add a folder with your music. Subfolders are included."}
          </p>
        </div>
      )}
    </>
  );
}
