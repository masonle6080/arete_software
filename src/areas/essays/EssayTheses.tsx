import { useCallback, useEffect, useState } from "react";
import { toCoreError } from "../../shell/coreError";
import { thesisApi } from "../thesis/api";
import type { Thesis } from "../thesis/types";
import { essayThesisApi } from "./api";

/**
 * The Theses an Essay supports, with controls to link and unlink them. An
 * Essay can support any number of Theses, including none.
 */
export function EssayTheses({ essayId }: { essayId: number }) {
  const [linked, setLinked] = useState<Thesis[]>([]);
  const [all, setAll] = useState<Thesis[]>([]);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(() => {
    essayThesisApi.thesesOfEssay(essayId).then(setLinked, (e) => setError(toCoreError(e).message));
    thesisApi
      .listTheses()
      .then((groups) => setAll(groups.flatMap((g) => g.theses)), (e) => setError(toCoreError(e).message));
  }, [essayId]);
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

  const unlinked = all.filter((t) => !linked.some((l) => l.id === t.id));

  return (
    <div className="essay-theses">
      <span className="muted">Theses:</span>
      {linked.length === 0 && <span className="muted">none</span>}
      {linked.map((t) => (
        <span key={t.id} className="essay-thesis-chip">
          {t.title}
          <button
            aria-label={`Unlink from ${t.title}`}
            title="Unlink"
            onClick={() => change(essayThesisApi.unlink(essayId, t.id))}
          >
            ×
          </button>
        </span>
      ))}
      {unlinked.length > 0 && (
        <select
          value=""
          aria-label="Link to a Thesis"
          onChange={(e) => e.target.value && change(essayThesisApi.link(essayId, Number(e.target.value)))}
        >
          <option value="">Link to a Thesis…</option>
          {unlinked.map((t) => (
            <option key={t.id} value={t.id}>
              {t.title}
            </option>
          ))}
        </select>
      )}
      {error && <p className="error">{error}</p>}
    </div>
  );
}
