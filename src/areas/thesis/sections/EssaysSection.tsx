import { useCallback, useEffect, useState } from "react";
import { toCoreError } from "../../../shell/coreError";
import { steps } from "../../../shell/navPath";
import { announceTreeChange } from "../../../shell/treeChanges";
import { Button, EmptyText } from "../../../ui";
import { documentApi, essayThesisApi, type DocumentSummary } from "../../essays/api";
import type { ThesisSection, ThesisSectionProps } from ".";

/**
 * The Essays linked to this Thesis. The founder can open, link, and unlink
 * them, and write a new one. An Essay opens inside this Thesis, and a new
 * Essay links to it at once.
 */
function EssaysSection({ thesis, onOpen }: ThesisSectionProps) {
  const [linked, setLinked] = useState<DocumentSummary[]>([]);
  const [all, setAll] = useState<DocumentSummary[]>([]);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(() => {
    essayThesisApi.essaysOfThesis(thesis.id).then(setLinked, (e) => setError(toCoreError(e).message));
    documentApi.list("essay").then(setAll, (e) => setError(toCoreError(e).message));
  }, [thesis.id]);
  useEffect(reload, [reload]);

  const change = async (action: Promise<void>) => {
    setError(null);
    try {
      await action;
    } catch (e) {
      setError(toCoreError(e).message);
    }
    reload();
    announceTreeChange();
  };

  const create = async () => {
    setError(null);
    try {
      const essay = await documentApi.create("essay", "");
      await essayThesisApi.link(essay.id, thesis.id);
      announceTreeChange();
      onOpen(steps.essay(essay.id, essay.title));
    } catch (e) {
      setError(toCoreError(e).message);
      reload();
    }
  };

  const unlinked = all.filter((e) => !linked.some((l) => l.id === e.id));

  return (
    <>
      {error && <p className="error">{error}</p>}
      {linked.length === 0 && <EmptyText>No essays linked yet.</EmptyText>}
      <ul className="thesis-essays">
        {linked.map((e) => (
          <li key={e.id}>
            <Button className="open" onClick={() => onOpen(steps.essay(e.id, e.title))}>
              {e.title || "Untitled"}
            </Button>
            <Button
              variant="plain"
              aria-label={`Unlink ${e.title || "Untitled"}`}
              onClick={() => change(essayThesisApi.unlink(e.id, thesis.id))}
            >
              Unlink
            </Button>
          </li>
        ))}
      </ul>
      <div className="actions thesis-essays-link">
        <Button onClick={create}>New essay</Button>
        {unlinked.length > 0 && (
          <select
            value=""
            aria-label="Link an essay"
            onChange={(ev) => ev.target.value && change(essayThesisApi.link(Number(ev.target.value), thesis.id))}
          >
            <option value="">Link an essay…</option>
            {unlinked.map((e) => (
              <option key={e.id} value={e.id}>
                {e.title || "Untitled"}
              </option>
            ))}
          </select>
        )}
      </div>
    </>
  );
}

export const essaysSection: ThesisSection = { id: "essays", title: "Essays", Component: EssaysSection };
