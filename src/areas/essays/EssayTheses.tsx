import { useCallback, useEffect, useState } from "react";
import { toCoreError } from "../../shell/coreError";
import { steps, type Step } from "../../shell/navPath";
import { announceTreeChange } from "../../shell/treeChanges";
import { linksApi, type EssayLinks } from "../links/api";
import { thesisApi } from "../thesis/api";
import type { Thesis } from "../thesis/types";
import { essayThesisApi } from "./api";
import { Chip, countLabel } from "../../ui";

interface Props {
  essayId: number;
  /** Changes after each save, so the Passage counts follow the text. */
  version: number;
  onOpen(step: Step): void;
}

/**
 * The Essay header chips: the Theses it supports, its number of Passages,
 * and the number of Design Decisions those Passages motivate. A Thesis chip
 * opens that Thesis and can unlink it; a select links another. An Essay can
 * support any number of Theses, including none.
 */
export function EssayTheses({ essayId, version, onOpen }: Props) {
  const [links, setLinks] = useState<EssayLinks>();
  const [all, setAll] = useState<Thesis[]>([]);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(() => {
    linksApi.essay(essayId).then(setLinks, (e) => setError(toCoreError(e).message));
  }, [essayId]);
  useEffect(reload, [reload, version]);

  useEffect(() => {
    thesisApi
      .listTheses()
      .then((groups) => setAll(groups.flatMap((g) => g.theses)), (e) => setError(toCoreError(e).message));
  }, []);

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

  if (!links) return error ? <p className="error">{error}</p> : null;
  const linked = links.theses;
  const unlinked = all.filter((t) => !linked.some((l) => l.id === t.id));

  return (
    <div className="essay-theses chips header-chips">
      {linked.length === 0 && <span className="muted">No Theses</span>}
      {linked.map((t) => (
        <Chip
          key={t.id}
          title={`Open ${t.title}`}
          onClick={() => onOpen(steps.thesis("overview", t.id))}
          removeLabel={`Unlink from ${t.title}`}
          onRemove={() => change(essayThesisApi.unlink(essayId, t.id))}
        >
          {t.title}
        </Chip>
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
      <Chip>{countLabel(links.passageCount, "Passage")}</Chip>
      <Chip>{countLabel(links.designDecisionCount, "Design Decision")}</Chip>
      {error && <p className="error">{error}</p>}
    </div>
  );
}
