import { useCallback, useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import { steps } from "../../../shell/navPath";
import { Badge, Button, EmptyText } from "../../../ui";
import { dropApi } from "../../drop/api";
import { stageLabel, warningText, type Drop, type DropStage, type DropWarning, type ThesisDrops } from "../../drop/types";
import { DropStyles } from "../../style/DropStyles";
import { DropProductionOrders } from "../../productionOrder/DropProductionOrders";
import type { ThesisSection, ThesisSectionProps } from ".";

/** A move that the core stopped until the founder confirms it. */
interface PendingMove {
  to: DropStage;
  warnings: DropWarning[];
}

function DropSectionView({ thesis, onThesisChanged, onOpen }: ThesisSectionProps) {
  const [data, setData] = useState<ThesisDrops>();
  const [name, setName] = useState(thesis.title);
  const [error, setError] = useState<CoreError>();
  const [pending, setPending] = useState<PendingMove>();
  const [confirmCancel, setConfirmCancel] = useState(false);

  const reload = useCallback(() => {
    dropApi.thesisDrops(thesis.id).then(setData, (raw) => setError(toCoreError(raw)));
  }, [thesis.id]);
  useEffect(reload, [reload]);

  async function run(action: () => Promise<unknown>) {
    setError(undefined);
    try {
      await action();
    } catch (raw) {
      setError(toCoreError(raw));
    }
    reload();
    onThesisChanged();
  }

  async function create(e: FormEvent) {
    e.preventDefault();
    await run(() => dropApi.createDrop(thesis.id, name));
  }

  async function move(drop: Drop, to: DropStage, confirmed = false) {
    setPending(undefined);
    setConfirmCancel(false);
    await run(async () => {
      const result = await dropApi.moveDrop(drop.id, to, confirmed);
      if (result.outcome === "needsConfirmation") setPending({ to, warnings: result.warnings });
    });
  }

  if (!data) return error ? <p className="error">{error.message}</p> : null;

  const active = data.drops.find((d) => d.stage !== "cancelled");
  const cancelled = data.drops.filter((d) => d.stage === "cancelled");

  return (
    <div className="drop-section">
      {error && <p className="error">{error.message}</p>}

      {active && (
        <div className="drop-card">
          <div className="card-title">
            <strong>{active.name}</strong>
            <Badge lifecycle="drop" status={active.stage} />
          </div>
          {active.moves.length > 0 && (
            <div className="actions">
              {active.moves
                .filter((to) => to !== "cancelled")
                .map((to) => (
                  <Button key={to} onClick={() => move(active, to)}>
                    Move to {stageLabel[to]}
                  </Button>
                ))}
              {active.moves.includes("cancelled") &&
                (confirmCancel ? (
                  <Button variant="destructive" onClick={() => move(active, "cancelled")}>
                    Confirm: cancel this Drop
                  </Button>
                ) : (
                  <Button variant="destructive" onClick={() => setConfirmCancel(true)}>
                    Cancel Drop
                  </Button>
                ))}
            </div>
          )}
          {pending && (
            <div className="drop-warning">
              <p>Moving to {stageLabel[pending.to]} needs confirmation:</p>
              <ul>
                {pending.warnings.map((w, i) => (
                  <li key={i}>{warningText(w)}</li>
                ))}
              </ul>
              <Button onClick={() => move(active, pending.to, true)}>Move to {stageLabel[pending.to]} anyway</Button>
              <Button onClick={() => setPending(undefined)}>Keep {stageLabel[active.stage]}</Button>
            </div>
          )}
          <DropStyles drop={active} onOpen={(style) => onOpen(steps.style(style))} />
          <DropProductionOrders drop={active} />
        </div>
      )}

      {data.canCreateDrop && (
        <form className="drop-create" onSubmit={create}>
          <input placeholder="Drop name" value={name} onChange={(e) => setName(e.target.value)} />
          <Button type="submit" variant="primary" disabled={!name.trim()}>
            Create Drop
          </Button>
        </form>
      )}
      {!active && !data.canCreateDrop && <EmptyText>No Drop</EmptyText>}

      {cancelled.length > 0 && (
        <ul className="drop-cancelled">
          {cancelled.map((d) => (
            <li key={d.id}>
              {d.name} <Badge lifecycle="drop" status={d.stage} />
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export const dropSection: ThesisSection = { id: "drop", title: "Drop & Styles", Component: DropSectionView };
