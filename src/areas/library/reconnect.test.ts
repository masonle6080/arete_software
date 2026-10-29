import { describe, expect, it } from "vitest";
import { waitForReconnect } from "./reconnect";

const info = { path: "/Volumes/Drive/Arete", schemaVersion: 1 };
const disconnected = { kind: "LibraryDisconnected", message: "the library at /Volumes/Drive/Arete is disconnected" };

describe("waitForReconnect", () => {
  it("keeps trying while the library is disconnected and returns it when the drive is back", async () => {
    let attempts = 0;
    const api = {
      reconnectLibrary: async () => {
        attempts += 1;
        if (attempts < 3) throw disconnected;
        return info;
      },
    };

    const library = await waitForReconnect(api, { intervalMs: 0 });

    expect(library).toEqual(info);
    expect(attempts).toBe(3);
  });

  it("stops with the error when reconnecting fails for another reason", async () => {
    const api = {
      reconnectLibrary: async () => {
        throw { kind: "Database", message: "database error: disk I/O error" };
      },
    };

    await expect(waitForReconnect(api, { intervalMs: 0 })).rejects.toEqual({
      kind: "Database",
      message: "database error: disk I/O error",
    });
  });

  it("stops quietly when cancelled", async () => {
    const controller = new AbortController();
    const api = {
      reconnectLibrary: async () => {
        controller.abort();
        throw disconnected;
      },
    };

    const library = await waitForReconnect(api, { intervalMs: 0, signal: controller.signal });

    expect(library).toBeNull();
  });
});
