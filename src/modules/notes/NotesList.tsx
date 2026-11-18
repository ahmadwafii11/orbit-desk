import type { ChangeEvent } from "react";
import { Pin, Search } from "lucide-react";

import { formatRelativeTime } from "../../lib/time";
import type { Note } from "../../types";
import { describeNote } from "../../lib/notes";

interface NotesListProps {
  notes: Note[];
  selectedId: string | null;
  query: string;
  onQueryChange: (value: string) => void;
  onSelect: (id: string) => void;
}

export function NotesList({
  notes,
  selectedId,
  query,
  onQueryChange,
  onSelect,
}: NotesListProps) {
  return (
    <div className="widget notes-list">
      <label className="notes-search">
        <Search size={13} />
        <input
          value={query}
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            onQueryChange(event.target.value)
          }
          placeholder="Search notes"
          aria-label="Search notes"
        />
      </label>

      {notes.length === 0 ? (
        <p className="notes-list-empty">
          {query ? "No matching notes" : "No notes yet"}
        </p>
      ) : (
        <ul className="notes-items">
          {notes.map((note) => {
            const { title, preview } = describeNote(note.content);
            const isActive = note.id === selectedId;

            return (
              <li key={note.id}>
                <button
                  className={`note-item ${isActive ? "active" : ""}`}
                  onClick={() => onSelect(note.id)}
                  aria-current={isActive ? "true" : undefined}
                >
                  <span className="note-item-title">
                    {note.pinned && <Pin size={10} />}
                    <span className="note-item-title-text">{title}</span>
                  </span>
                  <span className="note-item-preview">{preview}</span>
                  <span className="note-item-time">
                    {formatRelativeTime(note.updatedAt)}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
