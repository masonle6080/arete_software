import { Editor } from "@tiptap/core";
import { afterEach, describe, expect, it } from "vitest";
import { documentExtensions, fromMarkdown, toMarkdown } from ".";

let editor: Editor | undefined;
afterEach(() => editor?.destroy());

function open(markdown: string): Editor {
  // JSON content needs no DOM, so these tests run in the node environment.
  editor = new Editor({
    element: null,
    extensions: documentExtensions,
    content: fromMarkdown(markdown).toJSON(),
  });
  return editor;
}

/** The cursor position at the end of the last block's text. */
function endOfText(ed: Editor): number {
  return ed.state.doc.content.size - 1;
}

describe("a document in the editor", () => {
  it("saves unchanged when opened and not edited", () => {
    const md = "# Patience\n\nIt takes time.\n\n- one\n- two\n";

    expect(toMarkdown(open(md).state.doc)).toBe(md);
  });

  it("saves typed text as Markdown", () => {
    const ed = open("It takes time.\n");

    ed.chain()
      .setTextSelection(endOfText(ed))
      .insertContent([
        { type: "text", text: " And " },
        { type: "text", text: "care", marks: [{ type: "bold" }] },
        { type: "text", text: "." },
      ])
      .run();
    ed.chain().setTextSelection(endOfText(ed)).splitBlock().insertContent({ type: "text", text: "New paragraph." }).run();

    expect(toMarkdown(ed.state.doc)).toBe("It takes time. And **care**.\n\nNew paragraph.\n");
  });

  it("saves an empty document as an empty file", () => {
    expect(toMarkdown(open("").state.doc)).toBe("");
  });
});
