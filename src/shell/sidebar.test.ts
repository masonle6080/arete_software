import { describe, expect, it } from "vitest";
import type { Moodboard } from "../areas/moodboard/types";
import { push, steps, crumbs, thesisTabLabel, thesisTabs, type NavPath, type Step } from "./navPath";
import { blocksBackShortcut, isBackShortcut, openRow, openStep, pathOf, shortcutTab, type View } from "./sidebar";

const key = (k: string, mods: Partial<{ metaKey: boolean; ctrlKey: boolean; altKey: boolean; shiftKey: boolean }> = {}) => ({
  key: k,
  metaKey: false,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  ...mods,
});

describe("the Thesis tabs", () => {
  it("run Overview, Essays, Moodboards, Styles, Documentary, Production in the sidebar and for ⌘1–⌘6", () => {
    expect(thesisTabs).toEqual(["overview", "essays", "moodboards", "styles", "documentary", "production"]);
    expect(thesisTabs.map((t) => thesisTabLabel[t])).toEqual([
      "Overview",
      "Essays",
      "Moodboards",
      "Styles",
      "Documentary",
      "Production",
    ]);
  });

  it("label the breadcrumb level of each tab", () => {
    expect(crumbs(pathOf({ kind: "thesis", tab: "styles" }, names))).toEqual(["Impermanence", "Styles"]);
    expect(crumbs(pathOf({ kind: "thesis", tab: "production" }, names))).toEqual(["Impermanence", "Production"]);
  });
});

describe("⌘1–⌘6", () => {
  it("open the six Thesis tabs in order", () => {
    expect(["1", "2", "3", "4", "5", "6"].map((k) => shortcutTab(key(k, { metaKey: true })))).toEqual([
      "overview",
      "essays",
      "moodboards",
      "styles",
      "documentary",
      "production",
    ]);
  });

  it("do nothing without ⌘, with another modifier, or for other digits", () => {
    expect(shortcutTab(key("1"))).toBeUndefined();
    expect(shortcutTab(key("1", { metaKey: true, shiftKey: true }))).toBeUndefined();
    expect(shortcutTab(key("1", { metaKey: true, altKey: true }))).toBeUndefined();
    expect(shortcutTab(key("1", { metaKey: true, ctrlKey: true }))).toBeUndefined();
    expect(shortcutTab(key("7", { metaKey: true }))).toBeUndefined();
    expect(shortcutTab(key("0", { metaKey: true }))).toBeUndefined();
  });
});

const names = { thesis: "Impermanence", area: (id: string) => ({ essays: "Essays", vendors: "Vendors" })[id] ?? id };
const start = (view: View): NavPath => pathOf(view, names);
const coat = steps.style({ id: 4, name: "Hooded Coat", color: "Ash" });
const board = (id: number, owner: Moodboard["owner"]): Moodboard => ({ id, owner, title: "Moodboard" });

describe("opening a step, as a chip does", () => {
  const inStyles = push(start({ kind: "thesis", tab: "styles" }), coat);

  it("pushes a place that is not on the path", () => {
    const opened = openStep(inStyles, steps.essay(9, "On Ash"), 1, names);
    expect(crumbs(opened.path)).toEqual(["Impermanence", "Styles", "Hooded Coat (Ash)", "On Ash"]);
    expect(opened.thesisId).toBeUndefined();
  });

  it("goes back to a place that is already on the path, so the path never repeats it", () => {
    const onBoard = push(inStyles, steps.moodboard(board(2, { kind: "style", id: 4 })));
    expect(crumbs(openStep(onBoard, coat, 1, names).path)).toEqual(["Impermanence", "Styles", "Hooded Coat (Ash)"]);
  });

  it("opens a tab of the current Thesis from its root, wherever the path is", () => {
    expect(crumbs(openStep(inStyles, steps.thesis("essays"), 1, names).path)).toEqual(["Impermanence", "Essays"]);
    expect(crumbs(openStep(inStyles, steps.thesis("overview", 1), 1, names).path)).toEqual(["Impermanence"]);
    expect(openStep(inStyles, steps.thesis("essays", 1), 1, names).thesisId).toBeUndefined();
  });

  it("makes another Thesis current to open its tab", () => {
    const fromShared = start({ kind: "area", id: "essays" });
    const opened = openStep(push(fromShared, steps.essay(9, "On Ash")), steps.thesis("overview", 7), 1, names);
    expect(opened.thesisId).toBe(7);
    expect(opened.path[0].place).toEqual({ kind: "thesis", tab: "overview" });
    expect(opened.path).toHaveLength(1);
  });

  it("makes a Thesis current when none is", () => {
    const opened = openStep(start({ kind: "theses" }), steps.thesis("production", 7), undefined, names);
    expect(opened.thesisId).toBe(7);
    expect(crumbs(opened.path)[1]).toBe("Production");
  });
});

describe("the start path of a sidebar row", () => {
  it("starts at the Thesis title inside a Thesis", () => {
    expect(crumbs(start({ kind: "thesis", tab: "overview" }))).toEqual(["Impermanence"]);
    expect(crumbs(start({ kind: "thesis", tab: "styles" }))).toEqual(["Impermanence", "Styles"]);
    expect(crumbs(start({ kind: "thesis", tab: "styles", open: coat }))).toEqual(["Impermanence", "Styles", "Hooded Coat (Ash)"]);
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
    for (const tab of thesisTabs) expect(openRow(start({ kind: "thesis", tab }))).toBe(`thesis:${tab}`);
  });

  it("is the owning tab of the open Style, Essay, or Thesis Moodboard, because the sidebar has no item rows", () => {
    expect(openRow(start({ kind: "thesis", tab: "styles", open: coat }))).toBe("thesis:styles");
    expect(openRow(start({ kind: "thesis", tab: "essays", open: steps.essay(9, "On Ash") }))).toBe("thesis:essays");
    expect(openRow(start({ kind: "thesis", tab: "moodboards", open: steps.moodboard(board(2, { kind: "thesis", id: 1 })) }))).toBe(
      "thesis:moodboards",
    );
  });

  it("is the owning tab even when the item opened from another tab, for example the Overview", () => {
    const fromOverview = (open: Step) => openRow(start({ kind: "thesis", tab: "overview", open }));
    expect(fromOverview(coat)).toBe("thesis:styles");
    expect(fromOverview(steps.essay(9, "On Ash"))).toBe("thesis:essays");
    expect(fromOverview(steps.moodboard(board(2, { kind: "thesis", id: 1 })))).toBe("thesis:moodboards");
  });

  it("is Styles for the Moodboard of a Style", () => {
    const path = push(start({ kind: "thesis", tab: "styles", open: coat }), steps.moodboard(board(2, { kind: "style", id: 4 })));
    expect(openRow(path)).toBe("thesis:styles");
    expect(openRow(start({ kind: "thesis", tab: "overview", open: steps.moodboard(board(2, { kind: "style", id: 4 })) }))).toBe(
      "thesis:styles",
    );
  });

  it("is the tab below a Passage or a Script", () => {
    const passage = push(start({ kind: "thesis", tab: "essays", open: steps.essay(9, "On Ash") }), steps.passage("p1"));
    expect(openRow(passage)).toBe("thesis:essays");
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
