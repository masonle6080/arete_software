import { Schema, type Mark, type Node as PMNode } from "@tiptap/pm/model";
import MarkdownIt from "markdown-it";
import type Token from "markdown-it/lib/token.mjs";
import {
  MarkdownParser,
  MarkdownSerializer,
  defaultMarkdownSerializer,
  type MarkdownSerializerState,
} from "prosemirror-markdown";
import { documentSchema } from "./extensions";
import { PASSAGE } from "./passage";

/*
 * Markdown <-> editor document.
 *
 * The file format is CommonMark plus ~~strikethrough~~. Raw HTML is read as
 * text. Markdown that the editor wrote reloads byte for byte. Other Markdown
 * settles into the editor's form on the first save (for example `*` bullets
 * become `-`).
 *
 * A Passage is a pair of HTML-comment markers around its text:
 *
 *   Hello <!--passage:ID-->brave new<!--/passage:ID--> world.
 *
 * Other Markdown tools ignore the comments. Markers of different Passages
 * can cross. A Passage that spans blocks is closed at the end of each block
 * and opened again, with the same ID, at the start of the next one.
 */

/** Matches one marker at the start of the string. */
const MARKER = /^<!--(\/?)passage:([A-Za-z0-9_-]+)-->/;
/** The "<" of text that would read as a marker. It is written escaped. */
const MARKER_LIKE = /<(?=!--\/?passage:[A-Za-z0-9_-]+-->)/;

const nodes = defaultMarkdownSerializer.nodes;
const marks = defaultMarkdownSerializer.marks;

