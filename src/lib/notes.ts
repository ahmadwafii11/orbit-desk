import type { Note } from "../types";

function nonEmptyLines(content: string): string[] {
  return content
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line !== "");
}

// Title = baris pertama yang tidak kosong. 
// Preview = baris kedua.
export function describeNote(content: string): {
  title: string;
  preview: string;
} {
  const lines = nonEmptyLines(content);

  return {
    title: (lines[0] ?? "New note").slice(0, 80),
    preview: (lines[1] ?? "No additional text").slice(0, 80),
  };
}

// Summary dua baris pertama untuk kartu di Dashboard.
export function summarizeNote(content: string): string {
  return nonEmptyLines(content).slice(0, 2).join(" · ").slice(0, 90);
}

export function countWords(content: string): number {
  const trimmed = content.trim();
  return trimmed ? trimmed.split(/\s+/).length : 0;
}

export function isBlank(note: Pick<Note, "content">): boolean {
  return note.content.trim() === "";
}

export function matchesQuery(note: Note, query: string): boolean {
  return note.content.toLowerCase().includes(query.trim().toLowerCase());
}