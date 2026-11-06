import { useCallback, useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { announceThesisChange } from "../../shell/thesisChanges";
import { Badge, Button, SectionHeader } from "../../ui";
import type { Step, ThesisTab } from "../../shell/navPath";
import type { View } from "../../shell/sidebar";
import { thesisApi } from "./api";
import { thesisSections } from "./sections";
import { ThesisChips } from "./ThesisChips";
import { ThesisOverview } from "./ThesisOverview";
import { statusLabel, warningText, type Thesis, type ThesisStatus, type ThesisWarning } from "./types";

/** A move that the core stopped until the founder confirms it. */
interface PendingMove {
  to: ThesisStatus;
  warnings: ThesisWarning[];
}

interface Props {
  id: number;
  tab: ThesisTab;
  /** Opens an item of this Thesis one level below the tab in the navigation path. */
  onOpen(step: Step): void;
  /** Opens a tab of this Thesis, or an item on top of its tab, as the sidebar does. */
  onGo(view: View): void;
}

/**
 * One tab of a Thesis under its header, which shows the status Badge, the
 * title, and the chips on every tab. Overview adds the statement and the
 * status moves to the header, and sums up the Thesis below it; any other
 * tab shows that one section.
 */
export function ThesisView({ id, tab, onOpen, onGo }: Props) {
  const [thesis, setThesis] = useState<Thesis>();
  const [error, setError] = useState<CoreError>();
  const [pending, setPending] = useState<PendingMove>();
  const [confirmAbandon, setConfirmAbandon] = useState(false);

  const reload = useCallback(() => {
    thesisApi.getThesis(id).then(setThesis, (raw) => setError(toCoreError(raw)));
  }, [id]);
  useEffect(reload, [reload]);

  async function move(to: ThesisStatus, confirmed = false) {
    setError(undefined);
    setPending(undefined);
    setConfirmAbandon(false);
    try {
      const result = await thesisApi.moveThesis(id, to, confirmed);
      if (result.outcome === "needsConfirmation") {
        setThesis(result.thesis);
        setPending({ to, warnings: result.warnings });
      } else {
        const { outcome: _, ...moved } = result;
        setThesis(moved);
        announceThesisChange();
      }
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  const section = tab === "overview" ? undefined : thesisSections[tab];

  return (
    <section className="thesis-view">
      {error && <p className="error">{error.message}</p>}
      {thesis && (
        <>
          <header>
            <Badge lifecycle="thesis" status={thesis.status} />
            <h1>{thesis.title}</h1>
            <ThesisChips thesis={thesis} onOpen={onOpen} />
            {!section && thesis.statement && <p className="statement">{thesis.statement}</p>}
            {!section && thesis.moves.length > 0 && (
              <div className="actions">
                {thesis.moves
                  .filter((to) => to !== "abandoned")
                  .map((to) => (
                    <Button key={to} onClick={() => move(to)}>
                      Move to {statusLabel[to]}
                    </Button>
                  ))}
                {thesis.moves.includes("abandoned") &&
                  (confirmAbandon ? (
                    <Button variant="destructive" onClick={() => move("abandoned")}>
                      Confirm: abandon this Thesis
                    </Button>
                  ) : (
                    <Button variant="destructive" onClick={() => setConfirmAbandon(true)}>
                      Abandon
                    </Button>
                  ))}
              </div>
            )}
            {!section && pending && (
              <div className="move-warning">
                <p>Moving to {statusLabel[pending.to]} needs confirmation:</p>
                <ul>
                  {pending.warnings.map((w, i) => (
                    <li key={i}>{warningText(w)}</li>
                  ))}
                </ul>
                <Button onClick={() => move(pending.to, true)}>Move to {statusLabel[pending.to]} anyway</Button>
                <Button onClick={() => setPending(undefined)}>Keep {statusLabel[thesis.status]}</Button>
              </div>
            )}
          </header>
          {!section && <ThesisOverview thesis={thesis} onGo={onGo} />}
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
