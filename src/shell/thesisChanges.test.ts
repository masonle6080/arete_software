import { describe, expect, it } from "vitest";
import { createThesisChanges } from "./thesisChanges";

describe("Thesis changes", () => {
  it("tell every listener that a Style, Essay, or Moodboard changed", () => {
    const changes = createThesisChanges();
    const heard: string[] = [];
    changes.listen(() => heard.push("a"));
    changes.listen(() => heard.push("b"));

    changes.announce();

    expect(heard).toEqual(["a", "b"]);
  });

  it("stop reaching a listener after it stops listening", () => {
    const changes = createThesisChanges();
    let heard = 0;
    const stop = changes.listen(() => heard++);

    changes.announce();
    stop();
    changes.announce();

    expect(heard).toBe(1);
  });
});
