import { call } from "../../shell/invoke";
import type { Clip } from "../clips/api";
import type { StoredDocument } from "../essays/api";
import type { Documentary, DocumentaryStatus } from "./types";

/** IPC wrappers for `src-tauri/src/ipc/documentary.rs`. */
export const documentaryApi = {
  createDocumentary: (thesisId: number, title: string) =>
    call<Documentary>("create_documentary", { thesisId, title }),
  moveDocumentary: (id: number, to: DocumentaryStatus) => call<Documentary>("move_documentary", { id, to }),
  thesisDocumentary: (thesisId: number) => call<Documentary | null>("thesis_documentary", { thesisId }),
  listDocumentaries: () => call<Documentary[]>("list_documentaries"),
  addClip: (id: number, clipId: number) => call<void>("add_clip_to_documentary", { id, clipId }),
  removeClip: (id: number, clipId: number) => call<void>("remove_clip_from_documentary", { id, clipId }),
  documentaryClips: (id: number) => call<Clip[]>("documentary_clips", { id }),
  clipDocumentaries: (clipId: number) => call<Documentary[]>("clip_documentaries", { clipId }),
  /** Opens the Documentary's one Script (created on first open). Save it with `documentApi.save`. */
  script: (id: number) => call<StoredDocument>("documentary_script", { id }),
};
