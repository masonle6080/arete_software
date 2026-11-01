/**
 * What a Thesis holds, for the sidebar tree and the Overview: its linked
 * Essays, its Moodboards, and the Styles of its Drop. Pure apart from the
 * injected sources, so it is tested without the core.
 */
import { steps, type ThesisTab } from "../../shell/navPath";
import type { View } from "../../shell/sidebar";
import { dropApi } from "../drop/api";
import type { ThesisDrops } from "../drop/types";
import { essayThesisApi, type DocumentSummary } from "../essays/api";
import { moodboardApi } from "../moodboard/api";
import type { Moodboard } from "../moodboard/types";
import { styleApi } from "../style/api";
import type { Style } from "../style/types";

export interface ThesisContents {
  essays: DocumentSummary[];
  moodboards: Moodboard[];
  /** The Styles of the Thesis's active (not Cancelled) Drop, Cut ones included. Empty when it has none. */
  styles: Style[];
}

/** Where the contents come from. The default reads the Library core. */
export interface ContentSources {
  essaysOfThesis(thesisId: number): Promise<DocumentSummary[]>;
  moodboardsOfThesis(thesisId: number): Promise<Moodboard[]>;
  thesisDrops(thesisId: number): Promise<ThesisDrops>;
  dropStyles(dropId: number): Promise<Style[]>;
}

const coreSources: ContentSources = {
  essaysOfThesis: essayThesisApi.essaysOfThesis,
  moodboardsOfThesis: (id) => moodboardApi.listMoodboards({ kind: "thesis", id }),
  thesisDrops: dropApi.thesisDrops,
  dropStyles: styleApi.dropStyles,
};

/** Loads the contents of Thesis `thesisId`. */
export async function loadThesisContents(thesisId: number, sources = coreSources): Promise<ThesisContents> {
  const [essays, moodboards, drops] = await Promise.all([
    sources.essaysOfThesis(thesisId),
    sources.moodboardsOfThesis(thesisId),
    sources.thesisDrops(thesisId),
  ]);
  const active = drops.drops.find((d) => d.stage !== "cancelled");
  const styles = active ? await sources.dropStyles(active.id) : [];
  return { essays, moodboards, styles };
}

/**
 * What opening each item of the current Thesis does, in the tree and on the
 * Overview: a tab opens from the Thesis root, and an item opens on top of
 * its tab, so the breadcrumb reads `Thesis › Drop › Style`.
 */
export const itemView = {
  tab: (tab: ThesisTab): View => ({ kind: "thesis", tab }),
  essay: (e: DocumentSummary): View => ({ kind: "thesis", tab: "essays", open: steps.essay(e.id, e.title) }),
  moodboard: (b: Moodboard): View => ({ kind: "thesis", tab: "moodboards", open: steps.moodboard(b) }),
  style: (s: Style): View => ({ kind: "thesis", tab: "drop", open: steps.style(s) }),
};
