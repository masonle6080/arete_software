import { EditorContent, useEditor, useEditorState, type Editor } from "@tiptap/react";
import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from "react";
import { documentExtensions } from "./extensions";
import { fromMarkdown } from "./markdown";
import { passageIdsAt, type PassageText } from "./passage";
import { saveReport } from "./saveReport";
import "./editor.css";

export interface DocumentEditorProps {
  title: string;
  markdown: string;
  /**
   * Called after edits pause, and once more on close if edits are unsaved.
   * `passages` is the save report: every Passage present and its text.
   */
  onSave(edit: DocumentSave): Promise<void>;
  /** When set, the Passage bar offers to open the Passage at the cursor. */
  onOpenPassage?(id: string): void;
}

/** What one save sends: the title and the save report. */
export interface DocumentSave {
  title: string;
  markdown: string;
  passages: PassageText[];
}

type SaveStatus = "saved" | "unsaved" | "saving" | { error: string };

const SAVE_DELAY_MS = 600;

export interface DocumentEditorHandle {
  /** Saves unsaved edits now. Resolves when the save is done. */
  flush(): Promise<void>;
}

/**
 * A distraction-free editor for one document (Essay or Script). It loads
 * Markdown and saves Markdown. Remount it (with `key`) to open another one.
 */
export const DocumentEditor = forwardRef<DocumentEditorHandle, DocumentEditorProps>(function DocumentEditor(
  { title: initialTitle, markdown, onSave, onOpenPassage },
  ref,
) {
  const [title, setTitle] = useState(initialTitle);
  const [status, setStatus] = useState<SaveStatus>("saved");
  const lastSaved = useRef<{ title: string; markdown: string }>({ title: initialTitle, markdown });
  const pending = useRef<DocumentSave | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const titleRef = useRef(initialTitle);
  const inflight = useRef<Promise<void>>(Promise.resolve());
  const onSaveRef = useRef(onSave);
  onSaveRef.current = onSave;

  const flush = (): Promise<void> => {
    clearTimeout(timer.current);
    const edit = pending.current;
    if (!edit) return inflight.current;
    pending.current = null;
    if (edit.title === lastSaved.current.title && edit.markdown === lastSaved.current.markdown) {
      setStatus("saved");
      return inflight.current;
    }
    setStatus("saving");
    // Saves run one after another, so an older save never lands last.
    inflight.current = inflight.current
      .then(() => onSaveRef.current(edit))
      .then(
        () => {
          lastSaved.current = edit;
          setStatus(pending.current ? "unsaved" : "saved");
        },
        (err: { message?: string }) => setStatus({ error: err?.message ?? String(err) }),
      );
    return inflight.current;
  };

  useImperativeHandle(ref, () => ({ flush }));

  const editor = useEditor({
    extensions: documentExtensions,
    content: fromMarkdown(markdown).toJSON(),
    immediatelyRender: true,
    autofocus: "end",
    onUpdate: ({ editor }) => schedule({ title: titleRef.current, ...saveReport(editor.state.doc) }),
  });

  const schedule = (edit: DocumentSave) => {
    pending.current = edit;
    setStatus("unsaved");
    clearTimeout(timer.current);
    timer.current = setTimeout(flush, SAVE_DELAY_MS);
  };

  // Save unsaved edits when the editor closes.
  // `flush` reads only refs, so the first render's copy is safe to keep.
  useEffect(
    () => () => {
      void flush();
    },
    [], // eslint-disable-line react-hooks/exhaustive-deps
  );

  const onTitle = (next: string) => {
    setTitle(next);
    titleRef.current = next;
    // The editor exists from the first render (`immediatelyRender`).
    schedule({ title: next, ...saveReport(editor.state.doc) });
  };

  return (
    <div className="document-editor">
      <input
        className="document-title"
        value={title}
        placeholder="Untitled"
        onChange={(e) => onTitle(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            editor?.commands.focus("start");
          }
        }}
      />
      <PassageBar editor={editor} onOpenPassage={onOpenPassage} />
      <EditorContent editor={editor} className="document-body" />
      <div className="document-status" aria-live="polite">
        {typeof status === "object" ? (
          <span className="error">Not saved: {status.error}</span>
        ) : status === "saved" ? (
          "Saved"
        ) : (
          "Saving…"
        )}
      </div>
    </div>
  );
});

/**
 * Quiet controls for Passages: mark the selected text, or remove the
 * innermost Passage at the cursor. The text stays when a mark is removed.
 */
function PassageBar({ editor, onOpenPassage }: { editor: Editor; onOpenPassage?(id: string): void }) {
  const state = useEditorState({
    editor,
    selector: ({ editor }) => ({
      canMark: editor.can().setPassage(),
      under: passageIdsAt(editor.state.doc, editor.state.selection.from)[0] ?? null,
    }),
  });

  return (
    <div className="passage-bar">
      <button
        disabled={!state.canMark}
        title="Mark the selected text as a Passage (⇧⌘P)"
        onMouseDown={(e) => e.preventDefault()}
        onClick={() => editor.chain().focus().setPassage().run()}
      >
        Mark Passage
      </button>
      {state.under && (
        <button
          title="Remove this Passage mark and keep the text"
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => editor.chain().focus().unsetPassage(state.under!).run()}
        >
          Remove Passage
        </button>
      )}
      {state.under && onOpenPassage && (
        <button
          title="Show this Passage and everything linked to it"
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => onOpenPassage(state.under!)}
        >
          Passage details
        </button>
      )}
    </div>
  );
}
