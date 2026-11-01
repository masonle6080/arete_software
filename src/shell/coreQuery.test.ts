import { describe, expect, it } from "vitest";
import { startQuery } from "./coreQuery";

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("a core query", () => {
  it("hands over what the core returns", async () => {
    const got: string[] = [];
    startQuery(
      () => Promise.resolve("Amor fati"),
      (data) => got.push(data),
      () => got.push("error"),
    );
    await settle();
    expect(got).toEqual(["Amor fati"]);
  });

  it("hands over a refusal as a CoreError", async () => {
    const errors: unknown[] = [];
    startQuery(
      () => Promise.reject({ kind: "NotFound", message: "Style 9 not found" }),
      () => errors.push("data"),
      (error) => errors.push(error),
    );
    await settle();
    expect(errors).toEqual([{ kind: "NotFound", message: "Style 9 not found" }]);
  });

  it("drops a late answer after it is cancelled", async () => {
    const got: string[] = [];
    const cancel = startQuery(
      () => Promise.resolve("stale"),
      (data) => got.push(data),
      () => got.push("error"),
    );
    cancel();
    await settle();
    expect(got).toEqual([]);
  });

  it("drops a late refusal after it is cancelled", async () => {
    const got: string[] = [];
    const cancel = startQuery(
      () => Promise.reject(new Error("gone")),
      () => got.push("data"),
      () => got.push("error"),
    );
    cancel();
    await settle();
    expect(got).toEqual([]);
  });
});
