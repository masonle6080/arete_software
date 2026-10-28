import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { noteApi, type Note, type NoteOwner } from "./api";
import "./note.css";

/**
 * The dated notes and attachments of one record (a Vendor, Sample, or
 * Production Order), newest first, with a form to add a note. Not a nav
 * area: owner screens embed it.
 */
export function NotesPanel({ owner }: { owner: NoteOwner }) {
  const [notes, setNotes] = useState<Note[]>([]);
  const [body, setBody] = useState("");
  const [files, setFiles] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<CoreError | undefined>();

  async function reload() {
    try {
      setNotes(await noteApi.listNotes(owner));
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  useEffect(() => {
    setBody("");
    setFiles([]);
    setError(undefined);
    reload();
  }, [owner.kind, owner.id]);

  async function pickFiles() {
    const picked = await open({ multiple: true, title: "Attach files" });
    if (!picked) return;
    const paths = Array.isArray(picked) ? picked : [picked];
    setFiles([...files, ...paths.filter((p) => !files.includes(p))]);
  }

  async function add() {
    setBusy(true);
    setError(undefined);
    try {
      await noteApi.addNote(owner, { body, attachments: files });
      setBody("");
      setFiles([]);
      await reload();
    } catch (raw) {
      setError(toCoreError(raw));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="notes">
      <h2>Notes</h2>
      <form className="note-form" onSubmit={(e) => { e.preventDefault(); add(); }}>
        <textarea rows={3} placeholder="Write a note" value={body} onChange={(e) => setBody(e.target.value)} />
        {files.length > 0 && (
          <ul className="pending-files">
            {files.map((f) => (
              <li key={f}>
                {baseName(f)}
                <button type="button" onClick={() => setFiles(files.filter((x) => x !== f))}>Remove</button>
              </li>
            ))}
          </ul>
        )}
        <div className="actions">
          <button type="button" onClick={pickFiles}>Attach files</button>
          <button type="submit" disabled={busy}>Add note</button>
        </div>
      </form>

      {error && <p className="error">{error.message}</p>}

      {notes.length === 0 ? (
        <p className="muted">No notes yet.</p>
      ) : (
        <ul className="note-list">
          {notes.map((n) => (
            <li key={n.id}>
              <time dateTime={n.createdAt}>{new Date(n.createdAt).toLocaleString()}</time>
              {n.body && <p className="body">{n.body}</p>}
              {n.attachments.length > 0 && (
                <ul className="attachments">
                  {n.attachments.map((a, i) => (
                    <li key={i} title={a.path}>{a.fileName}</li>
                  ))}
                </ul>
              )}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}
