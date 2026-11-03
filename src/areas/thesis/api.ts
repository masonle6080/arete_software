import { call } from "../../shell/invoke";
import type { Thesis, ThesisGroup, ThesisMove, ThesisStatus } from "./types";

/** IPC wrappers for `src-tauri/src/ipc/thesis.rs`. */
export const thesisApi = {
  createThesis: (title: string, statement: string) => call<Thesis>("create_thesis", { title, statement }),
  /** Stops with warnings unless `confirmed`; repeat with `confirmed` to go past them. */
  moveThesis: (id: number, to: ThesisStatus, confirmed = false) =>
    call<ThesisMove>("move_thesis", { id, to, confirmed }),
  listTheses: () => call<ThesisGroup[]>("list_theses"),
  getThesis: (id: number) => call<Thesis>("get_thesis", { id }),
};
