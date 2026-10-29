import { Mark, type CommandProps } from "@tiptap/core";
import { Fragment, Slice, type Node as PMNode } from "@tiptap/pm/model";
import { Plugin, PluginKey } from "@tiptap/pm/state";

/*
 * A Passage is a mark that carries its ID (ADR-0001). The text owns the
 * Passage boundaries: ProseMirror moves the mark with every edit, so there
 * are no offsets to keep up to date.
 *
 * - Marks with different IDs can overlap and nest (`excludes: ""`).
 * - Typing at either edge does not extend a Passage (`inclusive: false`).
 * - A Passage can span several blocks. It is one mark ID on several runs.
 */

export const PASSAGE = "passage";

/** The characters of a Passage ID. The Markdown markers depend on it. */
export const PASSAGE_ID = /^[A-Za-z0-9_-]+$/;

/**
 * A new Passage ID, unique across the library: a random UUID (v4).
 * The core stores it as is.
 */
export function newPassageId(): string {
  const b = crypto.getRandomValues(new Uint8Array(16));
  b[6] = (b[6] & 0x0f) | 0x40;
  b[8] = (b[8] & 0x3f) | 0x80;
  const h = Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
  return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`;
}

/** A Passage present in a document, and its current text. */
export interface PassageText {
  id: string;
  /** The covered text. Text from different blocks is joined with "\n". */
  text: string;
}

/**
 * The Passages present in a document, in order of their first character.
 * A Passage whose text was fully deleted is not in the list.
 */
export function passagesIn(doc: PMNode): PassageText[] {
  const found = new Map<string, { parts: string[]; block: PMNode | null }>();
  doc.descendants((node, _pos, parent) => {
    if (!node.isInline) return;
    for (const m of node.marks) {
      if (m.type.name !== PASSAGE) continue;
      const id: string = m.attrs.id;
      let p = found.get(id);
      if (!p) found.set(id, (p = { parts: [], block: parent }));
      else if (p.block !== parent) p.parts.push("\n");
      p.block = parent;
      p.parts.push(node.isText ? node.text! : node.type.spec.leafText?.(node) ?? "\n");
    }
  });
  return Array.from(found, ([id, p]) => ({ id, text: p.parts.join("") }));
}

/** The IDs of the Passages that cover position `pos`, innermost first. */
export function passageIdsAt(doc: PMNode, pos: number): string[] {
  const $pos = doc.resolve(pos);
  const node = $pos.nodeAfter ?? $pos.nodeBefore;
  const ids = (node?.marks ?? []).filter((m) => m.type.name === PASSAGE).map((m) => m.attrs.id as string);
  const size = new Map(passagesIn(doc).map((p) => [p.id, p.text.length]));
  return ids.sort((a, b) => (size.get(a) ?? 0) - (size.get(b) ?? 0));
}

/**
 * The document range a Passage covers, from its first character to its
 * last, or `null` when the Passage is not in the document. Use it to land
 * on a Passage (select it and scroll to it).
 */
export function passageRange(doc: PMNode, id: string): { from: number; to: number } | null {
  let from = -1;
  let to = -1;
  doc.descendants((node, pos) => {
    if (!node.isInline) return;
    if (!node.marks.some((m) => m.type.name === PASSAGE && m.attrs.id === id)) return;
    if (from < 0) from = pos;
    to = pos + node.nodeSize;
  });
  return from < 0 ? null : { from, to };
}

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    passage: {
      /**
       * Marks the selected text as a new Passage. Pass `id` only to restore
       * a known Passage; leave it out to get a new unique ID.
       */
      setPassage: (attrs?: { id?: string }) => ReturnType;
      /** Removes one Passage mark everywhere in the document. Keeps the text. */
      unsetPassage: (id: string) => ReturnType;
    };
  }
}

/** The Passage IDs in a fragment of a document. */
function idsIn(fragment: Fragment): Set<string> {
  const ids = new Set<string>();
  fragment.descendants((node) => {
    for (const m of node.marks) if (m.type.name === PASSAGE) ids.add(m.attrs.id);
  });
  return ids;
}

/** `slice` with each Passage ID replaced by `rename(id)`. */
function renamePassages(slice: Slice, rename: (id: string) => string): Slice {
  const map = (fragment: Fragment): Fragment => {
    const children: PMNode[] = [];
    fragment.forEach((node) => {
      const marks = node.marks.map((m) =>
        m.type.name === PASSAGE ? m.type.create({ ...m.attrs, id: rename(m.attrs.id) }) : m,
      );
      const content = map(node.content);
      children.push(node.isText ? node.mark(marks) : node.copy(content).mark(marks));
    });
    return Fragment.from(children);
  };
  return new Slice(map(slice.content), slice.openStart, slice.openEnd);
}

/**
 * Keeps Passage IDs unique across the library when text is pasted.
 * A pasted Passage gets a new ID, so a copy never shares the ID of the
 * Passage it came from (in this Essay or another one). Text that is moved
 * within one Essay keeps its IDs: a drag, or a cut and paste while the ID
 * is not in the document.
 */
function passagePastePlugin(): Plugin {
  let cut = new Set<string>();
  return new Plugin({
    key: new PluginKey("passagePaste"),
    props: {
      handleDOMEvents: {
        cut: (view) => {
          cut = idsIn(view.state.selection.content().content);
          return false;
        },
      },
      transformPasted: (slice, view) => {
        if (view.dragging?.move) return slice;
        const present = idsIn(view.state.doc.content);
        const renamed = new Map<string, string>();
        return renamePassages(slice, (id) => {
          if (cut.has(id) && !present.has(id)) return id;
          if (!renamed.has(id)) renamed.set(id, newPassageId());
          return renamed.get(id)!;
        });
      },
    },
  });
}

export const Passage = Mark.create({
  name: PASSAGE,
  excludes: "",
  inclusive: false,
  // Last in rank, so a Passage mark wraps every other mark.
  priority: 1,

  addAttributes() {
    return {
      id: {
        default: null,
        parseHTML: (el) => el.getAttribute("data-passage-id"),
        renderHTML: (attrs) => ({ "data-passage-id": attrs.id }),
      },
    };
  },

  parseHTML() {
    return [{ tag: "span[data-passage-id]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["span", { ...HTMLAttributes, class: "passage" }, 0];
  },

  addKeyboardShortcuts() {
    return { "Mod-Shift-p": () => this.editor.commands.setPassage() };
  },

  addProseMirrorPlugins() {
    return [passagePastePlugin()];
  },

  addCommands() {
    return {
      setPassage:
        (attrs = {}) =>
        ({ state, tr, dispatch }: CommandProps) => {
          const { from, to, empty } = state.selection;
          if (empty) return false;
          const id = attrs.id ?? newPassageId();
          if (!PASSAGE_ID.test(id)) return false;
          const mark = state.schema.marks[PASSAGE].create({ id });
          tr.addMark(from, to, mark);
          if (!tr.docChanged) return false;
          dispatch?.(tr);
          return true;
        },
      unsetPassage:
        (id: string) =>
        ({ state, tr, dispatch }: CommandProps) => {
          const mark = state.schema.marks[PASSAGE].create({ id });
          tr.removeMark(0, state.doc.content.size, mark);
          if (!tr.docChanged) return false;
          dispatch?.(tr);
          return true;
        },
    };
  },
});
