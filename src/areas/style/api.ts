import { call } from "../../shell/invoke";
import type { Style, StyleFile, StyleStatus } from "./types";

/** IPC wrappers for `src-tauri/src/ipc/style.rs`. */
export const styleApi = {
  createStyle: (dropId: number, name: string, color: string) =>
    call<Style>("create_style", { dropId, name, color }),
  moveStyle: (id: number, to: StyleStatus) => call<Style>("move_style", { id, to }),
  getStyle: (id: number) => call<Style>("get_style", { id }),
  dropStyles: (dropId: number) => call<Style[]>("drop_styles", { dropId }),
  /** `sources` are paths outside the library; the core imports a copy of each. */
  attachStyleFiles: (id: number, sources: string[]) => call<StyleFile[]>("attach_style_files", { id, sources }),
  styleFiles: (id: number) => call<StyleFile[]>("style_files", { id }),
};
