import { describe, expect, it } from "vitest";
import { createLeaveGuards } from "./leaveGuards";

describe("leave guards", () => {
  it("finish every registered step before the shell leaves the screen", async () => {
    const guards = createLeaveGuards();
    const saved: string[] = [];
    let finishSave!: () => void;
    guards.add(() => new Promise<void>((resolve) => (finishSave = resolve)).then(() => void saved.push("essay")));

    let left = false;
    const leaving = guards.run().then(() => (left = true));
    await Promise.resolve();
    expect(left).toBe(false);

    finishSave();
    await leaving;
    expect(saved).toEqual(["essay"]);
    expect(left).toBe(true);
  });

  it("stop running a step once its screen removes it", async () => {
    const guards = createLeaveGuards();
    let runs = 0;
    const remove = guards.add(async () => void runs++);
    await guards.run();
    remove();
    await guards.run();
    expect(runs).toBe(1);
  });

  it("let the shell leave even when a step fails, because the editor shows its own save error", async () => {
    const guards = createLeaveGuards();
    guards.add(() => Promise.reject(new Error("disk full")));
    await expect(guards.run()).resolves.toBeUndefined();
  });
});
