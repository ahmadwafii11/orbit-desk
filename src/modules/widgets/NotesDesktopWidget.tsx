import { useEffect, useRef } from "react";
import type { ChangeEvent } from "react";
import { ChevronLeft, ChevronRight, Plus } from "lucide-react";

import { useNotesStore } from "../../stores/notesStore";
import { useNotesSync } from "../../hooks/useNotesSync";

import type { Note } from "../../types";
import { countWords } from "../../lib/notes";
import { SaveState } from "../../stores/notesStore";

import { DesktopWidgetFrame } from "../../components/widget/DesktopWidgetFrame";

const SAVE_LABELS: Record<SaveState, string> = {
  saved: "Saved",
  saving: "Saving...",
  error: "Save failed",
};

/** Dipasang dengan `key={note.id}` agar fokus dan state di-reset tiap ganti catatan. */
function NoteTextarea({ note }: { note: Note }) {
  const updateContent = useNotesStore((s) => s.updateContent);
  const ref = useRef<HTMLTextAreaElement>(null);

  // Catatan baru langsung siap diketik.
  useEffect(() => {
    if (note.content === "") ref.current?.focus();
  }, []);

  return (
    <textarea
      ref={ref}
      value={note.content}
      onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
        updateContent(note.id, event.target.value)
      }
      placeholder="Write something..."
      spellCheck={false}
    />
  );
}

/** Widget catatan: satu catatan sekali tampil, pindah dengan panah, tambah dengan "+". */
export function NotesDesktopWidget() {
  useNotesSync();

  const notes = useNotesStore((s) => s.notes);
  const selectedId = useNotesStore((s) => s.selectedId);
  const loadState = useNotesStore((s) => s.loadState);
  const saveState = useNotesStore((s) => s.saveState);
  const error = useNotesStore((s) => s.error);
  const select = useNotesStore((s) => s.select);
  const create = useNotesStore((s) => s.create);

  // Simpan ketikan yang masih menunggu saat widget kehilangan fokus,
  // disembunyikan, atau ditutup.
  useEffect(() => {
    const flushNow = () => {
      void useNotesStore.getState().flush();
    };

    window.addEventListener("blur", flushNow);
    window.addEventListener("pagehide", flushNow);

    return () => {
      window.removeEventListener("blur", flushNow);
      window.removeEventListener("pagehide", flushNow);
      flushNow();
    };
  }, []);

  const index = notes.findIndex((n) => n.id === selectedId);
  const note = index >= 0 ? notes[index] : undefined;

  const go = (delta: number) => {
    if (notes.length === 0) return;

    const next = index < 0 ? 0 : (index + delta + notes.length) % notes.length;
    select(notes[next].id);
  };

  return (
    <DesktopWidgetFrame title="NOTES" interactive>
      <div className="desktop-notes-toolbar">
        <button
          className="desktop-notes-button"
          onClick={() => go(-1)}
          disabled={notes.length < 2}
          title="Previous note"
          aria-label="Previous note"
        >
          <ChevronLeft size={14} />
        </button>

        <span className="desktop-notes-counter">
          {note ? `${index + 1} / ${notes.length}` : "No notes"}
        </span>

        <button
          className="desktop-notes-button"
          onClick={() => go(1)}
          disabled={notes.length < 2}
          title="Next note"
          aria-label="Next note"
        >
          <ChevronRight size={14} />
        </button>

        <button
          className="desktop-notes-button"
          onClick={() => void create()}
          title="New note"
          aria-label="New note"
        >
          <Plus size={14} />
        </button>
      </div>

      {note ? (
        <NoteTextarea key={note.id} note={note} />
      ) : (
        <div className="desktop-notes-empty">
          <p>
            {loadState === "error"
              ? `Could not load notes: ${error}`
              : loadState === "ready"
                ? "No notes yet"
                : "Loading..."}
          </p>

          {loadState === "ready" && (
            <button className="desktop-notes-create" onClick={() => void create()}>
              Create note
            </button>
          )}
        </div>
      )}

      <div className="desktop-notes-footer">
        <span>{note ? `${countWords(note.content)} words` : ""}</span>
        <span
          className="save-state"
          data-state={saveState}
          title={saveState === "error" && error ? error : undefined}
        >
          {SAVE_LABELS[saveState]}
        </span>
      </div>
    </DesktopWidgetFrame>
  );
}