import { describe, expect, it } from "vitest";
import { importExtension, mimeTypeOf } from "./mediaType";

describe("importExtension", () => {
  it("keeps the extension of a file dragged from Finder, in lower case", () => {
    expect(importExtension({ name: "Look 01.JPG", type: "image/jpeg" })).toBe("jpg");
  });

  it("uses the MIME type when a browser drag gives no usable name", () => {
    expect(importExtension({ name: "image", type: "image/webp" })).toBe("webp");
    expect(importExtension({ name: "", type: "image/svg+xml" })).toBe("svg");
  });

  it("drops an extension that is not plain ASCII letters and digits", () => {
    expect(importExtension({ name: "photo.pñg", type: "image/png" })).toBe("png");
  });

  it("gives no extension when neither the name nor the type helps", () => {
    expect(importExtension({ name: "blob", type: "" })).toBe("");
  });
});

describe("mimeTypeOf", () => {
  it("reads the MIME type from a stored media name", () => {
    expect(mimeTypeOf("6437b3ac.png")).toBe("image/png");
    expect(mimeTypeOf("6437b3ac.jpeg")).toBe("image/jpeg");
    expect(mimeTypeOf("6437b3ac.svg")).toBe("image/svg+xml");
  });

  it("falls back to a generic binary type", () => {
    expect(mimeTypeOf("6437b3ac")).toBe("application/octet-stream");
  });
});
