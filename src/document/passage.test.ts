// @vitest-environment jsdom
// A mounted editor: the editor plugins (undo, paste) only run in a view.
import { Editor } from "@tiptap/core";
import { closeHistory } from "@tiptap/pm/history";
import { TextSelection } from "@tiptap/pm/state";
import type { Node as PMNode } from "@tiptap/pm/model";
import { afterEach, describe, expect, it } from "vitest";
import { documentExtensions, fromMarkdown, passagesIn, toMarkdown } from ".";

let editor: Editor | undefined;
afterEach(() => editor?.destroy());

function open(markdown: string): Editor {
  editor = new Editor({
    element: document.createElement("div"),
    extensions: documentExtensions,
    content: fromMarkdown(markdown).toJSON(),
  });
  return editor;
}

/**
 * The document position just before the first occurrence of `text` within
 * one block (a block of text only, with no hard breaks).
 */
function posOf(ed: Editor, text: string): number {
  let found = -1;
  ed.state.doc.descendants((node, pos) => {
    if (found >= 0) return false;
    if (!node.isTextblock) return true;
    const i = node.textContent.indexOf(text);
    if (i >= 0) found = pos + 1 + i;
    return false;
  });
  if (found < 0) throw new Error(`no "${text}" in the document`);
  return found;
}

/** Selects `text` (its first occurrence). */
function select(ed: Editor, text: string): Editor {
  const from = posOf(ed, text);
  ed.commands.setTextSelection({ from, to: from + text.length });
  return ed;
}

/**
 * Marks `text` (or the current selection, when `text` is left out) as a
 * Passage and returns the new Passage's ID.
 */
function mark(ed: Editor, text?: string): string {
  const before = new Set(passagesIn(ed.state.doc).map((p) => p.id));
  if (text !== undefined) select(ed, text);
  expect(ed.commands.setPassage()).toBe(true);
  const added = passagesIn(ed.state.doc).filter((p) => !before.has(p.id));
  expect(added).toHaveLength(1);
  return added[0].id;
}

/** Text of each Passage, by ID. */
function texts(doc: PMNode): Record<string, string> {
  return Object.fromEntries(passagesIn(doc).map((p) => [p.id, p.text]));
}

/** Saves and reopens, as a close and reopen of the Essay does. */
function reload(ed: Editor): PMNode {
  return fromMarkdown(toMarkdown(ed.state.doc));
}

describe("marking a Passage", () => {
  it("covers the selected text and saves it with HTML-comment markers", () => {
    const ed = open("Hello brave new world.\n");

    const id = mark(ed, "brave new");

    expect(texts(ed.state.doc)).toEqual({ [id]: "brave new" });
    expect(toMarkdown(ed.state.doc)).toBe(`Hello <!--passage:${id}-->brave new<!--/passage:${id}--> world.\n`);
    expect(texts(reload(ed))).toEqual({ [id]: "brave new" });
  });
});

/** Types `text` at `pos`, as a keystroke does (`tr.insertText`). */
function type(ed: Editor, pos: number, text: string) {
  ed.commands.command(({ tr }) => {
    tr.setSelection(TextSelection.create(tr.doc, pos)).insertText(text);
    return true;
  });
}

/** Replaces `from..to` with typed `text`, as typing over a selection does. */
function typeOver(ed: Editor, from: number, to: number, text: string) {
  ed.commands.command(({ tr }) => {
    tr.setSelection(TextSelection.create(tr.doc, from, to)).insertText(text);
    return true;
  });
}

/** Ends the current undo step, as a pause in typing does. */
function pause(ed: Editor) {
  ed.commands.command(({ tr }) => {
    closeHistory(tr);
    return true;
  });
}

function remove(ed: Editor, from: number, to: number) {
  ed.commands.deleteRange({ from, to });
}

/** Checks the Passages now, and after a save and reopen. */
function expectPassages(ed: Editor, expected: Record<string, string>) {
  expect(texts(ed.state.doc)).toEqual(expected);
  expect(texts(reload(ed))).toEqual(expected);
}

