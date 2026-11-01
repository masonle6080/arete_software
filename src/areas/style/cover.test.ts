import { describe, expect, it } from "vitest";
import { canBeCover } from "./cover";

describe("canBeCover", () => {
  it("accepts PNG, JPEG, WebP, and HEIC files in any letter case", () => {
    for (const name of ["a.png", "b.jpg", "c.jpeg", "d.webp", "e.heic", "F.PNG", "g.JpG", "front view.HEIC"]) {
      expect(canBeCover(name), name).toBe(true);
    }
  });

  it("refuses other files", () => {
    for (const name of ["flat.ai", "spec.pdf", "art.procreate", "anim.gif", "noext", "png", "a.png.ai"]) {
      expect(canBeCover(name), name).toBe(false);
    }
  });
});
