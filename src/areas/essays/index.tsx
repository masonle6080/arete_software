import { useCallback, useEffect, useRef, useState } from "react";
import type { Area } from "../../shell/area";
import { toCoreError } from "../../shell/coreError";
import { DocumentEditor, type DocumentEditorHandle, type DocumentSave } from "../../document/DocumentEditor";
import { OrphanedPassages } from "../passage/OrphanedPassages";
import { PassageView } from "../passage/PassageView";
import { documentApi, type DocumentSummary, type StoredDocument } from "./api";
import { EssayExport } from "./EssayExport";
import { EssayTheses } from "./EssayTheses";
import "./essays.css";

function EssayList({ onOpen }: { onOpen(id: number): void }) {
  const [essays, setEssays] = useState<DocumentSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    documentApi.list("essay").then(setEssays, (e) => setError(toCoreError(e).message));
  }, []);

  const create = async () => {
    try {
      const essay = await documentApi.create("essay", "");
      onOpen(essay.id);
    } catch (e) {
      setError(toCoreError(e).message);
    }
  };

  return (
    <section>
      <header className="essay-list-header">
        <h1>Essays</h1>
        <button onClick={create}>New essay</button>
      </header>
      {error && <p className="error">{error}</p>}
      {essays?.length === 0 && <p className="muted">No essays yet.</p>}
      <ul className="essay-list">
        {essays?.map((e) => (
          <li key={e.id}>
            <button onClick={() => onOpen(e.id)}>
              <span>{e.title || "Untitled"}</span>
              <time dateTime={e.updatedAt}>{new Date(e.updatedAt).toLocaleString()}</time>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

interface EssayScreenProps {
  id: number;
  onClose(): void;
  /** Text of the back button, for example "← Essays". */
  backLabel?: string;
}

/**
 * One Essay in the editor, with its Thesis links and orphaned Passages.
 * Other areas can open it too. A Passage opens in place over the editor; the
 * editor stays mounted, so undo still works after coming back.
 */
export function EssayScreen({ id, onClose, backLabel = "← Essays" }: EssayScreenProps) {
  const [essay, setEssay] = useState<StoredDocument | null>(null);
  const [error, setError] = useState<string | null>(null);
  const editor = useRef<DocumentEditorHandle>(null);
  const [saves, setSaves] = useState(0);
  const [passageId, setPassageId] = useState<string | null>(null);

  useEffect(() => {
    documentApi.open(id).then(setEssay, (e) => setError(toCoreError(e).message));
  }, [id]);

  const save = useCallback(
    async ({ title, markdown, passages }: DocumentSave) => {
      await documentApi.save(id, { title, markdown }, passages);
      setSaves((n) => n + 1);
    },
    [id],
  );

  const openPassage = useCallback(async (pid: string) => {
    // The Passage must be saved before the core knows it.
    await editor.current?.flush();
    setPassageId(pid);
  }, []);

  return (
    <section>
      {passageId && (
        <PassageView key={passageId} id={passageId} backLabel="← Essay" onBack={() => setPassageId(null)} />
      )}
      <div hidden={passageId !== null}>
        <button
          className="essay-back"
          onClick={async () => {
            await editor.current?.flush();
            onClose();
          }}
        >
          {backLabel}
        </button>
        {error && <p className="error">{error}</p>}
        {essay && <EssayTheses essayId={essay.id} />}
        {essay && <EssayExport essayId={essay.id} flush={async () => editor.current?.flush()} />}
        {essay && <OrphanedPassages documentId={essay.id} version={saves} onOpen={openPassage} />}
        {essay && (
          <DocumentEditor
            ref={editor}
            key={essay.id}
            title={essay.title}
            markdown={essay.markdown}
            onSave={save}
            onOpenPassage={openPassage}
          />
        )}
      </div>
    </section>
  );
}

function EssaysArea() {
  const [openId, setOpenId] = useState<number | null>(null);
  return openId ? (
    <EssayScreen id={openId} onClose={() => setOpenId(null)} />
  ) : (
    <EssayList onOpen={setOpenId} />
  );
}

export const essaysArea: Area = { id: "essays", label: "Essays", Screen: EssaysArea };
