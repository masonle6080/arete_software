import { call } from "../shell/invoke";

/*
 * Exports: files and PDFs the founder takes out of the app.
 *
 * - `exportPdf(job)`: opens a print window that renders `job.html` and opens
 *   the system print dialog. The founder picks "Save as PDF" there. Essays,
 *   Tech Packs, and Style traces all use it. (macOS can only print from a
 *   top-level window, so this is a window and not an iframe.)
 * - `saveTextFile(...)`: a save dialog, then a file write outside the
 *   library. Returns the saved path, or null when the founder cancels.
 *
 * IPC: `src-tauri/src/ipc/export.rs`.
 */

/** What a PDF export shows. Mirrors `PrintJob` in `src-tauri/src/export/mod.rs`. */
export interface PrintJob {
  /** The window title and the default PDF file name. */
  title: string;
  /** An HTML fragment (no `<html>` wrapper). Escape any user text in it. */
  html: string;
  /** CSS on top of the base print style, for example table rules. */
  css?: string | null;
}

export function exportPdf(job: PrintJob): Promise<void> {
  return call<void>("print_export", { job: { css: null, ...job } });
}

export interface TextFile {
  /** The suggested file name, for example "Patience.md". */
  defaultName: string;
  contents: string;
  /** The file type in the dialog, for example "Markdown". */
  filterName: string;
  /** File extensions without the dot, for example ["md"]. */
  extensions: string[];
}

export function saveTextFile(file: TextFile): Promise<string | null> {
  return call<string | null>("save_export_file", { ...file });
}

/** A file name from a title: no characters that macOS or Finder refuse. */
export function fileNameFor(title: string, extension: string): string {
  const base = title.replace(/[/:\\]/g, "-").replace(/^\.+/, "").trim() || "Untitled";
  return `${base}.${extension}`;
}

/** The print job ID in a print window's URL hash (`#print=ID`), or null. */
export function printJobId(hash: string): string | null {
  const m = /^#print=([A-Za-z0-9_-]+)$/.exec(hash);
  return m ? m[1] : null;
}

export const printApi = {
  takeJob: (id: string) => call<PrintJob | null>("take_print_job", { id }),
};
