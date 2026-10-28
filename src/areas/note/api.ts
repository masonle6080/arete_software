import { call } from "../../shell/invoke";

/** The record a note belongs to. Mirrors `NoteOwner` in `src-tauri/src/note/mod.rs`. */
export type NoteOwner =
  | { kind: "vendor"; id: number }
  | { kind: "sample"; id: number }
  | { kind: "productionOrder"; id: number };

export interface Attachment {
  media: string;
  /** The name the file had when it was attached. */
  fileName: string;
  /** Absolute path of the stored copy in the library. */
  path: string;
}

export interface Note {
  id: number;
  owner: NoteOwner;
  body: string;
  /** UTC, `YYYY-MM-DDTHH:MM:SS.sssZ`. */
  createdAt: string;
  attachments: Attachment[];
}

export interface NoteInput {
  body: string;
  /** Paths of files outside the library. The core imports a copy of each. */
  attachments: string[];
}

/** IPC wrappers for `src-tauri/src/ipc/note.rs`. */
export const noteApi = {
  listNotes: (owner: NoteOwner) => call<Note[]>("list_notes", { owner }),
  addNote: (owner: NoteOwner, input: NoteInput) => call<Note>("add_note", { owner, input }),
};
