import { describe, expect, it } from "vitest";
import { canBeCover, cardFace } from "./cover";
import type { Style } from "./types";

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

describe("cardFace", () => {
  const style: Style = { id: 1, thesisId: 1, name: "Hooded Coat", color: "Ash", status: "draft", moves: [], cover: null };

  it("shows the Cover image when the Style has a Cover", () => {
    const cover = { id: 7, media: "m", fileName: "front.png", path: "/lib/media/front.png" };
    expect(cardFace({ ...style, cover })).toEqual({ kind: "cover", path: "/lib/media/front.png" });
  });

  it("shows a neutral placeholder with the color name when the Style has no Cover", () => {
    expect(cardFace(style)).toEqual({ kind: "placeholder", color: "Ash" });
  });
});
