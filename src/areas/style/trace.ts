import { designAspectLabel, type DesignDecision } from "../designDecision/api";
import type { Style } from "./types";

/*
 * A Style's trace: the Style, its Design Decisions, and the Passages that
 * motivate them. The founder exports it as story copy for product pages and
 * the Documentary. Markdown and PDF (HTML) show the same content.
 */

/** The Style fields a trace shows. */
export type TraceStyle = Pick<Style, "name" | "color">;

/** The trace title, also the default file name: "Name (Color)". */
export function styleTraceTitle(style: TraceStyle): string {
  const name = style.name.trim() || "Untitled";
  const color = style.color.trim();
  return color ? `${name} (${color})` : name;
}

/** The trace as Markdown. */
export function styleTraceMarkdown(style: TraceStyle, decisions: DesignDecision[]): string {
  const blocks = [`# ${styleTraceTitle(style)}`];
  for (const d of decisions) {
    blocks.push(`## ${designAspectLabel[d.aspect]}`, plainLines(d.rationale).join("\n"));
    for (const p of d.passages) {
      const lines = [...plainLines(p.lastKnownText), "", `— ${p.documentTitle.trim() || "Untitled"}`];
      blocks.push(lines.map((l) => (l ? `> ${l}` : ">")).join("\n"));
    }
  }
  return blocks.join("\n\n") + "\n";
}

/**
 * The trace as an HTML fragment for the PDF export (`exportPdf`). All user
 * text is escaped.
 */
export function styleTraceHtml(style: TraceStyle, decisions: DesignDecision[]): string {
  let html = `<h1>${escapeHtml(styleTraceTitle(style))}</h1>\n`;
  for (const d of decisions) {
    html += `<h2>${designAspectLabel[d.aspect]}</h2>\n${paragraphs(d.rationale)}\n`;
    for (const p of d.passages) {
      const source = escapeHtml(p.documentTitle.trim() || "Untitled");
      html += `<blockquote>${paragraphs(p.lastKnownText)}<p class="trace-source">— ${source}</p></blockquote>\n`;
    }
  }
  return html;
}

/** CSS for the trace PDF, on top of the base print style. */
export const styleTraceCss = `
.print-page h2 { font-size: 14pt; margin: 20pt 0 6pt; }
.print-page blockquote { margin-top: 8pt; margin-bottom: 8pt; break-inside: avoid; }
.print-page .trace-source { color: #6b6b6b; font-size: 10pt; }
`;

/** Plain text as HTML paragraphs: a blank line splits paragraphs, a newline breaks a line. */
function paragraphs(text: string): string {
  return text
    .trim()
    .split(/(?:\r?\n\s*){2,}/)
    .map((para) => `<p>${para.split(/\r?\n/).map((l) => escapeHtml(l.trim())).join("<br>")}</p>`)
    .join("");
}

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

/**
 * User text as Markdown lines. The text stays as written (it is copy, not
 * Markdown), except that a line start that would make a heading, quote, or
 * list is escaped.
 */
function plainLines(text: string): string[] {
  return text
    .trim()
    .split(/\r?\n/)
    .map((line) => line.trimEnd().replace(/^(\s*)([#>+*=-]|\d+(?=[.)]))/, (_, space, mark) =>
      /\d/.test(mark) ? `${space}${mark}\\` : `${space}\\${mark}`,
    ));
}
