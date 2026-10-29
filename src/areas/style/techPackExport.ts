import { mimeTypeOf } from "../moodboard/mediaType";
import type { TechPack } from "./techPack";

/*
 * A Tech Pack as an HTML fragment for the shared PDF export (`exportPdf`).
 * Pure: the caller reads the flats and names the fabric Vendor first.
 */

export interface TechPackExportInput {
  styleName: string;
  styleColor: string;
  techPack: TechPack;
  /** The fabric Vendor's name, or null when the fabric has no Vendor. */
  fabricVendorName: string | null;
  /** Image source per flat ID (see `flatImageSrc`). A flat with no entry shows as "Image not available". */
  flatImages: Record<number, string>;
}

const NOT_SPECIFIED = '<p class="tp-none">Not specified</p>';

function escape(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

/** Escaped text with its line breaks kept. */
function multiline(text: string): string {
  return escape(text).replace(/\r?\n/g, "<br>");
}

function section(heading: string, body: string): string {
  return `<section class="tp-section"><h2>${escape(heading)}</h2>${body}</section>`;
}

function textSection(heading: string, text: string): string {
  return section(heading, text.trim() ? `<p>${multiline(text.trim())}</p>` : NOT_SPECIFIED);
}

function measureTable(tp: TechPack): string {
  if (tp.pointsOfMeasure.length === 0) return NOT_SPECIFIED;
  const head = ["Point of Measure", "Tolerance ±", ...tp.sizes.map((s) => s.name)]
    .map((h) => `<th>${escape(h)}</th>`)
    .join("");
  const rows = tp.pointsOfMeasure
    .map((pom) => {
      const cells = [pom.tolerance, ...pom.values].map((v) => `<td class="tp-num">${v === null ? "" : v}</td>`);
      return `<tr><td>${escape(pom.name)}</td>${cells.join("")}</tr>`;
    })
    .join("");
  return `<table class="tp-pom"><thead><tr>${head}</tr></thead><tbody>${rows}</tbody></table>`;
}

function fabric(tp: TechPack, vendorName: string | null): string {
  const { composition, weight, color } = tp.details.fabric;
  const rows = (
    [
      ["Composition", composition],
      ["Weight", weight],
      ["Color", color],
      ["Vendor", vendorName ?? ""],
    ] as const
  ).map(([label, value]) => `<tr><th>${label}</th><td>${value.trim() ? escape(value.trim()) : "Not specified"}</td></tr>`);
  return `<table class="tp-fabric"><tbody>${rows.join("")}</tbody></table>`;
}

function flats(tp: TechPack, images: Record<number, string>): string {
  if (tp.flats.length === 0) return NOT_SPECIFIED;
  const figures = tp.flats.map((f) => {
    const src = images[f.id];
    const picture = src
      ? `<img src="${escape(src)}" alt="${escape(f.fileName)}">`
      : `<p class="tp-none">Image not available</p>`;
    return `<figure class="tp-flat">${picture}<figcaption>${escape(f.fileName)}</figcaption></figure>`;
  });
  return `<div class="tp-flats">${figures.join("")}</div>`;
}

/** The whole Tech Pack: every part a factory needs, in a fixed order. Escapes all user text. */
export function techPackExportHtml(input: TechPackExportInput): string {
  const { techPack: tp } = input;
  return [
    `<h1>${escape(input.styleName.trim() || "Untitled")} — Tech Pack</h1>`,
    input.styleColor.trim() ? `<p class="tp-meta">Color: ${escape(input.styleColor.trim())}</p>` : "",
    section(`Points of Measure (${tp.details.unit})`, measureTable(tp)),
    section("Fabric", fabric(tp, input.fabricVendorName)),
    textSection("Trims", tp.details.trims),
    textSection("Construction notes", tp.details.constructionNotes),
    textSection("Label placement", tp.details.labelPlacement),
    section("Flats", flats(tp, input.flatImages)),
  ].join("");
}

/** Print CSS for `techPackExportHtml`, on top of the base print style. */
export const techPackExportCss = `
.print-page { font-family: -apple-system, "Helvetica Neue", Arial, sans-serif; font-size: 10.5pt; }
.print-page .tp-meta { color: #444; margin: -12pt 0 12pt; }
.print-page .tp-section { margin-bottom: 16pt; }
.print-page .tp-section h2 { font-size: 13pt; border-bottom: 1px solid #999; padding-bottom: 2pt; margin: 0 0 6pt; }
.print-page .tp-none { color: #777; font-style: italic; }
.print-page .tp-num { text-align: right; font-variant-numeric: tabular-nums; }
.print-page .tp-pom th { background: #f0f0ee; }
.print-page .tp-fabric th { width: 30%; background: #f0f0ee; }
.print-page .tp-flats { display: flex; flex-wrap: wrap; gap: 12pt; }
.print-page .tp-flat { margin: 0; flex: 1 1 45%; break-inside: avoid; text-align: center; }
.print-page .tp-flat img { max-height: 110mm; object-fit: contain; border: 1px solid #ddd; }
.print-page .tp-flat figcaption { font-size: 9pt; color: #444; margin-top: 3pt; }
`;

/**
 * A stored flat as a data URL. The print window is a separate window, so an
 * object URL from this window may not load there; a data URL always does.
 */
export function flatImageSrc(media: string, bytes: ArrayBuffer): string {
  const view = new Uint8Array(bytes);
  let binary = "";
  for (let i = 0; i < view.length; i += 0x8000) {
    binary += String.fromCharCode(...view.subarray(i, i + 0x8000));
  }
  return `data:${mimeTypeOf(media)};base64,${btoa(binary)}`;
}
