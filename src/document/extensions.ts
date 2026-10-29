import { Extension, getSchema, type AnyExtension } from "@tiptap/core";
import Code from "@tiptap/extension-code";
import StarterKit from "@tiptap/starter-kit";
import { Passage } from "./passage";

/**
 * Lists remember if they are tight (no blank line between items), so a
 * loose list in a file stays loose after a save.
 */
const ListTightness = Extension.create({
  name: "listTightness",
  addGlobalAttributes() {
    return [
      {
        types: ["bulletList", "orderedList"],
        attributes: {
          tight: {
            default: true,
            rendered: false,
          },
        },
      },
    ];
  },
});

/**
 * The editor extensions of a document (Essay or Script). Every node and mark
 * here has a Markdown form in `markdown.ts`. Add a node or mark in both
 * places, or the editor can make content that does not save.
 */
export const documentExtensions: AnyExtension[] = [
  StarterKit.configure({
    // No Markdown form for underline.
    underline: false,
    // It adds an empty paragraph after a final heading or list, which would
    // change the file on every open.
    trailingNode: false,
    link: { openOnClick: false, autolink: true },
    code: false,
  }),
  // Inline code excludes the other inline marks, but not a Passage, so a
  // Passage can cover code.
  Code.extend({ excludes: "bold italic strike link" }),
  ListTightness,
  Passage,
];

/** The schema of a document, for use without an editor (for example tests). */
export const documentSchema = getSchema(documentExtensions);
