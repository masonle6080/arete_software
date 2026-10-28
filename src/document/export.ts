import { documentSchema } from "./extensions";
import { fromMarkdown, markdownIt, toMarkdown } from "./markdown";

/*
 * Exports of a document (Essay or Script) for use outside the app.
 * An export has no Passage markers: Passages are an Arete concept.
 * The title becomes the first heading.
 */

/** The document as plain Markdown: the title as a heading, no Passage markers. */
export function exportMarkdown(title: string, markdown: string): string {
  const doc = fromMarkdown(markdown);
  const blocks = title.trim()
    ? [documentSchema.node("heading", { level: 1 }, documentSchema.text(title.trim())), ...doc.children]
    : doc.children;
  return toMarkdown(documentSchema.node("doc", null, blocks), { passages: false });
}

/**
 * The document as an HTML fragment (no `<html>` wrapper), for a PDF export.
 * Raw HTML in the text is escaped, so the fragment is safe to insert.
 */
export function exportHtml(title: string, markdown: string): string {
  return markdownIt().render(exportMarkdown(title, markdown));
}
