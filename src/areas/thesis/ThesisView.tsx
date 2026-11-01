import { useCallback, useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { Badge, Button, SectionHeader } from "../../ui";
import type { Moodboard } from "../moodboard/types";
import { thesisApi } from "./api";
import { thesisSections } from "./sections";
import { statusLabel, type Thesis, type ThesisStatus } from "./types";

/** The sidebar tabs of the current Thesis: Overview, then every registered section. */
export const thesisTabs: { id: string; title: string }[] = [
  { id: "overview", title: "Overview" },
  ...thesisSections.map(({ id, title }) => ({ id, title })),
];

interface Props {
  id: number;
  /** "overview" or a Thesis section ID. */
  tab: string;
  /** Opens the Style view for one Style of this Thesis's Drop. */
  onOpenStyle(styleId: number): void;
  /** Opens one Essay linked to this Thesis. */
  onOpenEssay(essayId: number): void;
  /** Opens one Moodboard of this Thesis on its own screen. */
  onOpenMoodboard(board: Moodboard): void;
}

/**
 * One tab of a Thesis under its header. Overview shows the statement and the
 * status moves; any other tab shows that one section.
 */
export function ThesisView({ id, tab, onOpenStyle, onOpenEssay, onOpenMoodboard }: Props) {
  const [thesis, setThesis] = useState<Thesis>();
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    thesisApi.getThesis(id).then(setThesis, (raw) => setError(toCoreError(raw)));
  }, [id]);
  useEffect(reload, [reload]);

  async function move(to: ThesisStatus) {
    setError(undefined);
    try {
      setThesis(await thesisApi.moveThesis(id, to));
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  const section = thesisSections.find((s) => s.id === tab);

  return (
    <section className="thesis-view">
      {error && <p className="error">{error.message}</p>}
      {thesis && (
        <>
          <header>
            <Badge lifecycle="thesis" status={thesis.status} />
            <h1>{thesis.title}</h1>
            {!section && thesis.statement && <p className="statement">{thesis.statement}</p>}
            {!section && thesis.manualMoves.length > 0 && (
              <div className="actions">
                {thesis.manualMoves.map((to) => (
                  <Button key={to} onClick={() => move(to)}>
                    Move to {statusLabel[to]}
                  </Button>
                ))}
              </div>
            )}
          </header>
          {section && (
            <section className="thesis-section">
              <SectionHeader title={section.title} />
              <section.Component
                thesis={thesis}
                onThesisChanged={reload}
                onOpenStyle={onOpenStyle}
                onOpenEssay={onOpenEssay}
                onOpenMoodboard={onOpenMoodboard}
              />
            </section>
          )}
        </>
      )}
    </section>
  );
}
