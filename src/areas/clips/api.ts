import { call } from "../../shell/invoke";
import type { Passage } from "../passage/api";

/** One Clip in the pool. Mirrors `Clip` in `src-tauri/src/clip/mod.rs`. */
export interface Clip {
  id: number;
  media: string;
  originalName: string;
  durationMs: number | null;
  /** `YYYY-MM-DD`, or null when unknown. */
  dateShot: string | null;
  path: string;
  thumbnailPath: string | null;
  /** Alphabetical, case ignored. */
  tags: string[];
  /** Free-text notes; empty when none. */
  notes: string;
}

/** Mirrors `ClipFilter` in `src-tauri/src/clip/links.rs`. Every set field must match. */
export interface ClipFilter {
  tag: string | null;
  passageId: string | null;
}

/** IPC wrappers for `src-tauri/src/ipc/clip.rs`. */
export const clipApi = {
  listClips: () => call<Clip[]>("list_clips"),
  importClip: (path: string) => call<Clip>("import_clip", { path }),
  setDateShot: (id: number, dateShot: string | null) => call<Clip>("set_clip_date_shot", { id, dateShot }),
  revealClip: (id: number) => call<void>("reveal_clip", { id }),
  addTag: (id: number, tag: string) => call<Clip>("add_clip_tag", { id, tag }),
  removeTag: (id: number, tag: string) => call<Clip>("remove_clip_tag", { id, tag }),
  setNotes: (id: number, notes: string) => call<Clip>("set_clip_notes", { id, notes }),
  allTags: () => call<string[]>("all_clip_tags"),
  filterClips: (filter: ClipFilter) => call<Clip[]>("filter_clips", { filter }),
  linkPassage: (id: number, passageId: string) => call<void>("link_clip_to_passage", { id, passageId }),
  unlinkPassage: (id: number, passageId: string) => call<void>("unlink_clip_from_passage", { id, passageId }),
  clipPassages: (id: number) => call<Passage[]>("clip_passages", { id }),
  passageClips: (passageId: string) => call<Clip[]>("passage_clips", { passageId }),
};
