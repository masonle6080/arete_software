import { toCoreError } from "../../shell/coreError";
import type { LibraryInfo } from "./session";

export interface ReconnectApi {
  reconnectLibrary(): Promise<LibraryInfo>;
}

/** How often the app checks for a disconnected library's folder. */
export const RECONNECT_INTERVAL_MS = 2000;

/**
 * Polls until the disconnected library's folder is back (ADR-0002), then
 * returns the reconnected library. Returns null when `signal` aborts.
 * Throws any error other than `LibraryDisconnected`.
 */
export async function waitForReconnect(
  api: ReconnectApi,
  { intervalMs = RECONNECT_INTERVAL_MS, signal }: { intervalMs?: number; signal?: AbortSignal } = {},
): Promise<LibraryInfo | null> {
  while (!signal?.aborted) {
    try {
      return await api.reconnectLibrary();
    } catch (raw) {
      const err = toCoreError(raw);
      if (err.kind !== "LibraryDisconnected") throw err;
    }
    if (signal?.aborted) break;
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }
  return null;
}
