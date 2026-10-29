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

/** Mirrors `LinkedMoodboardItem` in `src-tauri/src/moodboard/passage_link.rs`. */
export interface LinkedMoodboardItem {
  moodboard: Moodboard;
  /** The Excalidraw element ID. */
  elementId: string;
  /** The Excalidraw element type: "image", "text", "rectangle", ... */
  elementType: string;
  text: string | null;
  /** The media name of an image element. */
  media: string | null;
  /** The fill color of a swatch. */
  color: string | null;
}
