// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import type { TechPack } from "./techPack";
import { flatImageSrc, techPackExportHtml } from "./techPackExport";

function techPack(overrides: Partial<TechPack> = {}): TechPack {
  return {
    styleId: 7,
    sizes: [
      { id: 1, name: "S" },
      { id: 2, name: "M" },
    ],
    pointsOfMeasure: [
      { id: 10, name: "Chest width", tolerance: 0.5, values: [20, 21.5] },
      { id: 11, name: "Body length", tolerance: 1, values: [27, null] },
    ],
    details: {
      unit: "in",
      fabric: { composition: "100% cotton", weight: "240 gsm", color: "Bone", vendorId: 3 },
      trims: "Woven neck label",
      constructionNotes: "Double-needle hem.\nCoverstitch at neck.",
      labelPlacement: "Center back neck",
    },
    flats: [
      { id: 100, media: "aa.png", fileName: "front.png", path: "/lib/media/aa.png" },
      { id: 101, media: "bb.svg", fileName: "back.svg", path: "/lib/media/bb.svg" },
    ],
    ...overrides,
  };
}

/** Parses the fragment so tests read it as a manufacturing Vendor would: by section. */
function render(html: string): HTMLElement {
  const root = document.createElement("div");
  root.innerHTML = html;
  return root;
}

function section(root: HTMLElement, heading: string): HTMLElement {
  const h = Array.from(root.querySelectorAll("h2")).find((el) => el.textContent === heading);
  if (!h) throw new Error(`no section "${heading}"`);
  return h.closest("section")!;
}

const base = {
  styleName: "Heavy Tee",
  styleColor: "Bone",
  fabricVendorName: "Acme Mills",
  flatImages: { 100: "data:image/png;base64,AAA", 101: "data:image/svg+xml;base64,BBB" },
};

describe("techPackExportHtml", () => {
  it("titles the export with the Style name and color", () => {
    const root = render(techPackExportHtml({ ...base, techPack: techPack() }));
    expect(root.querySelector("h1")?.textContent).toBe("Heavy Tee — Tech Pack");
    expect(root.textContent).toContain("Color: Bone");
  });

  it("shows the Points of Measure as a table: one column per size, empty cells blank", () => {
    const root = render(techPackExportHtml({ ...base, techPack: techPack() }));
    const table = section(root, "Points of Measure (in)").querySelector("table")!;
    const rows = Array.from(table.querySelectorAll("tr")).map((tr) =>
      Array.from(tr.children).map((c) => c.textContent),
    );
    expect(rows).toEqual([
      ["Point of Measure", "Tolerance ±", "S", "M"],
      ["Chest width", "0.5", "20", "21.5"],
      ["Body length", "1", "27", ""],
    ]);
  });

  it("shows fabric composition, weight, color, and the fabric Vendor", () => {
    const root = render(techPackExportHtml({ ...base, techPack: techPack() }));
    const text = section(root, "Fabric").textContent;
    expect(text).toContain("100% cotton");
    expect(text).toContain("240 gsm");
    expect(text).toContain("Bone");
    expect(text).toContain("Acme Mills");
  });

  it("shows trims, construction notes, and label placement, keeping line breaks", () => {
    const root = render(techPackExportHtml({ ...base, techPack: techPack() }));
    expect(section(root, "Trims").textContent).toContain("Woven neck label");
    expect(section(root, "Label placement").textContent).toContain("Center back neck");
    const notes = section(root, "Construction notes");
    expect(notes.innerHTML).toContain("Double-needle hem.<br>Coverstitch at neck.");
  });

  it("shows each flat as an image with its file name as the caption", () => {
    const root = render(techPackExportHtml({ ...base, techPack: techPack() }));
    const figures = Array.from(section(root, "Flats").querySelectorAll("figure"));
    expect(figures.map((f) => f.querySelector("img")?.getAttribute("src"))).toEqual([
      "data:image/png;base64,AAA",
      "data:image/svg+xml;base64,BBB",
    ]);
    expect(figures.map((f) => f.querySelector("figcaption")?.textContent)).toEqual(["front.png", "back.svg"]);
  });

  it("names a flat whose image could not be read instead of dropping it", () => {
    const root = render(techPackExportHtml({ ...base, flatImages: { 100: "data:x" }, techPack: techPack() }));
    const flats = section(root, "Flats");
    expect(flats.querySelectorAll("img")).toHaveLength(1);
    expect(flats.textContent).toContain("back.svg");
    expect(flats.textContent).toContain("Image not available");
  });

  it("marks empty parts as not specified, so a manufacturing Vendor sees nothing is missing by mistake", () => {
    const empty = techPack({
      sizes: [],
      pointsOfMeasure: [],
      flats: [],
      details: {
        unit: "cm",
        fabric: { composition: "", weight: "", color: "", vendorId: null },
        trims: "",
        constructionNotes: "  ",
        labelPlacement: "",
      },
    });
    const root = render(techPackExportHtml({ ...base, fabricVendorName: null, techPack: empty }));
    for (const heading of ["Points of Measure (cm)", "Trims", "Construction notes", "Label placement", "Flats"]) {
      expect(section(root, heading).textContent).toContain("Not specified");
    }
    expect(section(root, "Fabric").textContent).toContain("Not specified");
  });

  it("escapes user text so it cannot inject markup", () => {
    const evil = techPack({
      details: { ...techPack().details, trims: '<img src=x onerror="alert(1)">' },
      flats: [{ id: 100, media: "aa.png", fileName: '"><script>x</script>', path: "/p" }],
    });
    const root = render(
      techPackExportHtml({ ...base, styleName: "<b>Tee</b>", techPack: evil, flatImages: { 100: 'a" onload="x' } }),
    );
    expect(root.querySelector("b")).toBeNull();
    expect(root.querySelector("script")).toBeNull();
    expect(section(root, "Trims").querySelector("img")).toBeNull();
    expect(section(root, "Trims").textContent).toContain('<img src=x onerror="alert(1)">');
    expect(section(root, "Flats").querySelector("img")?.getAttribute("onload")).toBeNull();
  });
});

describe("flatImageSrc", () => {
  it("encodes a stored flat as a data URL with its MIME type", () => {
    const bytes = new Uint8Array([104, 105]).buffer; // "hi"
    expect(flatImageSrc("aa.png", bytes)).toBe("data:image/png;base64,aGk=");
  });
});
