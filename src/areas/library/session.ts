import { toCoreError, type CoreError } from "../../shell/coreError";

export interface LibraryInfo {
  path: string;
  schemaVersion: number;
}

export interface LibraryApi {
  createLibrary(path: string): Promise<LibraryInfo>;
  openLibrary(path: string): Promise<LibraryInfo>;
  reopenLastLibrary(): Promise<LibraryInfo | null>;
}

export type SessionState =
  | { status: "open"; library: LibraryInfo }
  | { status: "disconnected"; library: LibraryInfo }
  | { status: "welcome"; error?: CoreError };

/** On launch: reopen the last library, else show the welcome screen. */
export async function startSession(api: LibraryApi): Promise<SessionState> {
  try {
    const library = await api.reopenLastLibrary();
    return library ? { status: "open", library } : { status: "welcome" };
  } catch (raw) {
    return { status: "welcome", error: toCoreError(raw) };
  }
}
