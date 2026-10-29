import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import { decorationApi, type Decoration, type DecorationInput } from "../../decoration/api";
import { vendorApi, type Vendor } from "../../vendor/api";
import type { StyleSection, StyleSectionProps } from ".";

const EMPTY: DecorationInput = { technique: "", placement: "", size: "", colors: [], vendorId: null };

/** The Decorations of the Style: technique, placement, size, colors, artwork, and Vendor. */
function DecorationsSectionView({ style }: StyleSectionProps) {
  const [decorations, setDecorations] = useState<Decoration[]>([]);
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [editingId, setEditingId] = useState<number | "new">();
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    decorationApi.ofStyle(style.id).then(setDecorations, (raw) => setError(toCoreError(raw)));
  }, [style.id]);
  useEffect(reload, [reload]);
  useEffect(() => {
    vendorApi.listVendors().then(setVendors, (raw) => setError(toCoreError(raw)));
  }, []);

  async function run(action: () => Promise<unknown>): Promise<boolean> {
    setError(undefined);
    try {
      await action();
      reload();
      return true;
    } catch (raw) {
      setError(toCoreError(raw));
      return false;
    }
  }

  async function attachArtwork(id: number) {
    const picked = await open({ multiple: false, title: "Choose the artwork file" });
    if (typeof picked !== "string") return;
    await run(() => decorationApi.setArtwork(id, picked));
  }

  return (
    <div className="decorations">
      {error && <p className="error">{error.message}</p>}
      {decorations.length === 0 ? (
        <p className="empty">No Decorations yet.</p>
      ) : (
        <ul>
          {decorations.map((d) =>
            editingId === d.id ? (
              <li key={d.id} className="decoration">
                <DecorationForm
                  initial={{ ...d }}
                  vendors={vendors}
                  submitLabel="Save"
                  onSubmit={async (input) => {
                    if (await run(() => decorationApi.update(d.id, input))) setEditingId(undefined);
                  }}
                  onCancel={() => setEditingId(undefined)}
                />
              </li>
            ) : (
              <li key={d.id} className="decoration">
                <p className="technique">{d.technique}</p>
                <dl>
                  {d.placement && (
                    <>
                      <dt>Placement</dt>
                      <dd>{d.placement}</dd>
                    </>
                  )}
                  {d.size && (
                    <>
                      <dt>Size</dt>
                      <dd>{d.size}</dd>
                    </>
                  )}
                  {d.colors.length > 0 && (
                    <>
                      <dt>Colors</dt>
                      <dd>{d.colors.join(", ")}</dd>
                    </>
                  )}
                  <dt>Vendor</dt>
                  <dd>{d.vendorName ?? "Not chosen"}</dd>
                  <dt>Artwork</dt>
                  <dd title={d.artwork?.path}>{d.artwork?.fileName ?? "None"}</dd>
                </dl>
                <div className="actions">
                  <button onClick={() => setEditingId(d.id)}>Edit</button>
                  <button onClick={() => attachArtwork(d.id)}>{d.artwork ? "Replace artwork" : "Add artwork"}</button>
                  {d.artwork && <button onClick={() => run(() => decorationApi.clearArtwork(d.id))}>Remove artwork</button>}
                  <button onClick={() => run(() => decorationApi.remove(d.id))}>Remove Decoration</button>
                </div>
              </li>
            ),
          )}
        </ul>
      )}
      {editingId === "new" ? (
        <DecorationForm
          initial={EMPTY}
          vendors={vendors}
          submitLabel="Add Decoration"
          onSubmit={async (input) => {
            if (await run(() => decorationApi.add(style.id, input))) setEditingId(undefined);
          }}
          onCancel={() => setEditingId(undefined)}
        />
      ) : (
        <button onClick={() => setEditingId("new")}>Add Decoration</button>
      )}
    </div>
  );
}

interface FormProps {
  initial: DecorationInput;
  vendors: Vendor[];
  submitLabel: string;
  onSubmit(input: DecorationInput): void;
  onCancel(): void;
}

/** The fields of one Decoration. Colors are one comma-separated line. */
function DecorationForm({ initial, vendors, submitLabel, onSubmit, onCancel }: FormProps) {
  const [technique, setTechnique] = useState(initial.technique);
  const [placement, setPlacement] = useState(initial.placement);
  const [size, setSize] = useState(initial.size);
  const [colors, setColors] = useState(initial.colors.join(", "));
  const [vendorId, setVendorId] = useState<number | null>(initial.vendorId);

  function submit() {
    onSubmit({ technique, placement, size, colors: colors.split(","), vendorId });
  }

  return (
    <div className="decoration-form">
      <label>
        Technique
        <input value={technique} placeholder="Screen print, embroidery, dye… (required)" onChange={(e) => setTechnique(e.target.value)} />
      </label>
      <label>
        Placement
        <input value={placement} placeholder="Center chest" onChange={(e) => setPlacement(e.target.value)} />
      </label>
      <label>
        Size
        <input value={size} placeholder="10 x 12 cm" onChange={(e) => setSize(e.target.value)} />
      </label>
      <label>
        Colors
        <input value={colors} placeholder="Black, Oxblood" onChange={(e) => setColors(e.target.value)} />
      </label>
      <label>
        Vendor
        <select value={vendorId ?? ""} onChange={(e) => setVendorId(e.target.value ? Number(e.target.value) : null)}>
          <option value="">Not chosen</option>
          {vendors.map((v) => (
            <option key={v.id} value={v.id}>
              {v.name}
            </option>
          ))}
        </select>
      </label>
      <div className="actions">
        <button onClick={submit} disabled={technique.trim() === ""}>
          {submitLabel}
        </button>
        <button onClick={onCancel}>Cancel</button>
      </div>
    </div>
  );
}

export const decorationsSection: StyleSection = {
  id: "decorations",
  title: "Decorations",
  Component: DecorationsSectionView,
};
