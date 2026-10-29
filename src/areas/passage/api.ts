import { call } from "../../shell/invoke";
import type { DocumentKind } from "../essays/api";

/** Mirrors `Passage` in `src-tauri/src/document/passage.rs`. */
export interface Passage {
  id: string;
  documentId: number;
  documentKind: DocumentKind;
  documentTitle: string;
  /** The text at the last save that had this Passage. */
  lastKnownText: string;
  /** True when the last save did not have this Passage. */
  orphaned: boolean;
  createdAt: string;
  updatedAt: string;
}

/**
 * IPC wrappers for the Passage commands in `src-tauri/src/ipc/document.rs`.
 * Saves report Passages through `documentApi.save`.
 */
export const passageApi = {
  get: (id: string) => call<Passage>("get_passage", { id }),
  ofDocument: (documentId: number) => call<Passage[]>("passages_of_document", { documentId }),
};
