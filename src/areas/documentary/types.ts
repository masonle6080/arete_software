/** Mirrors `DocumentaryStatus` in `src-tauri/src/documentary/status.rs`. */
export type DocumentaryStatus = "planning" | "filming" | "editing" | "released" | "abandoned";

export const documentaryStatusLabel: Record<DocumentaryStatus, string> = {
  planning: "Planning",
  filming: "Filming",
  editing: "Editing",
  released: "Released",
  abandoned: "Abandoned",
};

/** Mirrors `Documentary` in `src-tauri/src/documentary/mod.rs`. */
export interface Documentary {
  id: number;
  thesisId: number;
  title: string;
  status: DocumentaryStatus;
  /** Statuses the lifecycle allows next. The core owns this rule. */
  moves: DocumentaryStatus[];
}