describe("a Passage after edits", () => {
  const md = "One. Hello brave new world. Two.\n";

  it("keeps its text when text is typed before it", () => {
    const ed = open(md);
    const id = mark(ed, "brave new");

    type(ed, posOf(ed, "One"), "Zero. ");
    type(ed, posOf(ed, "brave"), "very ");

    expectPassages(ed, { [id]: "brave new" });
  });

  it("keeps its text when text is typed after it", () => {
    const ed = open(md);
    const id = mark(ed, "brave new");

    type(ed, posOf(ed, " world"), " and old");
    type(ed, posOf(ed, "Two.") + 4, " Three.");

    expectPassages(ed, { [id]: "brave new" });
  });

  it("grows when text is typed inside it", () => {
    const ed = open(md);
    const id = mark(ed, "brave new");

    type(ed, posOf(ed, " new"), ", bold,");

    expectPassages(ed, { [id]: "brave, bold, new" });
  });

  it("shrinks when text inside it is deleted", () => {
    const ed = open(md);
    const id = mark(ed, "brave new");

    remove(ed, posOf(ed, "ave"), posOf(ed, " new"));

    expectPassages(ed, { [id]: "br new" });
  });

  it("loses only the deleted part when a deletion crosses its start", () => {
    const ed = open(md);
    const id = mark(ed, "brave new");

    remove(ed, posOf(ed, "Hello"), posOf(ed, " new"));

    expectPassages(ed, { [id]: " new" });
  });

  it("loses only the deleted part when a deletion crosses its end", () => {
    const ed = open(md);
    const id = mark(ed, "brave new");

    remove(ed, posOf(ed, "ew world"), posOf(ed, ". Two"));

    expectPassages(ed, { [id]: "brave n" });
  });

  it("keeps its surviving text when typing replaces text across its start", () => {
    const ed = open(md);
    const id = mark(ed, "brave new");

    typeOver(ed, posOf(ed, "lo brave"), posOf(ed, "ave"), "p! Gr");

    expect(ed.state.doc.textContent).toBe("One. Help! Grave new world. Two.");
    expectPassages(ed, { [id]: "ave new" });
  });

  it("keeps its surviving text when typing replaces text across its end", () => {
    const ed = open(md);
    const id = mark(ed, "brave new");

    typeOver(ed, posOf(ed, "ew world"), posOf(ed, "orld"), "ice w");

    expect(ed.state.doc.textContent).toBe("One. Hello brave nice world. Two.");
    expectPassages(ed, { [id]: "brave n" });
  });

  it("is gone from the document when all its text is deleted, and back on undo", () => {
    const ed = open(md);
    const id = mark(ed, "brave new");
    pause(ed);

    remove(ed, posOf(ed, "Hello"), posOf(ed, " Two"));
    expectPassages(ed, {});

    expect(ed.commands.undo()).toBe(true);
    expect(ed.state.doc.textContent).toBe(md.trim());
    expectPassages(ed, { [id]: "brave new" });
  });

  it("does not grow when text is typed right at its edges", () => {
    const ed = open("brave new\n");
    const id = mark(ed, "brave new");

    type(ed, posOf(ed, "brave"), "Hello ");
    type(ed, ed.state.doc.content.size - 1, " world");

    expect(ed.state.doc.textContent).toBe("Hello brave new world");
    expectPassages(ed, { [id]: "brave new" });
  });

  it("keeps its text when its paragraph is split before it, and joined again", () => {
    const ed = open(md);
    const id = mark(ed, "brave new");

    ed.chain().setTextSelection(posOf(ed, "Hello")).splitBlock().run();
    expect(ed.state.doc.childCount).toBe(2);
    expectPassages(ed, { [id]: "brave new" });

    ed.chain().setTextSelection(posOf(ed, "Hello")).joinBackward().run();
    expectPassages(ed, { [id]: "brave new" });
  });

  it("spans two paragraphs when its paragraph is split inside it", () => {
    const ed = open(md);
    const id = mark(ed, "brave new");

    ed.chain().setTextSelection(posOf(ed, "new")).splitBlock().run();

    expect(toMarkdown(ed.state.doc)).toBe(
      `One. Hello <!--passage:${id}-->brave <!--/passage:${id}-->\n\n<!--passage:${id}-->new<!--/passage:${id}--> world. Two.\n`,
    );
    expectPassages(ed, { [id]: "brave \nnew" });
  });
});

/** Each character run with the Passage IDs on it, for an exact comparison. */
function runs(doc: PMNode): string[] {
  const out: string[] = [];
  doc.descendants((node) => {
    if (!node.isText) return;
    const ids = node.marks.filter((m) => m.type.name === "passage").map((m) => m.attrs.id).sort();
    out.push(`${node.text}|${ids.join(",")}`);
  });
  return out;
}

/** Selects from the start of `first` to the end of `last`. */
function selectSpan(ed: Editor, first: string, last: string): Editor {
  ed.commands.setTextSelection({ from: posOf(ed, first), to: posOf(ed, last) + last.length });
  return ed;
}

