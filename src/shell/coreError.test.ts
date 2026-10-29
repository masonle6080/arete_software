import { describe, expect, it } from "vitest";
import { toCoreError } from "./coreError";

describe("toCoreError", () => {
  it("keeps the kind and message the Library core sends", () => {
    const err = toCoreError({ kind: "NotALibrary", message: "/x is not an Arete library" });

    expect(err).toEqual({ kind: "NotALibrary", message: "/x is not an Arete library" });
  });

  it("wraps a plain string error as Unknown", () => {
    expect(toCoreError("boom")).toEqual({ kind: "Unknown", message: "boom" });
  });

  it("wraps a JS Error as Unknown", () => {
    expect(toCoreError(new Error("bad"))).toEqual({ kind: "Unknown", message: "bad" });
  });
});
