import { useCallback, useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import { documentaryApi } from "../../documentary/api";
import { ScriptScreen } from "../../documentary/ScriptScreen";
import { documentaryStatusLabel, type Documentary, type DocumentaryStatus } from "../../documentary/types";
import type { ThesisSection, ThesisSectionProps } from ".";

function DocumentarySectionView({ thesis }: ThesisSectionProps) {
  // undefined: loading; null: this Thesis has no Documentary.
  const [doc, setDoc] = useState<Documentary | null>();
  const [title, setTitle] = useState(thesis.title);
  const [error, setError] = useState<CoreError>();
  const [confirmAbandon, setConfirmAbandon] = useState(false);
  const [scriptOpen, setScriptOpen] = useState(false);

  const reload = useCallback(() => {
    documentaryApi.thesisDocumentary(thesis.id).then(setDoc, (raw) => setError(toCoreError(raw)));
  }, [thesis.id]);
  useEffect(reload, [reload]);

  async function run(action: () => Promise<unknown>) {
    setError(undefined);
    setConfirmAbandon(false);
    try {
      await action();
    } catch (raw) {
      setError(toCoreError(raw));
    }
    reload();
  }

  async function create(e: FormEvent) {
    e.preventDefault();
    await run(() => documentaryApi.createDocumentary(thesis.id, title));
  }

  const move = (d: Documentary, to: DocumentaryStatus) => run(() => documentaryApi.moveDocumentary(d.id, to));

  if (doc === undefined) return error ? <p className="error">{error.message}</p> : null;

  if (doc && scriptOpen) {
    return <ScriptScreen documentaryId={doc.id} backLabel={`← ${thesis.title}`} onClose={() => setScriptOpen(false)} />;
  }

  return (
    <div className="documentary-section">
      {error && <p className="error">{error.message}</p>}
      {doc ? (
        <div className="documentary-card">
          <p className="status">{documentaryStatusLabel[doc.status]}</p>
          <strong>{doc.title}</strong>
          <button onClick={() => setScriptOpen(true)}>Open Script</button>
          {doc.moves.length > 0 && (
            <div className="actions">
              {doc.moves
                .filter((to) => to !== "abandoned")
                .map((to) => (
                  <button key={to} onClick={() => move(doc, to)}>
                    Move to {documentaryStatusLabel[to]}
                  </button>
                ))}
              {doc.moves.includes("abandoned") &&
                (confirmAbandon ? (
                  <button onClick={() => move(doc, "abandoned")}>Confirm: abandon this Documentary</button>
                ) : (
                  <button onClick={() => setConfirmAbandon(true)}>Abandon Documentary</button>
                ))}
            </div>
          )}
        </div>
      ) : (
        <form className="documentary-create" onSubmit={create}>
          <input placeholder="Documentary title" value={title} onChange={(e) => setTitle(e.target.value)} />
          <button type="submit" disabled={!title.trim()}>
            Create Documentary
          </button>
        </form>
      )}
    </div>
  );
}

export const documentarySection: ThesisSection = {
  id: "documentary",
  title: "Documentary",
  Component: DocumentarySectionView,
};
