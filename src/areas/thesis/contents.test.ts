import { describe, expect, it } from "vitest";
import type { Style } from "../style/types";
import { crumbs } from "../../shell/navPath";
import { pathOf, type View } from "../../shell/sidebar";
import { itemView, loadThesisContents, type ContentSources } from "./contents";

const style = (id: number, thesisId: number) => ({ id, thesisId, name: `S${id}`, color: "Bone" }) as Style;

function sources(styles: Style[]): ContentSources {
  return {
    essaysOfThesis: async () => [{ id: 1, kind: "essay", title: "On Ash", createdAt: "", updatedAt: "" }],
    moodboardsOfThesis: async (thesisId) => [{ id: 9, owner: { kind: "thesis", id: thesisId }, title: "Ash" }],
    thesisStyles: async (thesisId) => styles.filter((s) => s.thesisId === thesisId),
  };
}

describe("the contents of a Thesis", () => {
  it("hold its Essays, its Moodboards, and its Styles", async () => {
    const c = await loadThesisContents(4, sources([style(5, 4), style(6, 4), style(7, 3)]));

    expect(c.essays.map((e) => e.title)).toEqual(["On Ash"]);
    expect(c.moodboards.map((b) => b.id)).toEqual([9]);
    expect(c.styles.map((s) => s.id)).toEqual([5, 6]);
  });

  it("have no Styles when the Thesis has none", async () => {
    expect((await loadThesisContents(4, sources([style(7, 3)]))).styles).toEqual([]);
  });
});

describe("each item of a Thesis", () => {
  const names = { thesis: "Impermanence", area: (id: string) => id };
  const trail = (view: View) => crumbs(pathOf(view, names));

  it("opens its tab from the Thesis root", () => {
    expect(trail(itemView.tab("styles"))).toEqual(["Impermanence", "Styles"]);
    expect(trail(itemView.tab("essays"))).toEqual(["Impermanence", "Essays"]);
    expect(trail(itemView.tab("moodboards"))).toEqual(["Impermanence", "Moodboards"]);
  });

  it("opens a Style under the Styles tab", () => {
    expect(trail(itemView.style(style(5, 4)))).toEqual(["Impermanence", "Styles", "S5 (Bone)"]);
  });

  it("opens an Essay under the Essays tab", () => {
    const essay = { id: 1, kind: "essay", title: "On Ash", createdAt: "", updatedAt: "" } as const;
    expect(trail(itemView.essay(essay))).toEqual(["Impermanence", "Essays", "On Ash"]);
  });

  it("opens a Moodboard under the Moodboards tab", () => {
    const board = { id: 9, owner: { kind: "thesis", id: 4 }, title: "Ash" } as const;
    expect(trail(itemView.moodboard(board))).toEqual(["Impermanence", "Moodboards", "Ash"]);
  });
});
