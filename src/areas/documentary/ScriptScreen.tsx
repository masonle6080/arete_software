import { useCallback, useEffect, useRef, useState } from "react";
import { toCoreError } from "../../shell/coreError";
import { DocumentEditor, type DocumentEditorHandle, type DocumentSave } from "../../document/DocumentEditor";
import { documentApi, type StoredDocument } from "../essays/api";
import { OrphanedPassages } from "../passage/OrphanedPassages";
import { PassageView } from "../passage/PassageView";
import { documentaryApi } from "./api";
import "../essays/essays.css";

interface Props {
  documentaryId: number;
  onClose(): void;
  /** Text of the back button, for example "← Amor fati". */
  backLabel: string;
}

/**
 * The Script of a Documentary in the Essay editor, with its orphaned
 * Passages. Saves send the save report, so Passages work as in Essays. A
 * Passage opens in place over the editor; the editor stays mounted.
 */
export function ScriptScreen({ documentaryId, onClose, backLabel }: Props) {
  const [script, setScript] = useState<StoredDocument | null>(null);
  const [error, setError] = useState<string | null>(null);
  const editor = useRef<DocumentEditorHandle>(null);
  const [saves, setSaves] = useState(0);
  const [passageId, setPassageId] = useState<string | null>(null);

  useEffect(() => {
    documentaryApi.script(documentaryId).then(setScript, (e) => setError(toCoreError(e).message));
  }, [documentaryId]);

  const scriptId = script?.id;
  const save = useCallback(
    async ({ title, markdown, passages }: DocumentSave) => {
      if (scriptId === undefined) return;
      await documentApi.save(scriptId, { title, markdown }, passages);
      setSaves((n) => n + 1);
    },
    [scriptId],
  );

  const openPassage = useCallback(async (pid: string) => {
    // The Passage must be saved before the core knows it.
    await editor.current?.flush();
    setPassageId(pid);
  }, []);

  return (
    <section>
      {passageId && (
        <PassageView key={passageId} id={passageId} backLabel="← Script" onBack={() => setPassageId(null)} />
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
        {script && <OrphanedPassages documentId={script.id} version={saves} onOpen={openPassage} />}
        {script && (
          <DocumentEditor
            ref={editor}
            key={script.id}
            title={script.title}
            markdown={script.markdown}
            onSave={save}
            onOpenPassage={openPassage}
          />
        )}
      </div>
    </section>
  );
}
