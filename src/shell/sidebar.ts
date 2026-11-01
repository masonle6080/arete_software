/**
 * The sidebar's navigation model: what the content pane shows, which sidebar
 * row that marks as open, and the ⌘1–⌘5 shortcuts. Pure, so it is tested
 * without a DOM.
 */
import type { Moodboard } from "../areas/moodboard/types";

/** What a Thesis tab opens on top of itself: a Style, an Essay, or a Moodboard. */
export type ThesisItem = { kind: "style"; id: number } | { kind: "essay"; id: number } | { kind: "moodboard"; board: Moodboard };

/**
 * What the content pane shows: a tab of the current Thesis (with the item
 * open on top of it, if any), every Thesis (`create` focuses the "Add
 * Thesis" form), or one Shared area or Settings.
 */
export type View =
  | { kind: "thesis"; tab: string; open?: ThesisItem }
  | { kind: "theses"; create?: boolean }
  | { kind: "area"; id: string };

/**
 * The key of the sidebar row that `view` marks as open. Tree rows use the
 * same keys: `thesis:<tab>`, `style:<id>`, `essay:<id>`, `moodboard:<id>`,
 * `area:<id>`. The Moodboard of a Style marks its Style, because the tree
 * stops at Style.
 */
export function openRow(view: View): string {
  switch (view.kind) {
    case "theses":
      return "theses";
    case "area":
      return `area:${view.id}`;
    case "thesis": {
      const item = view.open;
      if (!item) return `thesis:${view.tab}`;
      if (item.kind !== "moodboard") return `${item.kind}:${item.id}`;
      const { owner } = item.board;
      return owner.kind === "style" ? `style:${owner.id}` : `moodboard:${item.board.id}`;
    }
  }
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
