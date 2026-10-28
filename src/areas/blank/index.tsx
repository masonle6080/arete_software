import { useEffect, useState } from "react";
import type { Area } from "../../shell/area";
import { toCoreError } from "../../shell/coreError";
import { vendorApi, type Vendor } from "../vendor/api";
import { blankApi, blankLabel, formatCost, type Blank, type BlankInput } from "./api";
import "./blank.css";

/** The form's text fields. Lists are comma-separated; cost is in major units. */
interface Draft {
  brand: string;
  model: string;
  vendorId: number | null;
  colors: string;
  sizes: string;
  cost: string;
  currency: string;
  notes: string;
}

/** `null` id: a new Blank. */
type Editing = { id: number | null; draft: Draft };

const emptyDraft: Draft = { brand: "", model: "", vendorId: null, colors: "", sizes: "", cost: "", currency: "", notes: "" };

function toDraft(b: Blank): Draft {
  return {
    brand: b.brand,
    model: b.model,
    vendorId: b.vendorId,
    colors: b.colors.join(", "),
    sizes: b.sizes.join(", "),
    cost: b.costCents === null ? "" : (b.costCents / 100).toFixed(2),
    currency: b.currency,
    notes: b.notes,
  };
}

const splitList = (text: string) => text.split(",").map((s) => s.trim()).filter(Boolean);

/** Converts the form to core input. The core checks every rule; this only parses. */
function toInput(d: Draft): BlankInput | string {
  if (d.vendorId === null) return "Choose a Vendor.";
  const cost = d.cost.trim();
  const costNumber = Number(cost);
  if (cost !== "" && !Number.isFinite(costNumber)) return "Enter the cost as a number, for example 12.50.";
  return {
    brand: d.brand,
    model: d.model,
    vendorId: d.vendorId,
    colors: splitList(d.colors),
    sizes: splitList(d.sizes),
    costCents: cost === "" ? null : Math.round(costNumber * 100),
    currency: d.currency,
    notes: d.notes,
  };
}

function BlankScreen() {
  const [blanks, setBlanks] = useState<Blank[]>([]);
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [editing, setEditing] = useState<Editing | null>(null);
  const [error, setError] = useState<string>();

  async function reload() {
    try {
      const [b, v] = await Promise.all([blankApi.listBlanks(), vendorApi.listVendors()]);
      setBlanks(b);
      setVendors(v);
    } catch (raw) {
      setError(toCoreError(raw).message);
    }
  }

  useEffect(() => {
    reload();
  }, []);

  async function save() {
    if (!editing) return;
    const input = toInput(editing.draft);
    if (typeof input === "string") return setError(input);
    setError(undefined);
    try {
      if (editing.id === null) await blankApi.createBlank(input);
      else await blankApi.updateBlank(editing.id, input);
      setEditing(null);
      await reload();
    } catch (raw) {
      setError(toCoreError(raw).message);
    }
  }

  function startEdit(next: Editing | null) {
    setError(undefined);
    setEditing(next);
  }

  return (
    <section className="blanks">
      <header>
        <h1>Blanks</h1>
        <button onClick={() => startEdit({ id: null, draft: emptyDraft })}>New Blank</button>
      </header>

      {editing && (
        <BlankForm
          editing={editing}
          vendors={vendors}
          onChange={(draft) => setEditing({ ...editing, draft })}
          onSave={save}
          onCancel={() => startEdit(null)}
        />
      )}

      {error && <p className="error">{error}</p>}

      {blanks.length === 0 ? (
        <p className="muted">No Blanks yet.</p>
      ) : (
        <table className="blank-list">
          <thead>
            <tr>
              <th>Blank</th>
              <th>Vendor</th>
              <th>Colors</th>
              <th>Sizes</th>
              <th>Cost</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {blanks.map((b) => (
              <tr key={b.id} title={b.notes || undefined}>
                <td className="name">{blankLabel(b)}</td>
                <td>{b.vendorName}</td>
                <td>{b.colors.join(", ") || "—"}</td>
                <td>{b.sizes.join(", ")}</td>
                <td>{formatCost(b)}</td>
                <td>
                  <button onClick={() => startEdit({ id: b.id, draft: toDraft(b) })}>Edit</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

function BlankForm(props: {
  editing: Editing;
  vendors: Vendor[];
  onChange(draft: Draft): void;
  onSave(): void;
  onCancel(): void;
}) {
  const { editing, vendors, onChange } = props;
  const d = editing.draft;
  const set = (patch: Partial<Draft>) => onChange({ ...d, ...patch });

  return (
    <form className="blank-form" onSubmit={(e) => { e.preventDefault(); props.onSave(); }}>
      <label>
        Brand
        <input autoFocus value={d.brand} onChange={(e) => set({ brand: e.target.value })} />
      </label>
      <label>
        Model
        <input value={d.model} onChange={(e) => set({ model: e.target.value })} />
      </label>
      <label>
        Vendor
        <select
          value={d.vendorId ?? ""}
          onChange={(e) => set({ vendorId: e.target.value === "" ? null : Number(e.target.value) })}
        >
          <option value="">Choose a Vendor</option>
          {vendors.map((v) => (
            <option key={v.id} value={v.id}>{v.name}</option>
          ))}
        </select>
      </label>
      <label>
        Available colors (comma-separated)
        <input placeholder="Bone, Black" value={d.colors} onChange={(e) => set({ colors: e.target.value })} />
      </label>
      <label>
        Available sizes (comma-separated, in order)
        <input placeholder="S, M, L, XL" value={d.sizes} onChange={(e) => set({ sizes: e.target.value })} />
      </label>
      <div className="cost-row">
        <label>
          Unit cost
          <input inputMode="decimal" placeholder="12.50" value={d.cost} onChange={(e) => set({ cost: e.target.value })} />
        </label>
        <label>
          Currency
          <input placeholder="USD" value={d.currency} onChange={(e) => set({ currency: e.target.value })} />
        </label>
      </div>
      <label>
        Notes
        <textarea rows={3} value={d.notes} onChange={(e) => set({ notes: e.target.value })} />
      </label>
      <div className="actions">
        <button type="submit">{editing.id === null ? "Create" : "Save"}</button>
        <button type="button" onClick={props.onCancel}>Cancel</button>
      </div>
    </form>
  );
}

export const blankArea: Area = { id: "blank", label: "Blanks", Screen: BlankScreen };
