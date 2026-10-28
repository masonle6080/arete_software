import { call } from "../../shell/invoke";
import type { LibraryApi, LibraryInfo } from "./session";

/** IPC wrappers for `src-tauri/src/ipc/library.rs`. */
export const libraryApi: LibraryApi & { currentLibrary(): Promise<LibraryInfo | null> } = {
  createLibrary: (path) => call<LibraryInfo>("create_library", { path }),
  openLibrary: (path) => call<LibraryInfo>("open_library", { path }),
  reopenLastLibrary: () => call<LibraryInfo | null>("reopen_last_library"),
  currentLibrary: () => call<LibraryInfo | null>("current_library"),
};
