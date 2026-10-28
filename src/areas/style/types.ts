/** Mirrors `StyleStatus` in `src-tauri/src/style/status.rs`. */
export type StyleStatus = "draft" | "sampling" | "approved" | "cut";

export const styleStatusLabel: Record<StyleStatus, string> = {
  draft: "Draft",
  sampling: "Sampling",
  approved: "Approved",
  cut: "Cut",
};

/** Mirrors `Style` in `src-tauri/src/style/mod.rs`. */
export interface Style {
  id: number;
  dropId: number;
  name: string;
  /** The one color of this Style. */
  color: string;
  status: StyleStatus;
  /** Statuses the lifecycle allows next. The core owns this rule. */
  moves: StyleStatus[];
}

/** Mirrors `StyleFile`: one design file attached to a Style. */
export interface StyleFile {
  id: number;
  media: string;
  /** The name the file had when it was attached. */
  fileName: string;
  /** Absolute path of the stored copy in the library. */
  path: string;
}
