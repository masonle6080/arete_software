import { describe, expect, it } from "vitest";
import { countLabel } from "./count";

describe("a count label", () => {
  it("uses the singular for one and the plural otherwise", () => {
    expect(countLabel(1, "Essay")).toBe("1 Essay");
    expect(countLabel(0, "Passage")).toBe("0 Passages");
    expect(countLabel(3, "Design Decision")).toBe("3 Design Decisions");
  });
});
