import { invoke } from "@tauri-apps/api/core";
import { toCoreError } from "./coreError";

/**
 * Calls a Library core command. On failure it throws a `CoreError`.
 * Each area wraps its commands in `src/areas/<area>/api.ts`.
 */
export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (raw) {
    const err = toCoreError(raw);
    if (err.kind === "LibraryDisconnected") disconnectListeners.forEach((l) => l());
    throw err;
  }
}

const disconnectListeners = new Set<() => void>();

/**
 * Calls `listener` whenever any command fails with `LibraryDisconnected`.
 * The shell uses it to show the reconnect screen. Returns an unsubscribe.
 */
export function onLibraryDisconnected(listener: () => void): () => void {
  disconnectListeners.add(listener);
  return () => disconnectListeners.delete(listener);
}
