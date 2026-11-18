import { useEffect, useRef, useState } from "react";
import type { ChangeEvent } from "react";
import { Pin, PinOff } from "lucide-react";

import { useNotesStore } from "../../stores/notesStore";
import type { SaveState } from "../../stores/notesStore"

import { formatRelativeTime } from "../../lib/time";
import { countWords } from "../../lib/notes";;

import type { Note } from "../../types";

const SAVE_LABELS: Record<SaveState, string> = {
  saved: "Saved",
  saving: "Saving...",
  error: "Save failed. Will retry on next edit",
};

/** Pasang dengan `key={note.id}` agar state lokal di-reset tiap ganti catatan. */
export function NoteEditor({ note }: { note: Note }) {
  const updateContent = useNotesStore((s) => s.updateContent);
  const togglePin = useNotesStore((s) => s.togglePin);
  const remove = useNotesStore((s) => s.remove);
  const saveState = useNotesStore((s) => s.saveState);
  const error = useNotesStore((s) => s.error);

  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Catatan baru langsung siap diketik.
  useEffect(() => {
    if (note.content === "") textareaRef.current?.focus();
  }, []);

  // Konfirmasi hapus kedaluwarsa sendiri setelah 3 detik.
  useEffect(() => {
    if (!confirmingDelete) return;

    const timer = window.setTimeout(() => setConfirmingDelete(false), 3000);
    return () => window.clearTimeout(timer);
  }, [confirmingDelete]);

  const handleDelete = () => {
    if (!confirmingDelete) {
      setConfirmingDelete(true);
      return;
    }

    void remove(note.id);
  };

  return (
    <div className="widget notes-editor">
      <div className="notes-editor-toolbar">
        <span>Edited {formatRelativeTime(note.updatedAt)}</span>

        <div className="notes-editor-actions">
          <button
            className="icon-button notes-pin"
            onClick={() => void togglePin(note.id)}
            aria-pressed={note.pinned}
            title={note.pinned ? "Unpin note" : "Pin note"}
          >
            {note.pinned ? <PinOff size={15} /> : <Pin size={15} />}
          </button>

          <button
            className="notes-delete"
            onClick={handleDelete}
            data-confirm={confirmingDelete}
          >
            {confirmingDelete ? "Delete?" : "Delete"}
          </button>
        </div>
      </div>

      <textarea
        ref={textareaRef}
        value={note.content}
        onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
          updateContent(note.id, event.target.value)
        }
        placeholder="Start writing something..."
        spellCheck={false}
      />

      <div className="notes-editor-footer">
        <span>
          {countWords(note.content)} words · {note.content.length} characters
        </span>
        <span
          className="save-state"
          data-state={saveState}
          title={saveState === "error" && error ? error : undefined}
        >
          {SAVE_LABELS[saveState]}
        </span>
      </div>
    </div>
  );
}
