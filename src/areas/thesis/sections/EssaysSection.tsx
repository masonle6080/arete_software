import { useCallback, useEffect, useState } from "react";
import { toCoreError } from "../../../shell/coreError";
import { EssayScreen } from "../../essays";
import { documentApi, essayThesisApi, type DocumentSummary } from "../../essays/api";
import type { ThesisSection, ThesisSectionProps } from ".";

/** The Essays linked to this Thesis. The founder can open, link, and unlink them. */
function EssaysSection({ thesis }: ThesisSectionProps) {
  const [linked, setLinked] = useState<DocumentSummary[]>([]);
  const [all, setAll] = useState<DocumentSummary[]>([]);
  const [openId, setOpenId] = useState<number | null>(null);
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
  };

  if (openId !== null) {
    return (
      <EssayScreen
        id={openId}
        backLabel={`← ${thesis.title}`}
        onClose={() => {
          setOpenId(null);
          reload();
        }}
      />
    );
  }

  const unlinked = all.filter((e) => !linked.some((l) => l.id === e.id));

  return (
    <>
      {error && <p className="error">{error}</p>}
      {linked.length === 0 && <p className="muted">No essays linked yet.</p>}
      <ul className="thesis-essays">
        {linked.map((e) => (
          <li key={e.id}>
            <button className="open" onClick={() => setOpenId(e.id)}>
              {e.title || "Untitled"}
            </button>
            <button
              className="unlink"
              aria-label={`Unlink ${e.title || "Untitled"}`}
              onClick={() => change(essayThesisApi.unlink(e.id, thesis.id))}
            >
              Unlink
            </button>
          </li>
        ))}
      </ul>
      {unlinked.length > 0 && (
        <div className="thesis-essays-link">
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
        </div>
      )}
    </>
  );
}

export const essaysSection: ThesisSection = { id: "essays", title: "Essays", Component: EssaysSection };
