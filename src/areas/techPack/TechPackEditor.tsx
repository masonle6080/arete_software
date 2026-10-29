import { useCallback, useEffect, useState, type FormEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { vendorApi, type Vendor } from "../vendor/api";
import { styleApi } from "../style/api";
import { techPackApi, type TechPack, type TechPackDetails } from "./api";
import type { Style } from "../style/types";
import { TechPackExport } from "./TechPackExport";
import "./techPack.css";

export interface TechPackEditorProps {
  style: Style;
  /** Tells the Style view that the Style changed (its base or sizes). */
  onStyleChanged(): void;
}

/**
 * The Style's Tech Pack: the Point of Measure table (which also sets the
 * Style's sizes), fabric, trims, construction notes, label placement, and
 * flats. A Style without a base can start one here.
 */
export function TechPackEditor({ style, onStyleChanged }: TechPackEditorProps) {
  const [techPack, setTechPack] = useState<TechPack | null>();
  const [error, setError] = useState<CoreError>();

  // `style` (not only its id): the Base section reports a base change
  // through `onStyleChanged`, which hands every section a fresh Style.
  useEffect(() => {
    techPackApi.getTechPack(style.id).then(setTechPack, (raw) => setError(toCoreError(raw)));
  }, [style]);

  /**
   * Runs one core change and shows the Tech Pack it returns. Pass
   * `sizesChanged` when the change adds, renames, or removes a size, so the
   * Base section shows the Style's new sizes.
   */
  const run = useCallback(
    async (change: () => Promise<TechPack>, sizesChanged = false) => {
      setError(undefined);
      try {
        setTechPack(await change());
        if (sizesChanged) onStyleChanged();
      } catch (raw) {
        setError(toCoreError(raw));
      }
    },
    [onStyleChanged],
  );

  async function useTechPack() {
    setError(undefined);
    try {
      await styleApi.setStyleBase(style.id, { kind: "techPack" });
      onStyleChanged();
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  if (techPack === undefined) return error ? <p className="error">{error.message}</p> : null;

  return (
    <div className="tech-pack">
      {error && <p className="error">{error.message}</p>}
      {techPack === null ? (
        <>
          <p className="empty">This Style is not built on a Tech Pack.</p>
          <button onClick={useTechPack}>Use a Tech Pack as the base</button>
        </>
      ) : (
        <>
          <MeasureTable techPack={techPack} run={run} />
          <Details techPack={techPack} run={run} />
          <Flats techPack={techPack} run={run} />
          <TechPackExport style={style} />
        </>
      )}
    </div>
  );
}

interface PartProps {
  techPack: TechPack;
  run(change: () => Promise<TechPack>, sizesChanged?: boolean): Promise<void>;
}

/** Parses a cell. Empty text is an empty cell (`null`); other text must be a number. */
function parseNumber(text: string): number | null | undefined {
  const t = text.trim();
  if (t === "") return null;
  const n = Number(t);
  return Number.isFinite(n) ? n : undefined;
}

/** An input that keeps its own text and reports it on blur or Enter. */
function CellInput(props: { value: string; label: string; className?: string; onCommit(text: string): void }) {
  return (
    <input
      key={props.value}
      className={props.className}
      aria-label={props.label}
      defaultValue={props.value}
      onBlur={(e) => {
        if (e.currentTarget.value !== props.value) props.onCommit(e.currentTarget.value);
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter") e.currentTarget.blur();
      }}
    />
  );
}

function MeasureTable({ techPack, run }: PartProps) {
  const { sizes, pointsOfMeasure, details, styleId } = techPack;
  const [newSize, setNewSize] = useState("");
  const [newName, setNewName] = useState("");
  const [newTolerance, setNewTolerance] = useState("");

  async function addSize(e: FormEvent) {
    e.preventDefault();
    await run(() => techPackApi.addSize(styleId, newSize), true);
    setNewSize("");
  }

  async function addPointOfMeasure(e: FormEvent) {
    e.preventDefault();
    const tolerance = parseNumber(newTolerance) ?? 0;
    await run(() => techPackApi.addPointOfMeasure(styleId, newName, tolerance));
    setNewName("");
    setNewTolerance("");
  }

  return (
    <div className="tech-pack-part">
      <h3>Points of Measure ({details.unit})</h3>
      <p className="hint">The size columns are this Style's sizes.</p>
      <div className="point-of-measure-scroll">
        <table className="point-of-measure-table">
          <thead>
            <tr>
              <th>Point of Measure</th>
              <th>Tolerance ±</th>
              {sizes.map((size) => (
                <th key={size.id}>
                  <CellInput
                    className="size-name"
                    label="Size name"
                    value={size.name}
                    onCommit={(name) => run(() => techPackApi.renameSize(size.id, name), true)}
                  />
                  <button
                    className="remove"
                    title={`Remove size ${size.name} and its values`}
                    onClick={() => run(() => techPackApi.removeSize(size.id), true)}
                  >
                    ×
                  </button>
                </th>
              ))}
              <th />
            </tr>
          </thead>
          <tbody>
            {pointsOfMeasure.map((pointOfMeasure) => (
              <tr key={pointOfMeasure.id}>
                <td>
                  <CellInput
                    label="Point of Measure name"
                    value={pointOfMeasure.name}
                    onCommit={(name) => run(() => techPackApi.updatePointOfMeasure(pointOfMeasure.id, name, pointOfMeasure.tolerance))}
                  />
                </td>
                <td>
                  <CellInput
                    className="number"
                    label={`Tolerance of ${pointOfMeasure.name}`}
                    value={String(pointOfMeasure.tolerance)}
                    onCommit={(text) => {
                      const tolerance = parseNumber(text);
                      if (tolerance === undefined || tolerance === null) return;
                      run(() => techPackApi.updatePointOfMeasure(pointOfMeasure.id, pointOfMeasure.name, tolerance));
                    }}
                  />
                </td>
                {sizes.map((size, i) => (
                  <td key={size.id}>
                    <CellInput
                      className="number"
                      label={`${pointOfMeasure.name} in ${size.name}`}
                      value={pointOfMeasure.values[i] === null ? "" : String(pointOfMeasure.values[i])}
                      onCommit={(text) => {
                        const value = parseNumber(text);
                        if (value === undefined) return;
                        run(() => techPackApi.setPointOfMeasureValue(pointOfMeasure.id, size.id, value));
                      }}
                    />
                  </td>
                ))}
                <td>
                  <button
                    className="remove"
                    title={`Remove ${pointOfMeasure.name}`}
                    onClick={() => run(() => techPackApi.removePointOfMeasure(pointOfMeasure.id))}
                  >
                    ×
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="tech-pack-add">
        <form onSubmit={addPointOfMeasure}>
          <input placeholder="Point of Measure, e.g. Chest width" value={newName} onChange={(e) => setNewName(e.target.value)} />
          <input
            className="number"
            placeholder="Tolerance"
            value={newTolerance}
            onChange={(e) => setNewTolerance(e.target.value)}
          />
          <button type="submit" disabled={!newName.trim()}>
            Add Point of Measure
          </button>
        </form>
        <form onSubmit={addSize}>
          <input placeholder="Size, e.g. M" value={newSize} onChange={(e) => setNewSize(e.target.value)} />
          <button type="submit" disabled={!newSize.trim()}>
            Add size
          </button>
        </form>
      </div>
    </div>
  );
}

function Details({ techPack, run }: PartProps) {
  const [draft, setDraft] = useState<TechPackDetails>(techPack.details);
  const [vendors, setVendors] = useState<Vendor[]>([]);
  // Reset the draft only when the saved details change, not on every reload
  // (a table edit reloads the Tech Pack and must keep unsaved text here).
  const saved = JSON.stringify(techPack.details);
  useEffect(() => setDraft(JSON.parse(saved) as TechPackDetails), [saved]);
  useEffect(() => {
    vendorApi.listVendors().then(setVendors, () => setVendors([]));
  }, []);

  const changed = JSON.stringify(draft) !== saved;
  const setFabric = (patch: Partial<TechPackDetails["fabric"]>) =>
    setDraft((d) => ({ ...d, fabric: { ...d.fabric, ...patch } }));

  function save(e: FormEvent) {
    e.preventDefault();
    run(() => techPackApi.updateDetails(techPack.styleId, draft));
  }

  return (
    <form className="tech-pack-part tech-pack-details" onSubmit={save}>
      <h3>Fabric and construction</h3>
      <label>
        Unit of measure
        <input value={draft.unit} onChange={(e) => setDraft({ ...draft, unit: e.target.value })} />
      </label>
      <label>
        Fabric composition
        <input value={draft.fabric.composition} onChange={(e) => setFabric({ composition: e.target.value })} />
      </label>
      <label>
        Fabric weight
        <input value={draft.fabric.weight} onChange={(e) => setFabric({ weight: e.target.value })} />
      </label>
      <label>
        Fabric color
        <input value={draft.fabric.color} onChange={(e) => setFabric({ color: e.target.value })} />
      </label>
      <label>
        Fabric Vendor
        <select
          value={draft.fabric.vendorId ?? ""}
          onChange={(e) => setFabric({ vendorId: e.target.value === "" ? null : Number(e.target.value) })}
        >
          <option value="">None</option>
          {vendors.map((v) => (
            <option key={v.id} value={v.id}>
              {v.name}
            </option>
          ))}
        </select>
      </label>
      <label>
        Trims
        <textarea rows={2} value={draft.trims} onChange={(e) => setDraft({ ...draft, trims: e.target.value })} />
      </label>
      <label>
        Construction notes
        <textarea
          rows={4}
          value={draft.constructionNotes}
          onChange={(e) => setDraft({ ...draft, constructionNotes: e.target.value })}
        />
      </label>
      <label>
        Label placement
        <textarea
          rows={2}
          value={draft.labelPlacement}
          onChange={(e) => setDraft({ ...draft, labelPlacement: e.target.value })}
        />
      </label>
      <div>
        <button type="submit" disabled={!changed}>
          Save
        </button>
      </div>
    </form>
  );
}

function Flats({ techPack, run }: PartProps) {
  async function attach() {
    const picked = await open({ multiple: true, title: "Attach flats" });
    if (!picked) return;
    const paths = Array.isArray(picked) ? picked : [picked];
    await run(() => techPackApi.attachFlats(techPack.styleId, paths));
  }

  return (
    <div className="tech-pack-part">
      <h3>Flats</h3>
      {techPack.flats.length === 0 ? (
        <p className="empty">No flats yet.</p>
      ) : (
        <ul className="tech-pack-flats">
          {techPack.flats.map((f) => (
            <li key={f.id} title={f.path}>
              {f.fileName}
            </li>
          ))}
        </ul>
      )}
      <button onClick={attach}>Attach flats</button>
    </div>
  );
}

