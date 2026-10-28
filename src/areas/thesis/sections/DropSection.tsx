import { useCallback, useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import { dropApi } from "../../drop/api";
import { stageLabel, type Drop, type DropStage, type DropWarning, type ThesisDrops } from "../../drop/types";
import { DropStyles } from "../../style/DropStyles";
import type { ThesisSection, ThesisSectionProps } from ".";

/** A move that the core stopped until the founder confirms it. */
interface PendingMove {
  to: DropStage;
  warnings: DropWarning[];
}

function DropSectionView({ thesis, onThesisChanged, onOpenStyle }: ThesisSectionProps) {
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
          <p className="status">{stageLabel[active.stage]}</p>
          <strong>{active.name}</strong>
          {active.moves.length > 0 && (
            <div className="actions">
              {active.moves
                .filter((to) => to !== "cancelled")
                .map((to) => (
                  <button key={to} onClick={() => move(active, to)}>
                    Move to {stageLabel[to]}
                  </button>
                ))}
              {active.moves.includes("cancelled") &&
                (confirmCancel ? (
                  <button onClick={() => move(active, "cancelled")}>Confirm: cancel this Drop</button>
                ) : (
                  <button onClick={() => setConfirmCancel(true)}>Cancel Drop</button>
                ))}
            </div>
          )}
          {pending && (
            <div className="drop-warning">
              <p>
                Moving to {stageLabel[pending.to]} needs confirmation:{" "}
                {pending.warnings.map((w) => w.kind).join(", ")}
              </p>
              <button onClick={() => move(active, pending.to, true)}>Move to {stageLabel[pending.to]} anyway</button>
              <button onClick={() => setPending(undefined)}>Keep {stageLabel[active.stage]}</button>
            </div>
          )}
          <DropStyles drop={active} onOpen={onOpenStyle} />
        </div>
      )}

      {data.canCreateDrop && (
        <form className="drop-create" onSubmit={create}>
          <input placeholder="Drop name" value={name} onChange={(e) => setName(e.target.value)} />
          <button type="submit" disabled={!name.trim()}>
            Create Drop
          </button>
        </form>
      )}
      {!active && !data.canCreateDrop && <p className="empty">No Drop</p>}

      {cancelled.length > 0 && (
        <ul className="drop-cancelled">
          {cancelled.map((d) => (
            <li key={d.id}>
              {d.name} <span className="status">Cancelled</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export const dropSection: ThesisSection = { id: "drop", title: "Drop", Component: DropSectionView };
