import { describe, expect, it } from "vitest";
import type { Moodboard } from "../areas/moodboard/types";
import { openRow, shortcutTab } from "./sidebar";

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

describe("the open row", () => {
  const board = (id: number, owner: Moodboard["owner"]): Moodboard => ({ id, owner, title: "Ash" });

  it("is the Thesis tab while nothing is open on top of it", () => {
    expect(openRow({ kind: "thesis", tab: "drop" })).toBe("thesis:drop");
  });

  it("is the Style, Essay, or Thesis Moodboard that is open", () => {
    expect(openRow({ kind: "thesis", tab: "drop", open: { kind: "style", id: 4 } })).toBe("style:4");
    expect(openRow({ kind: "thesis", tab: "essays", open: { kind: "essay", id: 9 } })).toBe("essay:9");
    expect(
      openRow({ kind: "thesis", tab: "moodboards", open: { kind: "moodboard", board: board(2, { kind: "thesis", id: 1 }) } }),
    ).toBe("moodboard:2");
  });

  it("is the Style for the Moodboard of a Style, because the tree stops at Style", () => {
    expect(
      openRow({ kind: "thesis", tab: "drop", open: { kind: "moodboard", board: board(2, { kind: "style", id: 4 }) } }),
    ).toBe("style:4");
  });

  it("is the Shared area or the Thesis list outside the Thesis", () => {
    expect(openRow({ kind: "area", id: "vendors" })).toBe("area:vendors");
    expect(openRow({ kind: "theses" })).toBe("theses");
  });
});
