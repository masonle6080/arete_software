import type { Node as PMNode } from "@tiptap/pm/model";
import { toMarkdown } from "./markdown";
import { passagesIn, type PassageText } from "./passage";

/**
 * What the editor sends to the core on every save: the Markdown, and every
 * Passage present in it with its current text. The core updates each
 * Passage's last known text, orphans a known Passage that is missing, and
 * restores an orphaned one that is back (spec #1).
 */
export interface SaveReport {
  markdown: string;
  passages: PassageText[];
}

/** The save report of a document. A Passage with no text left is not in it. */
export function saveReport(doc: PMNode): SaveReport {
  return { markdown: toMarkdown(doc), passages: passagesIn(doc) };
}
