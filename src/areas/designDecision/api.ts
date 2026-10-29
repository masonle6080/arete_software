import { call } from "../../shell/invoke";
import type { Passage } from "../passage/api";
import type { StyleStatus } from "../style/types";

/** Mirrors `DesignAspect` in `src-tauri/src/design_decision/mod.rs`. */
export type DesignAspect = "decoration" | "fabric" | "color" | "silhouette" | "other";

export const designAspects: DesignAspect[] = ["decoration", "fabric", "color", "silhouette", "other"];

export const designAspectLabel: Record<DesignAspect, string> = {
  decoration: "Decoration",
  fabric: "Fabric",
  color: "Color",
  silhouette: "Silhouette",
  other: "Other",
};

/** Mirrors `DesignDecision`: its linked Passages are in the order linked. */
export interface DesignDecision {
  id: number;
  styleId: number;
  aspect: DesignAspect;
  rationale: string;
  passages: Passage[];
}

/** Mirrors `PassageDesignDecision`: a Design Decision as a Passage lists it. */
export interface PassageDesignDecision {
  id: number;
  styleId: number;
  styleName: string;
  styleColor: string;
  styleStatus: StyleStatus;
  aspect: DesignAspect;
  rationale: string;
}

/** IPC wrappers for `src-tauri/src/ipc/design_decision.rs`. */
export const designDecisionApi = {
  /** The core refuses an empty rationale. */
  add: (styleId: number, aspect: DesignAspect, rationale: string) =>
    call<DesignDecision>("add_design_decision", { styleId, aspect, rationale }),
  ofStyle: (styleId: number) => call<DesignDecision[]>("style_design_decisions", { styleId }),
  linkPassage: (id: number, passageId: string) =>
    call<DesignDecision>("link_design_decision_passage", { id, passageId }),
  unlinkPassage: (id: number, passageId: string) =>
    call<DesignDecision>("unlink_design_decision_passage", { id, passageId }),
  ofPassage: (passageId: string) => call<PassageDesignDecision[]>("passage_design_decisions", { passageId }),
};
