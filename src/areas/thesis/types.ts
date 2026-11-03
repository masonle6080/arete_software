import { documentaryStatusLabel, type DocumentaryStatus } from "../documentary/types";

/** Mirrors `ThesisStatus` in `src-tauri/src/thesis/status.rs`, in lifecycle order. */
export type ThesisStatus =
  | "seed"
  | "developing"
  | "design"
  | "sampling"
  | "production"
  | "released"
  | "archived"
  | "abandoned";

export const statusLabel: Record<ThesisStatus, string> = {
  seed: "Seed",
  developing: "Developing",
  design: "Design",
  sampling: "Sampling",
  production: "Production",
  released: "Released",
  archived: "Archived",
  abandoned: "Abandoned",
};

/** Mirrors `Thesis` in `src-tauri/src/thesis/mod.rs`. */
export interface Thesis {
  id: number;
  title: string;
  statement: string;
  status: ThesisStatus;
  /** Statuses the founder can move to by hand now. The core owns this rule. */
  moves: ThesisStatus[];
  /** Whether the Thesis takes new or duplicated Styles now. The core owns this rule. */
  canAddStyles: boolean;
  /** Whether the Thesis takes new Production Orders now. The core owns this rule. */
  canAddProductionOrders: boolean;
}

/** One status group of the Thesis list. The core sends every group, in lifecycle order. */
export interface ThesisGroup {
  status: ThesisStatus;
  theses: Thesis[];
}

/** Mirrors `ThesisWarning`: a reason a move needs confirmation. Tagged by `kind`. */
export type ThesisWarning = {
  kind: "documentaryNotReleased";
  documentaryId: number;
  title: string;
  status: DocumentaryStatus;
};

/** The founder-facing text of a warning. */
export function warningText(w: ThesisWarning): string {
  switch (w.kind) {
    case "documentaryNotReleased":
      return `The Documentary "${w.title}" is ${documentaryStatusLabel[w.status]}, not Released. The Documentary should come out before the Thesis is Released.`;
  }
}

/** Mirrors `ThesisMove`, the result of `move_thesis`. */
export type ThesisMove =
  | ({ outcome: "moved" } & Thesis)
  | { outcome: "needsConfirmation"; thesis: Thesis; warnings: ThesisWarning[] };
