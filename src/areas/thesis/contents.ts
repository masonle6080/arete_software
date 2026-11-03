/**
 * What a Thesis holds, for the Overview: its linked
 * Essays, its Moodboards, and its Styles. Pure apart from the
 * injected sources, so it is tested without the core.
 */
import { steps, type ThesisTab } from "../../shell/navPath";
import type { View } from "../../shell/sidebar";
import { essayThesisApi, type DocumentSummary } from "../essays/api";
import { moodboardApi } from "../moodboard/api";
import type { Moodboard } from "../moodboard/types";
import { styleApi } from "../style/api";
import type { Style } from "../style/types";

export interface ThesisContents {
  essays: DocumentSummary[];
  moodboards: Moodboard[];
  /** The Styles of the Thesis, Cut ones included. */
  styles: Style[];
}

/** Where the contents come from. The default reads the Library core. */
export interface ContentSources {
  essaysOfThesis(thesisId: number): Promise<DocumentSummary[]>;
  moodboardsOfThesis(thesisId: number): Promise<Moodboard[]>;
  thesisStyles(thesisId: number): Promise<Style[]>;
}

const coreSources: ContentSources = {
  essaysOfThesis: essayThesisApi.essaysOfThesis,
  moodboardsOfThesis: (id) => moodboardApi.listMoodboards({ kind: "thesis", id }),
  thesisStyles: styleApi.thesisStyles,
};

/** Loads the contents of Thesis `thesisId`. */
export async function loadThesisContents(thesisId: number, sources = coreSources): Promise<ThesisContents> {
  const [essays, moodboards, styles] = await Promise.all([
    sources.essaysOfThesis(thesisId),
    sources.moodboardsOfThesis(thesisId),
    sources.thesisStyles(thesisId),
  ]);
  return { essays, moodboards, styles };
}

/**
 * What opening each item of the current Thesis does on the Overview: a tab
 * opens from the Thesis root, and an item opens on top of its tab, so the
 * breadcrumb reads `Thesis › Styles › Style`.
 */
export const itemView = {
  tab: (tab: ThesisTab): View => ({ kind: "thesis", tab }),
  essay: (e: DocumentSummary): View => ({ kind: "thesis", tab: "essays", open: steps.essay(e.id, e.title) }),
  moodboard: (b: Moodboard): View => ({ kind: "thesis", tab: "moodboards", open: steps.moodboard(b) }),
  style: (s: Style): View => ({ kind: "thesis", tab: "styles", open: steps.style(s) }),
};
