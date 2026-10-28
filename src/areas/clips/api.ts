import { call } from "../../shell/invoke";

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
}

/** IPC wrappers for `src-tauri/src/ipc/clip.rs`. */
export const clipApi = {
  listClips: () => call<Clip[]>("list_clips"),
  importClip: (path: string) => call<Clip>("import_clip", { path }),
  setDateShot: (id: number, dateShot: string | null) => call<Clip>("set_clip_date_shot", { id, dateShot }),
  revealClip: (id: number) => call<void>("reveal_clip", { id }),
};
