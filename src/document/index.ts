/**
 * The document module: Markdown <-> editor document for Essays and Scripts.
 *
 * - `documentExtensions`: give these to the TipTap editor.
 * - `fromMarkdown(md, schema?)`: file text -> editor document.
 * - `toMarkdown(doc)`: editor document -> file text.
 *
 * - `passagesIn(doc)`: the Passages present in a document and their text.
 *   The save report (spec #1) builds on it.
 * - Editor commands `setPassage()` and `unsetPassage(id)`.
 *
 * A Passage is a mark carrying its ID in the editor, and a pair of
 * HTML-comment markers in Markdown (see `markdown.ts` and `passage.ts`).
 */
export { documentExtensions, documentSchema } from "./extensions";
export { fromMarkdown, toMarkdown } from "./markdown";
export { newPassageId, passageIdsAt, passagesIn, type PassageText } from "./passage";
