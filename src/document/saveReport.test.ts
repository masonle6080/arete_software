// @vitest-environment jsdom
// A mounted editor, so undo runs as it does in the app.
import { Editor } from "@tiptap/core";
import { afterEach, describe, expect, it } from "vitest";
import { documentExtensions, fromMarkdown, saveReport } from ".";

let editor: Editor | undefined;
afterEach(() => editor?.destroy());

const MD =
  "Hello <!--passage:p-brave-->brave new<!--/passage:p-brave--> world.\n\n" +
  "<!--passage:p-end-->The end.<!--/passage:p-end-->\n";

function open(markdown: string): Editor {
  editor = new Editor({
    element: document.createElement("div"),
    extensions: documentExtensions,
    content: fromMarkdown(markdown).toJSON(),
  });
  return editor;
}

/** Selects the first occurrence of `text` (within one block). */
function select(ed: Editor, text: string): void {
  let from = -1;
  ed.state.doc.descendants((node, pos) => {
    if (from >= 0 || !node.isTextblock) return from < 0;
    const i = node.textContent.indexOf(text);
    if (i >= 0) from = pos + 1 + i;
    return false;
  });
  ed.commands.setTextSelection({ from, to: from + text.length });
}

describe("the save report", () => {
  it("holds the Markdown and every Passage present with its text", () => {
    const report = saveReport(open(MD).state.doc);

    expect(report.markdown).toBe(MD);
    expect(report.passages).toEqual([
      { id: "p-brave", text: "brave new" },
      { id: "p-end", text: "The end." },
    ]);
  });

  it("drops a Passage whose full text was deleted", () => {
    const ed = open(MD);

    select(ed, "brave new");
    ed.commands.deleteSelection();

    const report = saveReport(ed.state.doc);
    expect(report.passages).toEqual([{ id: "p-end", text: "The end." }]);
    expect(report.markdown).not.toContain("p-brave");
  });

  it("keeps a Passage while part of its text is left", () => {
    const ed = open(MD);

    select(ed, "brave ");
    ed.commands.deleteSelection();

    expect(saveReport(ed.state.doc).passages[0]).toEqual({ id: "p-brave", text: "new" });
  });

  it("has a deleted Passage again after undo", () => {
    const ed = open(MD);
    select(ed, "brave new");
    ed.commands.deleteSelection();

    ed.commands.undo();

    expect(saveReport(ed.state.doc).passages.map((p) => p.id)).toEqual(["p-brave", "p-end"]);
  });
});
