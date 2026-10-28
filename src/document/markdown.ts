import type { Node as PMNode, Schema } from "@tiptap/pm/model";
import MarkdownIt from "markdown-it";
import type Token from "markdown-it/lib/token.mjs";
import {
  MarkdownParser,
  MarkdownSerializer,
  defaultMarkdownSerializer,
  type MarkdownSerializerState,
} from "prosemirror-markdown";
import { documentSchema } from "./extensions";

/*
 * Markdown <-> editor document.
 *
 * The file format is CommonMark plus ~~strikethrough~~. Raw HTML is read as
 * text. Markdown that the editor wrote reloads byte for byte. Other Markdown
 * settles into the editor's form on the first save (for example `*` bullets
 * become `-`).
 */

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
    text: nodes.text,
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

/** True when the list that opens at `tokens[i]` has no blank lines. */
function listIsTight(tokens: Token[], i: number): boolean {
  while (++i < tokens.length) {
    if (tokens[i].type !== "list_item_open") return tokens[i].hidden;
  }
  return false;
}

function markdownIt() {
  return MarkdownIt("commonmark", { html: false }).enable("strikethrough").disable("image");
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
    });
    parsers.set(schema, parser);
  }
  return parser;
}

/**
 * Reads a document from Markdown. Pass the editor's schema when the result
 * goes into an editor; the default is the shared document schema.
 */
export function fromMarkdown(markdown: string, schema: Schema = documentSchema): PMNode {
  return parserFor(schema).parse(markdown);
}

/** Writes a document as Markdown. Non-empty output ends with one newline. */
export function toMarkdown(doc: PMNode): string {
  const md = serializer.serialize(doc, { tightLists: true });
  return md === "" ? "" : md + "\n";
}
