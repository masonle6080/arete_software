/**
 * Mirrors `MoodboardOwner` in `src-tauri/src/moodboard/mod.rs`: the record a
 * Moodboard belongs to. Add a kind here when the core adds one. A Style has
 * at most one Moodboard.
 */
export type MoodboardOwner = { kind: "thesis"; id: number } | { kind: "style"; id: number };

/** Mirrors `Moodboard` in `src-tauri/src/moodboard/mod.rs`. */
export interface Moodboard {
  id: number;
  owner: MoodboardOwner;
  title: string;
}
