import { describe, expect, it } from "vitest";
import { crumbs, current, pop, popTo, push, relabel, screenLevel, steps, type NavPath } from "./navPath";

const thesisRoot: NavPath = [{ place: { kind: "thesis", tab: "overview" }, label: "Impermanence" }];
const coat = steps.style({ id: 4, name: "Hooded Coat", color: "Ash" });

describe("the navigation path", () => {
  it("push adds a place on top, and the breadcrumb shows every label from the root", () => {
    const path = push(push(thesisRoot, { place: { kind: "thesis", tab: "styles" }, label: "Styles" }), coat);
    expect(crumbs(path)).toEqual(["Impermanence", "Styles", "Hooded Coat (Ash)"]);
    expect(current(path).place).toEqual({ kind: "style", id: 4 });
  });

  it("pop goes back one level", () => {
    const path = pop(push(thesisRoot, coat));
    expect(crumbs(path)).toEqual(["Impermanence"]);
  });

  it("pop at the root does nothing", () => {
    expect(pop(thesisRoot)).toBe(thesisRoot);
  });

  it("popTo goes back to one level of the breadcrumb", () => {
    const path = push(push(push(thesisRoot, { place: { kind: "thesis", tab: "styles" }, label: "Styles" }), coat), steps.passage("p1"));
    expect(crumbs(popTo(path, 1))).toEqual(["Impermanence", "Styles"]);
    expect(crumbs(popTo(path, 0))).toEqual(["Impermanence"]);
    expect(popTo(path, 3)).toBe(path);
  });

  it("relabel changes the label of one level, for example after an Essay's title changes", () => {
    const path = push(thesisRoot, steps.essay(9, ""));
    expect(crumbs(path)).toEqual(["Impermanence", "Untitled"]);
    expect(crumbs(relabel(path, 1, "On Ash"))).toEqual(["Impermanence", "On Ash"]);
  });
});

describe("the screen level", () => {
  it("is the open place", () => {
    expect(screenLevel(push(thesisRoot, coat))).toEqual({ depth: 1 });
  });

  it("is the Essay or Script under an open Passage, so the editor stays mounted", () => {
    expect(screenLevel(push(push(thesisRoot, steps.essay(9, "On Ash")), steps.passage("p1")))).toEqual({ depth: 1, passageId: "p1" });
    expect(screenLevel(push(push(thesisRoot, steps.script(3, "Ash film")), steps.passage("p1")))).toEqual({
      depth: 1,
      passageId: "p1",
    });
  });

  it("is the Passage itself when no document is under it", () => {
    expect(screenLevel(push(thesisRoot, steps.passage("p1")))).toEqual({ depth: 1 });
  });
});

describe("step labels", () => {
  it("name each kind of place", () => {
    expect(coat.label).toBe("Hooded Coat (Ash)");
    expect(steps.essay(9, "On Ash").label).toBe("On Ash");
    expect(steps.essay(9, "  ").label).toBe("Untitled");
    expect(steps.passage("p1").label).toBe("Passage");
    expect(steps.moodboard({ id: 2, title: "Moodboard", owner: { kind: "style", id: 4 } }).label).toBe("Moodboard");
    expect(steps.script(3, "Ash film").label).toBe("Ash film");
  });

  it("keep the Passage an Essay opens at", () => {
    expect(steps.essay(9, "On Ash", "p1").place).toEqual({ kind: "essay", id: 9, focusPassage: "p1" });
  });
});
