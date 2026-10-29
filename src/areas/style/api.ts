import { call } from "../../shell/invoke";
import type { Style, StyleBase, StyleFile, StyleStatus } from "./types";

/** IPC wrappers for `src-tauri/src/ipc/style.rs`. */
export const styleApi = {
  createStyle: (dropId: number, name: string, color: string) =>
    call<Style>("create_style", { dropId, name, color }),
  /** The same design in another color: a new Draft Style on the same Drop. Samples are not copied. */
  duplicateStyle: (id: number, color: string) => call<Style>("duplicate_style", { id, color }),
  moveStyle: (id: number, to: StyleStatus) => call<Style>("move_style", { id, to }),
  getStyle: (id: number) => call<Style>("get_style", { id }),
  dropStyles: (dropId: number) => call<Style[]>("drop_styles", { dropId }),
  /** `sources` are paths outside the library; the core imports a copy of each. */
  attachStyleFiles: (id: number, sources: string[]) => call<StyleFile[]>("attach_style_files", { id, sources }),
  styleFiles: (id: number) => call<StyleFile[]>("style_files", { id }),
  /** `null` when the Style has no base yet. */
  styleBase: (id: number) => call<StyleBase | null>("style_base", { id }),
  /** The Style keeps only the sizes the new base also offers. */
  setStyleBase: (id: number, base: StyleBase) => call<StyleBase>("set_style_base", { id, base }),
  /** The Style's sizes, in the order its base lists them. */
  styleSizes: (id: number) => call<string[]>("style_sizes", { id }),
  setStyleSizes: (id: number, sizes: string[]) => call<string[]>("set_style_sizes", { id, sizes }),
};
