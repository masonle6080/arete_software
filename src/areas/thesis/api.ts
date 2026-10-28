import { call } from "../../shell/invoke";
import type { Thesis, ThesisGroup, ThesisStatus } from "./types";

/** IPC wrappers for `src-tauri/src/ipc/thesis.rs`. */
export const thesisApi = {
  createThesis: (title: string, statement: string) => call<Thesis>("create_thesis", { title, statement }),
  moveThesis: (id: number, to: ThesisStatus) => call<Thesis>("move_thesis", { id, to }),
  listTheses: () => call<ThesisGroup[]>("list_theses"),
  getThesis: (id: number) => call<Thesis>("get_thesis", { id }),
};
