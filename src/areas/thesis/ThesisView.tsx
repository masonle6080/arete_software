import { useCallback, useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { Badge, Button, SectionHeader } from "../../ui";
import type { Step } from "../../shell/navPath";
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
  /** Opens an item of this Thesis one level below the tab in the navigation path. */
  onOpen(step: Step): void;
}

/**
 * One tab of a Thesis under its header. Overview shows the statement and the
 * status moves; any other tab shows that one section.
 */
export function ThesisView({ id, tab, onOpen }: Props) {
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
                onOpen={onOpen}
              />
            </section>
          )}
        </>
      )}
    </section>
  );
}
