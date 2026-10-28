/** Mirrors `ThesisStatus` in `src-tauri/src/thesis/status.rs`. */
export type ThesisStatus = "seed" | "developing" | "committed" | "released" | "archived" | "abandoned";

export const statusLabel: Record<ThesisStatus, string> = {
  seed: "Seed",
  developing: "Developing",
  committed: "Committed",
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
  manualMoves: ThesisStatus[];
}

/** One status group of the Thesis list. The core sends every group, in lifecycle order. */
export interface ThesisGroup {
  status: ThesisStatus;
  theses: Thesis[];
}
