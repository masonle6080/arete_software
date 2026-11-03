import type { Style } from "./types";

/**
 * The file kinds that can be a Style's Cover: PNG, JPEG, WebP, and HEIC.
 * Mirrors `COVER_EXTENSIONS` in `src-tauri/src/style/mod.rs`. The core
 * enforces the rule; the UI uses this list only to offer "Set as Cover".
 */
const coverExtensions = ["png", "jpg", "jpeg", "webp", "heic"];

/** Whether a design file with this name can be a Cover. */
export function canBeCover(fileName: string): boolean {
  const dot = fileName.lastIndexOf(".");
  return dot > 0 && coverExtensions.includes(fileName.slice(dot + 1).toLowerCase());
}

/** What a Style's card in the Styles grid shows: its Cover, or a neutral placeholder with its color name. */
export type CardFace = { kind: "cover"; path: string } | { kind: "placeholder"; color: string };

/** Picks the face of a Style's card in the Styles grid. */
export function cardFace(style: Pick<Style, "cover" | "color">): CardFace {
  return style.cover ? { kind: "cover", path: style.cover.path } : { kind: "placeholder", color: style.color };
}
