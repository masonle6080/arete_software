/**
 * The document module: Markdown <-> editor document for Essays and Scripts.
 *
 * - `documentExtensions`: give these to the TipTap editor.
 * - `fromMarkdown(md, schema?)`: file text -> editor document.
 * - `toMarkdown(doc)`: editor document -> file text.
 *
 * Passage marks, their HTML-comment markers, and the save report come later
 * (see spec #1). They belong in this module too.
 */
export { documentExtensions, documentSchema } from "./extensions";
export { fromMarkdown, toMarkdown } from "./markdown";
