import { useCallback, useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import { steps } from "../../../shell/navPath";
import { Badge, Button } from "../../../ui";
import { documentaryApi } from "../../documentary/api";
import { documentaryStatusLabel, type Documentary, type DocumentaryStatus } from "../../documentary/types";
import type { ThesisSection, ThesisSectionProps } from ".";

function DocumentarySectionView({ thesis, onThesisChanged, onOpen }: ThesisSectionProps) {
  // undefined: loading; null: this Thesis has no Documentary.
  const [doc, setDoc] = useState<Documentary | null>();
  const [title, setTitle] = useState(thesis.title);
  const [error, setError] = useState<CoreError>();
  const [confirmAbandon, setConfirmAbandon] = useState(false);

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
    // The Thesis header shows the Documentary status.
    onThesisChanged();
  }

  async function create(e: FormEvent) {
    e.preventDefault();
    await run(() => documentaryApi.createDocumentary(thesis.id, title));
  }

  const move = (d: Documentary, to: DocumentaryStatus) => run(() => documentaryApi.moveDocumentary(d.id, to));

  if (doc === undefined) return error ? <p className="error">{error.message}</p> : null;

  return (
    <div className="documentary-section">
      {error && <p className="error">{error.message}</p>}
      {doc ? (
        <div className="documentary-card">
          <div className="card-title">
            <strong>{doc.title}</strong>
            <Badge lifecycle="documentary" status={doc.status} />
          </div>
          <Button onClick={() => onOpen(steps.script(doc.id, doc.title))}>Open Script</Button>
          {doc.moves.length > 0 && (
            <div className="actions">
              {doc.moves
                .filter((to) => to !== "abandoned")
                .map((to) => (
                  <Button key={to} onClick={() => move(doc, to)}>
                    Move to {documentaryStatusLabel[to]}
                  </Button>
                ))}
              {doc.moves.includes("abandoned") &&
                (confirmAbandon ? (
                  <Button variant="destructive" onClick={() => move(doc, "abandoned")}>
                    Confirm: abandon this Documentary
                  </Button>
                ) : (
                  <Button variant="destructive" onClick={() => setConfirmAbandon(true)}>
                    Abandon Documentary
                  </Button>
                ))}
            </div>
          )}
        </div>
      ) : (
        <form className="documentary-create" onSubmit={create}>
          <input placeholder="Documentary title" value={title} onChange={(e) => setTitle(e.target.value)} />
          <Button type="submit" variant="primary" disabled={!title.trim()}>
            Create Documentary
          </Button>
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
