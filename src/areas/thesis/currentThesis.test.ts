import { describe, expect, it } from "vitest";
import { loadCurrentThesis, saveCurrentThesis } from "./currentThesis";

function memoryStorage() {
  const items = new Map<string, string>();
  return {
    getItem: (k: string) => items.get(k) ?? null,
    setItem: (k: string, v: string) => void items.set(k, v),
    removeItem: (k: string) => void items.delete(k),
  };
}

describe("current Thesis", () => {
  it("is remembered for each library folder", () => {
    const storage = memoryStorage();
    saveCurrentThesis("/a", 3, storage);
    saveCurrentThesis("/b", 7, storage);

    expect(loadCurrentThesis("/a", storage)).toBe(3);
    expect(loadCurrentThesis("/b", storage)).toBe(7);
  });

  it("is undefined when never picked, cleared, or not an ID", () => {
    const storage = memoryStorage();
    expect(loadCurrentThesis("/a", storage)).toBeUndefined();

    saveCurrentThesis("/a", 3, storage);
    saveCurrentThesis("/a", undefined, storage);
    expect(loadCurrentThesis("/a", storage)).toBeUndefined();

    storage.setItem("arete.currentThesis:/a", "abc");
    expect(loadCurrentThesis("/a", storage)).toBeUndefined();
  });
});
