import { call } from "../../shell/invoke";

/** Mirrors `DecorationArtwork` in `src-tauri/src/decoration/mod.rs`. */
export interface DecorationArtwork {
  media: string;
  /** The name the file had when it was imported. */
  fileName: string;
  /** Absolute path of the stored copy in the library. */
  path: string;
}

/** Mirrors `Decoration`: colors are in the order listed. */
export interface Decoration {
  id: number;
  styleId: number;
  technique: string;
  placement: string;
  size: string;
  colors: string[];
  artwork: DecorationArtwork | null;
  vendorId: number | null;
  vendorName: string | null;
}

/** Mirrors `DecorationInput`: an edit replaces every field but the artwork. */
export interface DecorationInput {
  technique: string;
  placement: string;
  size: string;
  colors: string[];
  vendorId: number | null;
}

/** IPC wrappers for `src-tauri/src/ipc/decoration.rs`. */
export const decorationApi = {
  /** The core refuses an empty technique. */
  add: (styleId: number, input: DecorationInput) => call<Decoration>("add_decoration", { styleId, input }),
  update: (id: number, input: DecorationInput) => call<Decoration>("update_decoration", { id, input }),
  /** `source` is a path outside the library; the core imports a copy. */
  setArtwork: (id: number, source: string) => call<Decoration>("set_decoration_artwork", { id, source }),
  clearArtwork: (id: number) => call<Decoration>("clear_decoration_artwork", { id }),
  /** A Design Decision linked to it stays, with no Decoration. */
  remove: (id: number) => call<void>("remove_decoration", { id }),
  ofStyle: (styleId: number) => call<Decoration[]>("style_decorations", { styleId }),
};

/** A short name for a Decoration, for example "Screen print · Center chest". */
export function decorationLabel(d: Decoration): string {
  return d.placement ? `${d.technique} · ${d.placement}` : d.technique;
}
