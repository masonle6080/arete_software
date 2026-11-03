import { call } from "../../shell/invoke";
import type { Blank } from "../blank/api";
import type { Documentary } from "../documentary/types";
import type { Thesis } from "../thesis/types";

/** Mirrors `ThesisLinks` in `src-tauri/src/links/mod.rs`. */
export interface ThesisLinks {
  essayCount: number;
  documentary: Documentary | null;
}

/** Mirrors `EssayLinks` in `src-tauri/src/links/mod.rs`. */
export interface EssayLinks {
  theses: Thesis[];
  /** Passages in the Essay's text; orphaned Passages are not counted. */
  passageCount: number;
  /** Design Decisions that those Passages motivate. */
  designDecisionCount: number;
}

/** Mirrors `BaseLink` in `src-tauri/src/links/mod.rs`. */
export type BaseLink = { kind: "blank"; blank: Blank } | { kind: "techPack" };

/** Mirrors `StyleLinks` in `src-tauri/src/links/mod.rs`. */
export interface StyleLinks {
  thesis: Thesis;
  base: BaseLink | null;
  /** Passages that motivate the Style's Design Decisions, orphaned ones included. */
  passageCount: number;
}

/** IPC wrappers for `src-tauri/src/ipc/links.rs`: the links that screen headers show as chips. */
export const linksApi = {
  thesis: (thesisId: number) => call<ThesisLinks>("thesis_links", { thesisId }),
  essay: (essayId: number) => call<EssayLinks>("essay_links", { essayId }),
  style: (styleId: number) => call<StyleLinks>("style_links", { styleId }),
};
