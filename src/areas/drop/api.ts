import { call } from "../../shell/invoke";
import type { Drop, DropMove, DropStage, ThesisDrops } from "./types";

/** IPC wrappers for `src-tauri/src/ipc/drop.rs`. */
export const dropApi = {
  createDrop: (thesisId: number, name: string) => call<Drop>("create_drop", { thesisId, name }),
  moveDrop: (id: number, to: DropStage, confirmed = false) => call<DropMove>("move_drop", { id, to, confirmed }),
  thesisDrops: (thesisId: number) => call<ThesisDrops>("thesis_drops", { thesisId }),
};