describe("overlapping and nested Passages", () => {
  it("keep both Passages where they overlap", () => {
    const ed = open("Hello brave new world.\n");

    const a = mark(ed, "brave new");
    const b = mark(ed, "new world");

    expect(texts(ed.state.doc)).toEqual({ [a]: "brave new", [b]: "new world" });
    expect(toMarkdown(ed.state.doc)).toBe(
      `Hello <!--passage:${a}-->brave <!--passage:${b}-->new<!--/passage:${a}--> world<!--/passage:${b}-->.\n`,
    );
  });

  it("keep a Passage nested in another", () => {
    const ed = open("It takes time. And care. And more.\n");

    const outer = mark(ed, "It takes time. And care.");
    const inner = mark(ed, "And care");

    expect(texts(ed.state.doc)).toEqual({ [outer]: "It takes time. And care.", [inner]: "And care" });
  });

  it("round-trip through Markdown without change", () => {
    const ed = open("Hello brave new world. It takes time.\n");
    mark(ed, "brave new");
    mark(ed, "new world");
    mark(ed, "Hello brave new world.");
    mark(ed, "new");
    mark(ed, "time");

    const md = toMarkdown(ed.state.doc);
    const reopened = fromMarkdown(md);

    expect(runs(reopened)).toEqual(runs(ed.state.doc));
    expect(toMarkdown(reopened)).toBe(md);
  });

  it("round-trip hand-written crossing markers without change", () => {
    const md = "<!--passage:a-->One <!--passage:b-->two<!--/passage:a--> three<!--/passage:b--> four.\n";

    const doc = fromMarkdown(md);

    expect(texts(doc)).toEqual({ a: "One two", b: "two three" });
    expect(toMarkdown(doc)).toBe(md);
  });

  it("keep each Passage through edits inside the overlap", () => {
    const ed = open("Hello brave new world.\n");
    const a = mark(ed, "brave new");
    const b = mark(ed, "new world");

    type(ed, posOf(ed, "ew"), "EW n");
    remove(ed, posOf(ed, "brave"), posOf(ed, "ave"));

    expectPassages(ed, { [a]: "ave nEW new", [b]: "nEW new world" });
  });

  it("can be removed one at a time, and the text stays", () => {
    const ed = open("Hello brave new world.\n");
    const a = mark(ed, "brave new");
    const b = mark(ed, "new world");

    expect(ed.commands.unsetPassage(a)).toBe(true);

    expect(ed.state.doc.textContent).toBe("Hello brave new world.");
    expectPassages(ed, { [b]: "new world" });
    expect(ed.commands.unsetPassage(a)).toBe(false);
  });
});

describe("a Passage over formatting and blocks", () => {
  it("covers bold, italic, struck, and linked text", () => {
    const ed = open("Some **bold** and *it* and ~~old~~ and [a link](https://example.com) end.\n");

    const id = mark(selectSpan(ed, "old and a link", "a link"));
    const whole = mark(selectSpan(ed, "Some", "end."));

    expect(texts(ed.state.doc)).toEqual({
      [whole]: "Some bold and it and old and a link end.",
      [id]: "old and a link",
    });
    const md = toMarkdown(ed.state.doc);
    expect(md).toContain("**bold**");
    expect(md).toContain("[a link](https://example.com)");
    // The marker sits outside the ~~ run, as the run starts with the Passage.
    expect(md).toContain(`and <!--passage:${id}-->~~old~~ and`);
    expect(runs(fromMarkdown(md))).toEqual(runs(ed.state.doc));
    expect(toMarkdown(fromMarkdown(md))).toBe(md);
  });

  it("starts and ends inside a bold run without splitting it", () => {
    const ed = open("A **very bold claim** here.\n");

    const id = mark(ed, "bold");

    expect(toMarkdown(ed.state.doc)).toBe(`A **very <!--passage:${id}-->bold<!--/passage:${id}--> claim** here.\n`);
    expectPassages(ed, { [id]: "bold" });
  });

  it("covers inline code, and part of it", () => {
    const ed = open("Run `npm test` now.\n");

    const all = mark(ed, "Run npm test now.");
    const part = mark(ed, "test");

    expect(texts(ed.state.doc)).toEqual({ [all]: "Run npm test now.", [part]: "test" });
    expect(toMarkdown(ed.state.doc)).toBe(
      `<!--passage:${all}-->Run \`npm \`<!--passage:${part}-->\`test\`<!--/passage:${part}--> now.<!--/passage:${all}-->\n`,
    );
    expectPassages(ed, texts(ed.state.doc));
    expect(toMarkdown(reload(ed))).toBe(toMarkdown(ed.state.doc));
  });

  it("spans paragraphs, a heading, a list, and a quote", () => {
    const ed = open("# Patience\n\nIt takes time.\n\n- one\n- two\n\n> A quote.\n\nAfter.\n");

    const id = mark(selectSpan(ed, "tience", "A quote"));

    const expected = { [id]: "tience\nIt takes time.\none\ntwo\nA quote" };
    expectPassages(ed, expected);
    const md = toMarkdown(ed.state.doc);
    expect(toMarkdown(fromMarkdown(md))).toBe(md);
    expect(md).toContain(`- <!--passage:${id}-->one<!--/passage:${id}-->\n`);
  });

  it("keeps a multi-paragraph Passage when a paragraph inside it is deleted", () => {
    const ed = open("First part.\n\nMiddle.\n\nLast part.\n");
    const id = mark(selectSpan(ed, "part.", "Last"));

    const middle = posOf(ed, "Middle");
    remove(ed, middle - 1, middle + "Middle.".length + 1);

    expectPassages(ed, { [id]: "part.\nLast" });
  });

  it("covers a hard break inside it", () => {
    const ed = open("Line one\\\nline two\n");

    ed.commands.setTextSelection({ from: posOf(ed, "one"), to: ed.state.doc.content.size - 1 });
    const id = mark(ed);

    expectPassages(ed, { [id]: "one\nline two" });
  });
});