const serializer = new MarkdownSerializer(
  {
    doc: (state, node) => state.renderContent(node),
    paragraph: nodes.paragraph,
    heading: nodes.heading,
    blockquote: nodes.blockquote,
    horizontalRule: nodes.horizontal_rule,
    hardBreak: nodes.hard_break,
    text: (state, node) => {
      node.text!.split(MARKER_LIKE).forEach((part, i) => {
        if (i > 0) state.write("\\<");
        state.text(part, !(state as unknown as { inAutolink?: boolean }).inAutolink);
      });
    },
    passageMarker: (state, node) => {
      state.write(`<!--${node.attrs.close ? "/" : ""}passage:${node.attrs.id}-->`);
    },
    listItem: nodes.list_item,
    bulletList: (state, node) => state.renderList(node, "  ", () => "- "),
    orderedList: (state, node) => {
      const start: number = node.attrs.start ?? 1;
      const width = String(start + node.childCount - 1).length;
      state.renderList(node, " ".repeat(width + 2), (i) => {
        const n = String(start + i);
        return " ".repeat(width - n.length) + n + ". ";
      });
    },
    codeBlock: (state: MarkdownSerializerState, node: PMNode) => {
      const runs = node.textContent.match(/`{3,}/gm);
      const fence = runs ? runs.sort().slice(-1)[0] + "`" : "```";
      state.write(fence + (node.attrs.language ?? "") + "\n");
      state.text(node.textContent, false);
      state.write("\n");
      state.write(fence);
      state.closeBlock(node);
    },
  },
  {
    italic: marks.em,
    bold: marks.strong,
    strike: { open: "~~", close: "~~", mixable: true, expelEnclosingWhitespace: true },
    code: marks.code,
    link: marks.link,
  },
);

/**
 * The schema the serializer works on: the document schema with the Passage
 * mark replaced by marker nodes. The serializer never sees a Passage mark,
 * so Passages do not change how the other marks are written.
 */
const markerSchema = new Schema({
  nodes: documentSchema.spec.nodes.append({
    passageMarker: {
      inline: true,
      group: "inline",
      atom: true,
      attrs: { id: {}, close: { default: false } },
    },
  }),
  marks: documentSchema.spec.marks.remove(PASSAGE),
});

interface JSONMark {
  type: string;
  attrs?: Record<string, unknown>;
}
interface JSONNode {
  type: string;
  attrs?: Record<string, unknown>;
  content?: JSONNode[];
  marks?: JSONMark[];
}

/** The Passage IDs on `node`, sorted, so marker order is stable. */
function passageIds(node: PMNode | null): string[] {
  return (node?.marks ?? [])
    .filter((m) => m.type.name === PASSAGE)
    .map((m) => m.attrs.id as string)
    .sort();
}

function marksWithout(node: PMNode | null, names: string[]): Mark[] {
  return (node?.marks ?? []).filter((m) => !names.includes(m.type.name));
}

function marker(id: string, close: boolean, around: Mark[]): JSONNode {
  const json: JSONNode = { type: "passageMarker", attrs: { id, close } };
  if (around.length) json.marks = around.map((m) => m.toJSON());
  return json;
}

/**
 * One textblock's inline content, with a marker at each Passage edge.
 * With `markers` false, Passages are dropped and only their text stays.
 */
function withMarkers(block: PMNode, markers: boolean): JSONNode[] {
  const out: JSONNode[] = [];
  const children = [...block.children, null];
  let prev: PMNode | null = null;
  for (const child of children) {
    const before = passageIds(prev);
    const after = passageIds(child);
    const closed = before.filter((id) => !after.includes(id)).reverse();
    const opened = after.filter((id) => !before.includes(id));
    if (markers && (closed.length || opened.length)) {
      // A marker takes the marks on both sides, so it does not split a bold
      // run. Inline code cannot hold a marker, so a code span splits instead.
      const next = marksWithout(child, [PASSAGE, "code"]);
      const around = marksWithout(prev, [PASSAGE, "code"]).filter((m) => m.isInSet(next));
      for (const id of closed) out.push(marker(id, true, around));
      for (const id of opened) out.push(marker(id, false, around));
    }
    if (child) {
      const json = child.toJSON() as JSONNode;
      const kept = marksWithout(child, [PASSAGE]);
      if (kept.length) json.marks = kept.map((m) => m.toJSON());
      else delete json.marks;
      out.push(json);
    }
    prev = child;
  }
  return out;
}

/** The document in the marker schema. */
function toMarkerDoc(doc: PMNode, markers: boolean): PMNode {
  const convert = (node: PMNode): JSONNode => {
    const json = node.toJSON() as JSONNode;
    if (node.isTextblock) {
      if (node.childCount) json.content = withMarkers(node, markers);
    } else if (node.childCount) {
      json.content = node.children.map(convert);
    }
    return json;
  };
  return markerSchema.nodeFromJSON(convert(doc));
}

/** True when the list that opens at `tokens[i]` has no blank lines. */
function listIsTight(tokens: Token[], i: number): boolean {
  while (++i < tokens.length) {
    if (tokens[i].type !== "list_item_open") return tokens[i].hidden;
  }
  return false;
}

/** Reads a Passage marker as a `passage_marker` token with `meta: { id, close }`. */
function passageMarkers(md: ReturnType<typeof MarkdownIt>) {
  md.inline.ruler.before("html_inline", "passage_marker", (state, silent) => {
    const m = MARKER.exec(state.src.slice(state.pos));
    if (!m) return false;
    if (!silent) state.push("passage_marker", "", 0).meta = { close: m[1] === "/", id: m[2] };
    state.pos += m[0].length;
    return true;
  });
}

/** The Markdown reader for the file format. `export.ts` renders HTML with it. */
export function markdownIt() {
  return MarkdownIt("commonmark", { html: false })
    .enable("strikethrough")
    .disable("image")
    .use(passageMarkers);
}

/** The parse state methods the marker handler uses (prosemirror-markdown internals). */
interface ParseState {
  openMark(mark: Mark): void;
  closeMark(mark: Mark): void;
}

const parsers = new WeakMap<Schema, MarkdownParser>();

function parserFor(schema: Schema): MarkdownParser {
  let parser = parsers.get(schema);
  if (!parser) {
    parser = new MarkdownParser(schema, markdownIt(), {
      paragraph: { block: "paragraph" },
      heading: { block: "heading", getAttrs: (tok) => ({ level: +tok.tag.slice(1) }) },
      blockquote: { block: "blockquote" },
      bullet_list: {
        block: "bulletList",
        getAttrs: (_tok, tokens, i) => ({ tight: listIsTight(tokens, i) }),
      },
      ordered_list: {
        block: "orderedList",
        getAttrs: (tok, tokens, i) => ({
          start: +(tok.attrGet("start") ?? 1) || 1,
          tight: listIsTight(tokens, i),
        }),
      },
      list_item: { block: "listItem" },
      code_block: { block: "codeBlock", noCloseToken: true },
      fence: {
        block: "codeBlock",
        getAttrs: (tok) => ({ language: tok.info.trim() || null }),
        noCloseToken: true,
      },
      hr: { node: "horizontalRule" },
      hardbreak: { node: "hardBreak" },
      em: { mark: "italic" },
      strong: { mark: "bold" },
      s: { mark: "strike" },
      code_inline: { mark: "code", noCloseToken: true },
      link: {
        mark: "link",
        getAttrs: (tok) => ({ href: tok.attrGet("href"), title: tok.attrGet("title") || null }),
      },
      passage_marker: { ignore: true, noCloseToken: true },
    });
    // A mark spec closes marks by type, which would close every Passage at
    // once. Close by ID instead. `tokenHandlers` is internal to
    // prosemirror-markdown: the version is pinned, and the Passage tests
    // guard this.
    const handlers = (parser as unknown as { tokenHandlers: Record<string, TokenHandler> }).tokenHandlers;
    handlers.passage_marker = (state, tok) => {
      const mark = schema.marks[PASSAGE].create({ id: tok.meta.id });
      if (tok.meta.close) {
        openPassages.delete(tok.meta.id);
        state.closeMark(mark);
      } else {
        openPassages.set(tok.meta.id, mark);
        state.openMark(mark);
      }
    };
    // A hand-written file can open a Passage in one block and close it in a
    // later one. Each new block of text then opens the Passage again.
    for (const name of ["paragraph_open", "heading_open"]) {
      const open = handlers[name];
      handlers[name] = (state, tok, tokens, i) => {
        open(state, tok, tokens, i);
        for (const mark of openPassages.values()) state.openMark(mark);
      };
    }
    parsers.set(schema, parser);
  }
  return parser;
}

type TokenHandler = (state: ParseState, tok: Token, tokens: Token[], i: number) => void;

/** The Passages open at the current point of the running parse, by ID. */
const openPassages = new Map<string, Mark>();

/**
 * Reads a document from Markdown. Pass the editor's schema when the result
 * goes into an editor; the default is the shared document schema.
 */
export function fromMarkdown(markdown: string, schema: Schema = documentSchema): PMNode {
  openPassages.clear();
  return parserFor(schema).parse(markdown);
}

/**
 * Writes a document as Markdown. Non-empty output ends with one newline.
 * With `passages: false` it writes no Passage markers (for exports).
 */
export function toMarkdown(doc: PMNode, { passages = true }: { passages?: boolean } = {}): string {
  const md = serializer.serialize(toMarkerDoc(doc, passages), { tightLists: true });
  return md === "" ? "" : md + "\n";
}
