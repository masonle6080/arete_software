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
    throw toCoreError(raw);
  }
}
