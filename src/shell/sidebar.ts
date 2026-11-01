/**
 * The sidebar's navigation model: the path each sidebar row starts, which
 * row a path marks as open, and the ⌘1–⌘5 and ⌘[ shortcuts. Pure, so it is
 * tested without a DOM.
 */
import { placeKey, popTo, push, thesisTabLabel, type NavPath, type Step } from "./navPath";

export { thesisTabLabel };

/**
 * What a sidebar row or shortcut opens: a tab of the current Thesis (with
 * an item open on top of it, if any), every Thesis (`create` focuses the
 * "Add Thesis" form), or one Shared area or Settings.
 */
export type View =
  | { kind: "thesis"; tab: string; open?: Step }
  | { kind: "theses"; create?: boolean }
  | { kind: "area"; id: string };

/** The names a start path needs: the current Thesis title and the label of each area. */
export interface PathNames {
  thesis: string;
  area(id: string): string;
}

/**
 * The navigation path that `view` starts. Inside a Thesis the root is the
 * Thesis title (its Overview), then the tab, then the open item. Under
 * Shared the root is the area's list.
 */
export function pathOf(view: View, names: PathNames): NavPath {
  switch (view.kind) {
    case "theses":
      return [{ place: { kind: "theses", create: view.create }, label: "All Theses" }];
    case "area":
      return [{ place: { kind: "area", id: view.id }, label: names.area(view.id) }];
    case "thesis": {
      let path: NavPath = [{ place: { kind: "thesis", tab: "overview" }, label: names.thesis }];
      if (view.tab !== "overview") {
        path = push(path, { place: { kind: "thesis", tab: view.tab }, label: thesisTabLabel[view.tab] ?? view.tab });
      }
      return view.open ? push(path, view.open) : path;
    }
  }
}

/**
 * What opening `step` does, for example from a chip: the new path, and the
 * Thesis to make current when it is not `currentThesis`.
 *
 * - A Thesis tab opens from the Thesis root, as its tree row does. A tab of
 *   another Thesis makes that Thesis current first.
 * - A place already on the path goes back to that level, so the path never
 *   holds a place twice (a Style's Moodboard links back to the Style).
 * - Any other place opens one level down.
 */
export function openStep(
  path: NavPath,
  step: Step,
  currentThesis: number | undefined,
  names: PathNames,
): { path: NavPath; thesisId?: number } {
  const { place } = step;
  if (place.kind === "thesis") {
    const target = place.thesisId ?? currentThesis;
    const next = pathOf({ kind: "thesis", tab: place.tab }, names);
    return target === currentThesis ? { path: next } : { path: next, thesisId: target };
  }
  const key = placeKey(place);
  const depth = path.findIndex((s) => placeKey(s.place) === key);
  return { path: depth < 0 ? push(path, step) : popTo(path, depth) };
}

/** The tree row of one step, if it has one. */
function rowOf({ place }: Step): string | undefined {
  switch (place.kind) {
    case "thesis":
      return `thesis:${place.tab}`;
    case "style":
    case "essay":
      return `${place.kind}:${place.id}`;
    case "moodboard": {
      const { owner } = place.board;
      return owner.kind === "style" ? `style:${owner.id}` : `moodboard:${place.board.id}`;
    }
    default:
      return undefined;
  }
}

/**
 * The key of the sidebar row that `path` marks as open. Tree rows use the
 * same keys: `thesis:<tab>`, `style:<id>`, `essay:<id>`, `moodboard:<id>`.
 * Inside a Thesis it is the row of the deepest level that has one: the
 * Moodboard of a Style marks its Style, because the tree stops at Style.
 * Outside a Thesis it is the root: `area:<id>` or `theses`.
 */
export function openRow(path: NavPath): string {
  const root = path[0].place;
  if (root.kind === "theses") return "theses";
  if (root.kind === "area") return `area:${root.id}`;
  for (let i = path.length - 1; i >= 0; i--) {
    const row = rowOf(path[i]);
    if (row) return row;
  }
  return "thesis:overview";
}

/** The keys of a keydown event that a shortcut reads. */
export interface ShortcutKey {
  key: string;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

/**
 * The Thesis tab that ⌘1–⌘5 opens: ⌘1 the first of `tabs`, up to ⌘5 the
 * fifth. Undefined for any other key or modifier.
 */
export function shortcutTab(e: ShortcutKey, tabs: readonly string[]): string | undefined {
  if (!e.metaKey || e.ctrlKey || e.altKey || e.shiftKey) return undefined;
  const n = "12345".indexOf(e.key);
  return n < 0 ? undefined : tabs[n];
}

/** Whether a keydown is ⌘[, which goes back one breadcrumb level. */
export function isBackShortcut(e: ShortcutKey): boolean {
  return e.metaKey && !e.ctrlKey && !e.altKey && !e.shiftKey && e.key === "[";
}

/**
 * Where ⌘[ belongs to the focused element, not to the shell: the Moodboard
 * canvas (Excalidraw uses ⌘[ for "send backward"), text fields, and editors.
 */
const ownsBackShortcut = ".excalidraw, input, textarea, select, [contenteditable]";

/** Whether ⌘[ must not go back while focus is on `target` (a keydown target). */
export function blocksBackShortcut(target: { closest?(selectors: string): unknown } | null): boolean {
  return typeof target?.closest === "function" && target.closest(ownsBackShortcut) !== null;
}
