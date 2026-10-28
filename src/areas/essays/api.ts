import { call } from "../../shell/invoke";
import type { Thesis } from "../thesis/types";

/** Mirrors `DocumentKind` in `src-tauri/src/document/mod.rs`. */
export type DocumentKind = "essay" | "script";

export interface DocumentSummary {
  id: number;
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
  open: (id: number) => call<StoredDocument>("open_document", { id }),
  save: (id: number, edit: DocumentEdit) => call<DocumentSummary>("save_document", { id, edit }),
  list: (kind: DocumentKind) => call<DocumentSummary[]>("list_documents", { kind }),
};

/** Essay ↔ Thesis links (many-to-many). An Essay with no Thesis is valid. */
export const essayThesisApi = {
  link: (essayId: number, thesisId: number) => call<void>("link_essay_to_thesis", { essayId, thesisId }),
  unlink: (essayId: number, thesisId: number) => call<void>("unlink_essay_from_thesis", { essayId, thesisId }),
  thesesOfEssay: (essayId: number) => call<Thesis[]>("theses_of_essay", { essayId }),
  essaysOfThesis: (thesisId: number) => call<DocumentSummary[]>("essays_of_thesis", { thesisId }),
};
