import { call } from "../../shell/invoke";

/** Mirrors `DocumentKind` in `src-tauri/src/document/mod.rs`. */
export type DocumentKind = "essay" | "script";

export interface DocumentSummary {
  id: string;
  kind: DocumentKind;
  title: string;
  createdAt: string;
  updatedAt: string;
}

export interface StoredDocument extends DocumentSummary {
  markdown: string;
}

export interface DocumentEdit {
  title: string;
  markdown: string;
}

/**
 * IPC wrappers for `src-tauri/src/ipc/document.rs`. Storage is the same for
 * every kind, so Scripts can use this too.
 */
export const documentApi = {
  create: (kind: DocumentKind, title: string) =>
    call<StoredDocument>("create_document", { kind, title }),
  open: (id: string) => call<StoredDocument>("open_document", { id }),
  save: (id: string, edit: DocumentEdit) => call<DocumentSummary>("save_document", { id, edit }),
  list: (kind: DocumentKind) => call<DocumentSummary[]>("list_documents", { kind }),
};
