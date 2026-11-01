import { describe, expect, it } from "vitest";
import type { Drop } from "../drop/types";
import type { Style } from "../style/types";
import { crumbs } from "../../shell/navPath";
import { pathOf, type View } from "../../shell/sidebar";
import { itemView, loadThesisContents, type ContentSources } from "./contents";

const drop = (id: number, stage: Drop["stage"]) => ({ id, stage }) as Drop;
const style = (id: number, dropId: number) => ({ id, dropId, name: `S${id}`, color: "Bone" }) as Style;

function sources(drops: Drop[], styles: Style[]): ContentSources {
  return {
    essaysOfThesis: async () => [{ id: 1, kind: "essay", title: "On Ash", createdAt: "", updatedAt: "" }],
    moodboardsOfThesis: async (thesisId) => [{ id: 9, owner: { kind: "thesis", id: thesisId }, title: "Ash" }],
    thesisDrops: async () => ({ drops, canCreateDrop: false }),
    dropStyles: async (dropId) => styles.filter((s) => s.dropId === dropId),
  };
}

describe("the contents of a Thesis", () => {
  it("hold its Essays, its Moodboards, and the Styles of its Drop", async () => {
    const c = await loadThesisContents(4, sources([drop(2, "design")], [style(5, 2), style(6, 2)]));

    expect(c.essays.map((e) => e.title)).toEqual(["On Ash"]);
    expect(c.moodboards.map((b) => b.id)).toEqual([9]);
    expect(c.styles.map((s) => s.id)).toEqual([5, 6]);
  });

  it("take the Styles of the Drop that is not Cancelled", async () => {
    const c = await loadThesisContents(
      4,
      sources([drop(1, "cancelled"), drop(2, "design")], [style(5, 1), style(6, 2)]),
    );

    expect(c.styles.map((s) => s.id)).toEqual([6]);
  });

  it("have no Styles when the Thesis has no Drop, or only Cancelled ones", async () => {
    expect((await loadThesisContents(4, sources([], []))).styles).toEqual([]);
    expect((await loadThesisContents(4, sources([drop(1, "cancelled")], [style(5, 1)]))).styles).toEqual([]);
  });
});

describe("each item of a Thesis", () => {
  const names = { thesis: "Impermanence", area: (id: string) => id };
  const trail = (view: View) => crumbs(pathOf(view, names));

  it("opens its tab from the Thesis root", () => {
    expect(trail(itemView.tab("drop"))).toEqual(["Impermanence", "Drop"]);
    expect(trail(itemView.tab("essays"))).toEqual(["Impermanence", "Essays"]);
    expect(trail(itemView.tab("moodboards"))).toEqual(["Impermanence", "Moodboards"]);
  });

  it("opens a Style under the Drop tab, as its tree row does", () => {
    expect(trail(itemView.style(style(5, 2)))).toEqual(["Impermanence", "Drop", "S5 (Bone)"]);
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