describe("Passage markers in Markdown", () => {
  it("are HTML comments, so a CommonMark renderer shows none of them", async () => {
    const { default: MarkdownIt } = await import("markdown-it");
    const ed = open("Hello brave new world.\n");
    mark(ed, "brave new");
    mark(ed, "new world");

    const html = MarkdownIt({ html: true }).render(toMarkdown(ed.state.doc));
    const shown = html.replace(/<!--[\s\S]*?-->/g, "");

    expect(shown).toBe("<p>Hello brave new world.</p>\n");
  });

  it("keep literal marker-like text as text", () => {
    const ed = open("Plain.\n");
    type(ed, posOf(ed, "Plain.") + 6, " <!--passage:x-->not a marker<!--/passage:x-->");

    const md = toMarkdown(ed.state.doc);

    expect(passagesIn(fromMarkdown(md))).toEqual([]);
    expect(fromMarkdown(md).textContent).toBe("Plain. <!--passage:x-->not a marker<!--/passage:x-->");
    expect(toMarkdown(fromMarkdown(md))).toBe(md);
  });

  it("read a Passage that a hand-written file opens and closes in different blocks", () => {
    const doc = fromMarkdown("A <!--passage:p-->start.\n\nThe end<!--/passage:p-->. After.\n");

    expect(texts(doc)).toEqual({ p: "start.\nThe end" });
  });
});

/** Pastes clipboard HTML at the cursor. (jsdom has no ClipboardEvent.) */
function paste(ed: Editor, html: string) {
  ed.view.pasteHTML(html, new Event("paste") as ClipboardEvent);
}

describe("Passage IDs", () => {
  it("are new and different for each Passage", () => {
    const ed = open("one two three\n");

    const ids = [mark(ed, "one"), mark(ed, "two"), mark(ed, "three")];

    expect(new Set(ids).size).toBe(3);
    for (const id of ids) expect(id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
  });

  it("are not copied when a Passage is copied and pasted", () => {
    const ed = open("Hello brave new world.\n\nEnd.\n");
    const id = mark(ed, "brave new");

    ed.commands.setTextSelection(posOf(ed, "End.") + 4);
    paste(ed, `<p><span data-passage-id="${id}">brave new</span> again</p>`);

    const found = passagesIn(ed.state.doc);
    expect(found.map((p) => p.text)).toEqual(["brave new", "brave new"]);
    expect(found[0].id).toBe(id);
    expect(found[1].id).not.toBe(id);
    expect(toMarkdown(reload(ed))).toBe(toMarkdown(ed.state.doc));
  });

  it("are not taken over from another Essay by a paste", () => {
    const ed = open("Text.\n");

    ed.commands.setTextSelection(posOf(ed, "Text.") + 5);
    paste(ed, `<p><span data-passage-id="from-other-essay">pasted</span> <span data-passage-id="from-other-essay">twice</span></p>`);

    const found = passagesIn(ed.state.doc);
    expect(found).toHaveLength(1);
    expect(found[0].id).not.toBe("from-other-essay");
    expect(found[0].text).toBe("pastedtwice");
  });

  it("move with the text when a Passage is cut and pasted in the same Essay", () => {
    const ed = open("Hello brave new world.\n\nEnd.\n");
    const id = mark(ed, "brave new");

    select(ed, "brave new");
    ed.view.dom.dispatchEvent(new Event("cut", { bubbles: true }));
    expect(passagesIn(ed.state.doc)).toEqual([]);
    ed.commands.setTextSelection(posOf(ed, "End.") + 4);
    paste(ed, `<p><span data-passage-id="${id}">brave new</span></p>`);

    expectPassages(ed, { [id]: "brave new" });
  });
});
