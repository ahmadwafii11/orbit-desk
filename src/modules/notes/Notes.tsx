import { useEffect, useMemo, useState } from "react";
import { NotebookPen, Plus } from "lucide-react";

import { useNotesStore } from "../../stores/notesStore";
import { useNotesSync } from "../../hooks/useNotesSync";

import { matchesQuery } from "../../lib/notes";

import { NoteEditor } from "./NoteEditor";
import { NotesList } from "./NotesList";

import { PinWidgetButton } from "../../components/widget/PinWidgetButton";
import { SectionHeading } from "../../components/layout/SectionHeading";

export function Notes() {
  useNotesSync();

  const notes = useNotesStore((s) => s.notes);
  const selectedId = useNotesStore((s) => s.selectedId);
  const loadState = useNotesStore((s) => s.loadState);
  const error = useNotesStore((s) => s.error);
  const create = useNotesStore((s) => s.create);
  const select = useNotesStore((s) => s.select);

  const [query, setQuery] = useState("");

  // Simpan ketikan yang masih menunggu saat pindah modul atau jendela ditutup.
  useEffect(() => {
    const flushNow = () => {
      void useNotesStore.getState().flush();
    };

    window.addEventListener("pagehide", flushNow);

    return () => {
      window.removeEventListener("pagehide", flushNow);
      flushNow();
    };
  }, []);

  const visibleNotes = useMemo(
    () => (query.trim() ? notes.filter((n) => matchesQuery(n, query)) : notes),
    [notes, query],
  );

  const selected = notes.find((n) => n.id === selectedId);

  return (
    <>
      <SectionHeading
        eyebrow="PERSONAL WORKSPACE"
        title="Quick Notes."
        subtitle="Write down anything on your mind."
        action={
          <div className="heading-actions">
            <PinWidgetButton moduleId="notes" />

            <button className="notes-new-button" onClick={() => void create()}>
              <Plus size={15} /> New note
            </button>
          </div>
        }
      />

      {loadState === "error" && (
        <div className="widget notes-error">
          Could not load notes: {error}
        </div>
      )}

      <div className="notes-layout">
        <NotesList
          notes={visibleNotes}
          selectedId={selectedId}
          query={query}
          onQueryChange={setQuery}
          onSelect={select}
        />

        {selected ? (
          <NoteEditor key={selected.id} note={selected} />
        ) : (
          <div className="widget notes-empty">
            <NotebookPen size={34} strokeWidth={1.2} />
            <h3>{loadState === "loading" ? "Loading notes..." : "No note selected"}</h3>
            <p>Select a note from the list, or create a new one.</p>
          </div>
        )}
      </div>
    </>
  );
}