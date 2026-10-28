/**
 * Errors from the Library core arrive as `{ kind, message }`
 * (see `src-tauri/src/error.rs`). Add a kind here when you add a variant there.
 */
export type CoreErrorKind =
  | "NotALibrary"
  | "AlreadyALibrary"
  | "FolderNotEmpty"
  | "NewerSchema"
  | "NoLibraryOpen"
  | "LibraryDisconnected"
  | "Io"
  | "Database"
  | "Unknown";

export interface CoreError {
  kind: CoreErrorKind;
  message: string;
}

export function toCoreError(raw: unknown): CoreError {
  if (raw && typeof raw === "object" && "kind" in raw && "message" in raw) {
    const { kind, message } = raw as { kind: unknown; message: unknown };
    if (typeof kind === "string" && typeof message === "string") {
      return { kind: kind as CoreErrorKind, message };
    }
  }
  if (raw instanceof Error) return { kind: "Unknown", message: raw.message };
  return { kind: "Unknown", message: String(raw) };
}
