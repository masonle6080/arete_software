import { useCallback, useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
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
  /** Opens one Moodboard of this Thesis on its own screen. */
  onOpenMoodboard(board: Moodboard): void;
}

/**
 * One tab of a Thesis under its header. Overview shows the statement and the
 * status moves; any other tab shows that one section.
 */
export function ThesisView({ id, tab, onOpenStyle, onOpenMoodboard }: Props) {
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
            <p className="status">{statusLabel[thesis.status]}</p>
            <h1>{thesis.title}</h1>
            {!section && thesis.statement && <p className="statement">{thesis.statement}</p>}
            {!section && thesis.manualMoves.length > 0 && (
              <div className="actions">
                {thesis.manualMoves.map((to) => (
                  <button key={to} onClick={() => move(to)}>
                    Move to {statusLabel[to]}
                  </button>
                ))}
              </div>
            )}
          </header>
          {section && (
            <section className="thesis-section">
              <h2>{section.title}</h2>
              <section.Component
                thesis={thesis}
                onThesisChanged={reload}
                onOpenStyle={onOpenStyle}
                onOpenMoodboard={onOpenMoodboard}
              />
            </section>
          )}
        </>
      )}
    </section>
  );
}
