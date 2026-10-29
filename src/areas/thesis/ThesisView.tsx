import { useCallback, useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { thesisApi } from "./api";
import { thesisSections } from "./sections";
import { statusLabel, type Thesis, type ThesisStatus } from "./types";

interface Props {
  id: number;
  onBack(): void;
  /** Opens the Style view for one Style of this Thesis's Drop. */
  onOpenStyle(styleId: number): void;
}

/** One Thesis: its header and status moves, then every registered section. */
export function ThesisView({ id, onBack, onOpenStyle }: Props) {
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

  return (
    <section className="thesis-view">
      <button className="back" onClick={onBack}>
        ← Theses
      </button>
      {error && <p className="error">{error.message}</p>}
      {thesis && (
        <>
          <header>
            <p className="status">{statusLabel[thesis.status]}</p>
            <h1>{thesis.title}</h1>
            {thesis.statement && <p className="statement">{thesis.statement}</p>}
            {thesis.manualMoves.length > 0 && (
              <div className="actions">
                {thesis.manualMoves.map((to) => (
                  <button key={to} onClick={() => move(to)}>
                    Move to {statusLabel[to]}
                  </button>
                ))}
              </div>
            )}
          </header>
          {thesisSections.map(({ id: sectionId, title, Component }) => (
            <section key={sectionId} className="thesis-section">
              <h2>{title}</h2>
              <Component thesis={thesis} onThesisChanged={reload} onOpenStyle={onOpenStyle} />
            </section>
          ))}
        </>
      )}
    </section>
  );
}
