import { describe, expect, it } from "vitest";
import { startSession, type LibraryApi } from "./session";

const info = { path: "/Volumes/Drive/Arete", schemaVersion: 1 };

function fakeApi(reopen: LibraryApi["reopenLastLibrary"]): LibraryApi {
  return {
    reopenLastLibrary: reopen,
    createLibrary: async () => info,
    openLibrary: async () => info,
  };
}

describe("startSession", () => {
  it("goes straight to the last library when it reopens", async () => {
    const state = await startSession(fakeApi(async () => info));

    expect(state).toEqual({ status: "open", library: info });
  });

  it("shows the welcome screen on first launch", async () => {
    const state = await startSession(fakeApi(async () => null));

    expect(state).toEqual({ status: "welcome" });
  });

  it("shows the welcome screen with the reason when the last library fails to open", async () => {
    const state = await startSession(
      fakeApi(async () => {
        throw { kind: "NotALibrary", message: "/old is not an Arete library" };
      }),
    );

    expect(state).toEqual({
      status: "welcome",
      error: { kind: "NotALibrary", message: "/old is not an Arete library" },
    });
  });
});
