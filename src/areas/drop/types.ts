import { documentaryStatusLabel, type DocumentaryStatus } from "../documentary/types";

/** Mirrors `DropStage` in `src-tauri/src/drop/stage.rs`. */
export type DropStage = "design" | "sampling" | "production" | "released" | "cancelled";

export const stageLabel: Record<DropStage, string> = {
  design: "Design",
  sampling: "Sampling",
  production: "Production",
  released: "Released",
  cancelled: "Cancelled",
};

/** Mirrors `Drop` in `src-tauri/src/drop/mod.rs`. */
export interface Drop {
  id: number;
  thesisId: number;
  name: string;
  stage: DropStage;
  /** Stages the lifecycle allows next. The core owns this rule. */
  moves: DropStage[];
  /** Whether the Drop takes new or duplicated Styles now. The core owns this rule. */
  canAddStyles: boolean;
}

/** Mirrors `DropWarning`: a reason a move needs confirmation. Tagged by `kind`. */
export type DropWarning = {
  kind: "documentaryNotReleased";
  documentaryId: number;
  title: string;
  status: DocumentaryStatus;
};

/** The founder-facing text of a warning. */
export function warningText(w: DropWarning): string {
  switch (w.kind) {
    case "documentaryNotReleased":
      return `The Documentary "${w.title}" is ${documentaryStatusLabel[w.status]}, not Released. The Documentary should come out before the Drop.`;
  }
}

/** Mirrors `DropMove`, the result of `move_drop`. */
export type DropMove =
  | ({ outcome: "moved" } & Drop)
  | { outcome: "needsConfirmation"; drop: Drop; warnings: DropWarning[] };

/** Mirrors `ThesisDrops`. */
export interface ThesisDrops {
  /** Oldest first, Cancelled ones included. At most one is not Cancelled. */
  drops: Drop[];
  canCreateDrop: boolean;
}
