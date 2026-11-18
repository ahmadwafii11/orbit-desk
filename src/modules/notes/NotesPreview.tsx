import { useNotesStore } from "../../stores/notesStore";
import { isBlank, summarizeNote } from "../../lib/notes";
import { useNotesSync } from "../../hooks/useNotesSync";

/**
 * Isi kartu "Quick Notes" di Dashboard: ringkasan catatan teratas
 * (yang disematkan atau terbaru) dan jumlah catatan.
 */
export function NotesPreview() {
  useNotesSync();

  const notes = useNotesStore((s) => s.notes);

  const filled = notes.filter((note) => !isBlank(note));
  const top = filled[0];

  const description =
    filled.length > 0
      ? `${filled.length} ${filled.length === 1 ? "note" : "notes"} · Open notes`
      : "Open notes";

  return (
    <>
      <p className="notes-preview">
        {top ? summarizeNote(top.content) : "Capture your thoughts..."}
      </p>

      <span className="widget-description">{description}</span>
    </>
  );
}
