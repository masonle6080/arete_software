import { describe, expect, it } from "vitest";
import { styleSections } from "./sections";
import { baseKindOf, sectionsIn, styleSegments } from "./segments";

const ids = (sections: { id: string }[]) => sections.map((s) => s.id);

describe("Style segments", () => {
  it("Design holds Design Decisions, Decorations, Moodboard, and Files", () => {
    expect(ids(sectionsIn(styleSections, "design", "blank"))).toEqual([
      "design-decisions",
      "decorations",
      "moodboard",
      "files",
    ]);
  });

  it("Make holds Samples and Cost Sheet", () => {
    expect(ids(sectionsIn(styleSections, "make", "techPack"))).toEqual(["samples", "cost-sheet"]);
  });

  it("Build shows only Base for a Blank Style", () => {
    const kind = baseKindOf({ kind: "blank", blankId: 7 });
    expect(ids(sectionsIn(styleSections, "build", kind))).toEqual(["base"]);
  });

  it("Build shows only Tech Pack for a Tech Pack Style", () => {
    const kind = baseKindOf({ kind: "techPack" });
    expect(ids(sectionsIn(styleSections, "build", kind))).toEqual(["tech-pack"]);
  });

  it("Build shows only Tech Pack for a Style with no base yet", () => {
    expect(ids(sectionsIn(styleSections, "build", baseKindOf(null)))).toEqual(["tech-pack"]);
  });

  it("every registered section shows in some segment", () => {
    const shown = styleSegments.flatMap(({ value }) => [
      ...sectionsIn(styleSections, value, "blank"),
      ...sectionsIn(styleSections, value, "techPack"),
    ]);
    expect(new Set(ids(shown))).toEqual(new Set(ids(styleSections)));
  });
});
