import { EditorContent, useEditor } from "@tiptap/react";
import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from "react";
import { documentExtensions } from "./extensions";
import { fromMarkdown, toMarkdown } from "./markdown";
import "./editor.css";

export interface DocumentEditorProps {
  title: string;
  markdown: string;
  /** Called after edits pause, and once more on close if edits are unsaved. */
  onSave(edit: { title: string; markdown: string }): Promise<void>;
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
  { title: initialTitle, markdown, onSave },
  ref,
) {
  const [title, setTitle] = useState(initialTitle);
  const [status, setStatus] = useState<SaveStatus>("saved");
  const lastSaved = useRef({ title: initialTitle, markdown });
  const pending = useRef<{ title: string; markdown: string } | null>(null);
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
    onUpdate: ({ editor }) => schedule(titleRef.current, toMarkdown(editor.state.doc)),
  });

  const schedule = (nextTitle: string, nextMarkdown: string) => {
    pending.current = { title: nextTitle, markdown: nextMarkdown };
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
    schedule(next, editor ? toMarkdown(editor.state.doc) : lastSaved.current.markdown);
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
