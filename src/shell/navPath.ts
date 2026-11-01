/**
 * The navigation path: the stack of places from the root of the content
 * pane to the open screen. The breadcrumb shows it, each part opens its
 * level, and ⌘[ pops it. Pure, so it is tested without a DOM.
 */
import type { Moodboard } from "../areas/moodboard/types";

/** One place the content pane can show. */
export type Place =
  /** A tab of the current Thesis. The root of a Thesis path is its "overview" tab. */
  | { kind: "thesis"; tab: string }
  /** Every Thesis. `create` focuses the "Add Thesis" form. */
  | { kind: "theses"; create?: boolean }
  /** A Shared area, or Settings. */
  | { kind: "area"; id: string }
  | { kind: "style"; id: number }
  /** An Essay; `focusPassage` selects that Passage when the Essay opens. */
  | { kind: "essay"; id: number; focusPassage?: string }
  /** A Passage, opened over its Essay or Script. */
  | { kind: "passage"; id: string }
  | { kind: "moodboard"; board: Moodboard }
  /** The Script of a Documentary. */
  | { kind: "script"; documentaryId: number };

/** One level of the path: a place and its breadcrumb label. */
export interface Step {
  place: Place;
  label: string;
}

/** The path from the root to the open place. It is never empty. */
export type NavPath = readonly [Step, ...Step[]];

/** The open place: the top of the path. */
export const current = (path: NavPath): Step => path[path.length - 1];

/** Opens `step` on top of the path. */
export const push = (path: NavPath, step: Step): NavPath => [...path, step];

/** Goes back one level. At the root it returns the same path. */
export const pop = (path: NavPath): NavPath => popTo(path, path.length - 2);

/** Goes back to level `depth` (0 is the root). A level at or above the top returns the same path. */
export function popTo(path: NavPath, depth: number): NavPath {
  if (depth < 0 || depth >= path.length - 1) return path;
  return path.slice(0, depth + 1) as unknown as NavPath;
}

/** Changes the label of level `depth`, for example after an Essay's title changes. */
export function relabel(path: NavPath, depth: number, label: string): NavPath {
  if (path[depth] === undefined || path[depth].label === label) return path;
  return path.map((s, i) => (i === depth ? { ...s, label } : s)) as unknown as NavPath;
}

/** The breadcrumb labels, from the root to the open place. */
export const crumbs = (path: NavPath): string[] => path.map((s) => s.label);

/**
 * The level of `path` that owns the screen, and the Passage open over it.
 * A Passage opens over its Essay or Script, which stays mounted so the
 * editor keeps its undo history.
 */
export function screenLevel(path: NavPath): { depth: number; passageId?: string } {
  const top = path.length - 1;
  const place = path[top].place;
  const below = path[top - 1]?.place.kind;
  if (place.kind === "passage" && (below === "essay" || below === "script")) {
    return { depth: top - 1, passageId: place.id };
  }
  return { depth: top };
}

/** A key that changes when the screen at a level shows another place. */
export function placeKey(place: Place): string {
  switch (place.kind) {
    case "moodboard":
      return `moodboard:${place.board.id}`;
    case "thesis":
      return `thesis:${place.tab}`;
    case "theses":
      return "theses";
    case "area":
      return `area:${place.id}`;
    case "essay":
      return `essay:${place.id}:${place.focusPassage ?? ""}`;
    case "style":
      return `style:${place.id}`;
    case "passage":
      return `passage:${place.id}`;
    case "script":
      return `script:${place.documentaryId}`;
  }
}

const titled = (title: string) => title.trim() || "Untitled";

/** The steps that open an item, each with its breadcrumb label. */
export const steps = {
  /** A Style, labelled "Name (Color)". */
  style: (s: { id: number; name: string; color: string }): Step => ({
    place: { kind: "style", id: s.id },
    label: `${s.name} (${s.color})`,
  }),
  essay: (id: number, title: string, focusPassage?: string): Step => ({
    place: focusPassage === undefined ? { kind: "essay", id } : { kind: "essay", id, focusPassage },
    label: titled(title),
  }),
  passage: (id: string): Step => ({ place: { kind: "passage", id }, label: "Passage" }),
  moodboard: (board: Moodboard): Step => ({ place: { kind: "moodboard", board }, label: titled(board.title) }),
  script: (documentaryId: number, title: string): Step => ({
    place: { kind: "script", documentaryId },
    label: titled(title),
  }),
};
