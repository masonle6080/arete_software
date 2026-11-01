import { describe, expect, it } from "vitest";
import type { Moodboard } from "../areas/moodboard/types";
import { push, steps, crumbs, type NavPath } from "./navPath";
import { blocksBackShortcut, isBackShortcut, openRow, pathOf, shortcutTab, type View } from "./sidebar";

const tabs = ["overview", "essays", "moodboards", "drop", "documentary"];
const key = (k: string, mods: Partial<{ metaKey: boolean; ctrlKey: boolean; altKey: boolean; shiftKey: boolean }> = {}) => ({
  key: k,
  metaKey: false,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  ...mods,
});

describe("⌘1–⌘5", () => {
  it("open the five Thesis tree items in order", () => {
    expect(["1", "2", "3", "4", "5"].map((k) => shortcutTab(key(k, { metaKey: true }), tabs))).toEqual([
      "overview",
      "essays",
      "moodboards",
      "drop",
      "documentary",
    ]);
  });

  it("do nothing without ⌘, with another modifier, or for other digits", () => {
    expect(shortcutTab(key("1"), tabs)).toBeUndefined();
    expect(shortcutTab(key("1", { metaKey: true, shiftKey: true }), tabs)).toBeUndefined();
    expect(shortcutTab(key("1", { metaKey: true, altKey: true }), tabs)).toBeUndefined();
    expect(shortcutTab(key("1", { metaKey: true, ctrlKey: true }), tabs)).toBeUndefined();
    expect(shortcutTab(key("6", { metaKey: true }), tabs)).toBeUndefined();
    expect(shortcutTab(key("0", { metaKey: true }), tabs)).toBeUndefined();
  });
});

const names = { thesis: "Impermanence", area: (id: string) => ({ essays: "Essays", vendors: "Vendors" })[id] ?? id };
const start = (view: View): NavPath => pathOf(view, names);
const coat = steps.style({ id: 4, name: "Hooded Coat", color: "Ash" });
const board = (id: number, owner: Moodboard["owner"]): Moodboard => ({ id, owner, title: "Moodboard" });

describe("the start path of a sidebar row", () => {
  it("starts at the Thesis title inside a Thesis", () => {
    expect(crumbs(start({ kind: "thesis", tab: "overview" }))).toEqual(["Impermanence"]);
    expect(crumbs(start({ kind: "thesis", tab: "drop" }))).toEqual(["Impermanence", "Drop"]);
    expect(crumbs(start({ kind: "thesis", tab: "drop", open: coat }))).toEqual(["Impermanence", "Drop", "Hooded Coat (Ash)"]);
  });

  it("keeps the Thesis root for an Essay opened from the Thesis", () => {
    const path = start({ kind: "thesis", tab: "essays", open: steps.essay(9, "On Ash") });
    expect(crumbs(path)).toEqual(["Impermanence", "Essays", "On Ash"]);
  });

  it("starts at the list under Shared", () => {
    const path = push(start({ kind: "area", id: "essays" }), steps.essay(9, "On Ash"));
    expect(crumbs(path)).toEqual(["Essays", "On Ash"]);
  });

  it("starts at the Thesis list for All Theses", () => {
    expect(crumbs(start({ kind: "theses" }))).toEqual(["All Theses"]);
  });
});

describe("the open row", () => {
  it("is the Thesis tab while nothing is open on top of it", () => {
    expect(openRow(start({ kind: "thesis", tab: "drop" }))).toBe("thesis:drop");
    expect(openRow(start({ kind: "thesis", tab: "overview" }))).toBe("thesis:overview");
  });

  it("is the Style, Essay, or Thesis Moodboard that is open", () => {
    expect(openRow(start({ kind: "thesis", tab: "drop", open: coat }))).toBe("style:4");
    expect(openRow(start({ kind: "thesis", tab: "essays", open: steps.essay(9, "On Ash") }))).toBe("essay:9");
    expect(openRow(start({ kind: "thesis", tab: "moodboards", open: steps.moodboard(board(2, { kind: "thesis", id: 1 })) }))).toBe(
      "moodboard:2",
    );
  });

  it("is the Style for the Moodboard of a Style, because the tree stops at Style", () => {
    const path = push(start({ kind: "thesis", tab: "drop", open: coat }), steps.moodboard(board(2, { kind: "style", id: 4 })));
    expect(openRow(path)).toBe("style:4");
  });

  it("is the nearest row below a Passage or a Script", () => {
    const passage = push(start({ kind: "thesis", tab: "essays", open: steps.essay(9, "On Ash") }), steps.passage("p1"));
    expect(openRow(passage)).toBe("essay:9");
    expect(openRow(push(start({ kind: "thesis", tab: "documentary" }), steps.script(3, "Ash film")))).toBe("thesis:documentary");
  });

  it("is the Shared area or the Thesis list outside the Thesis, whatever is open in it", () => {
    expect(openRow(start({ kind: "area", id: "vendors" }))).toBe("area:vendors");
    expect(openRow(push(start({ kind: "area", id: "essays" }), steps.essay(9, "On Ash")))).toBe("area:essays");
    expect(openRow(start({ kind: "theses" }))).toBe("theses");
  });
});

describe("⌘[", () => {
  it("is ⌘ with [ and no other modifier", () => {
    expect(isBackShortcut(key("[", { metaKey: true }))).toBe(true);
    expect(isBackShortcut(key("["))).toBe(false);
    expect(isBackShortcut(key("[", { metaKey: true, shiftKey: true }))).toBe(false);
    expect(isBackShortcut(key("]", { metaKey: true }))).toBe(false);
  });

  /** A stand-in for the focused element: it matches the selectors in `inside`. */
  const focusIn = (...inside: string[]) => ({
    closest: (selectors: string) => (selectors.split(",").some((s) => inside.includes(s.trim())) ? {} : null),
  });

  it("is left to the Moodboard canvas and to text fields and editors, which use it themselves", () => {
    expect(blocksBackShortcut(focusIn(".excalidraw"))).toBe(true);
    expect(blocksBackShortcut(focusIn("input"))).toBe(true);
    expect(blocksBackShortcut(focusIn("textarea"))).toBe(true);
    expect(blocksBackShortcut(focusIn("select"))).toBe(true);
    expect(blocksBackShortcut(focusIn("[contenteditable]"))).toBe(true);
  });

  it("goes back from anywhere else", () => {
    expect(blocksBackShortcut(focusIn("button"))).toBe(false);
    expect(blocksBackShortcut(null)).toBe(false);
    expect(blocksBackShortcut({})).toBe(false);
  });
});
