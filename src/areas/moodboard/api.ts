import { call } from "../../shell/invoke";
import { importExtension } from "./mediaType";
import type { LinkedMoodboardItem, Moodboard, MoodboardOwner } from "./types";

/** IPC wrappers for `src-tauri/src/ipc/moodboard.rs` and `ipc/media.rs`. */
export const moodboardApi = {
  createMoodboard: (owner: MoodboardOwner, title: string) =>
    call<Moodboard>("create_moodboard", { owner, title }),
  listMoodboards: (owner: MoodboardOwner) => call<Moodboard[]>("list_moodboards", { owner }),
  /** The saved scene JSON, or null for a Moodboard never saved. */
  loadScene: (id: number) => call<string | null>("load_moodboard_scene", { id }),
  /**
   * Saves the scene. The scene owns Passage links: an element links to a
   * Passage through `customData.passageId`, and each save updates the core's
   * copy of those links.
   */
  saveScene: (id: number, scene: string) => call<void>("save_moodboard_scene", { id, scene }),
  /** The canvas items linked to a Passage, from every Moodboard. */
  itemsOfPassage: (passageId: string) =>
    call<LinkedMoodboardItem[]>("moodboard_items_of_passage", { passageId }),
};

/** IPC wrappers for media import and read (`src-tauri/src/ipc/media.rs`). */
export const mediaApi = {
  /** Imports a dropped file into the library (with dedup). Returns its media name. */
  importFile: async (file: File) =>
    call<string>("import_media_bytes", new Uint8Array(await file.arrayBuffer()), {
      headers: { "media-ext": importExtension(file) },
    }),
  /** The bytes of a stored media file. */
  read: (media: string) => call<ArrayBuffer>("read_media", { media }),
};
