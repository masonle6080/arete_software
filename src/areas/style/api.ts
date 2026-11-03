import { call } from "../../shell/invoke";
import type { Style, StyleBase, StyleFile, StyleStatus } from "./types";

/** IPC wrappers for `src-tauri/src/ipc/style.rs`. */
export const styleApi = {
  createStyle: (thesisId: number, name: string, color: string) =>
    call<Style>("create_style", { thesisId, name, color }),
  /** The same design in another color: a new Draft Style on the same Thesis. Samples are not copied. */
  duplicateStyle: (id: number, color: string) => call<Style>("duplicate_style", { id, color }),
  moveStyle: (id: number, to: StyleStatus) => call<Style>("move_style", { id, to }),
  getStyle: (id: number) => call<Style>("get_style", { id }),
  thesisStyles: (thesisId: number) => call<Style[]>("thesis_styles", { thesisId }),
  /** `sources` are paths outside the library; the core imports a copy of each. */
  attachStyleFiles: (id: number, sources: string[]) => call<StyleFile[]>("attach_style_files", { id, sources }),
  styleFiles: (id: number) => call<StyleFile[]>("style_files", { id }),
  /** Returns the files left. Removing the Cover file clears the Cover. */
  removeStyleFile: (id: number, fileId: number) => call<StyleFile[]>("remove_style_file", { id, fileId }),
  /** Only a PNG, JPEG, WebP, or HEIC file of this Style can be its Cover. */
  setStyleCover: (id: number, fileId: number) => call<Style>("set_style_cover", { id, fileId }),
  clearStyleCover: (id: number) => call<Style>("clear_style_cover", { id }),
  /** `null` when the Style has no base yet. */
  styleBase: (id: number) => call<StyleBase | null>("style_base", { id }),
  /** The Style keeps only the sizes the new base also offers. */
  setStyleBase: (id: number, base: StyleBase) => call<StyleBase>("set_style_base", { id, base }),
  /** The Style's sizes, in the order its base lists them. */
  styleSizes: (id: number) => call<string[]>("style_sizes", { id }),
  setStyleSizes: (id: number, sizes: string[]) => call<string[]>("set_style_sizes", { id, sizes }),
};
