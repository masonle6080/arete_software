import { useEffect, useState } from "react";
import type { Area } from "../../shell/area";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { NotesPanel } from "../note/NotesPanel";
import {
  emptyVendorInput,
  roleLabel,
  vendorApi,
  VENDOR_ROLES,
  type Vendor,
  type VendorContact,
  type VendorInput,
  type VendorRole,
} from "./api";
import "./vendor.css";

/** `null` id: a new Vendor. */
type Editing = { id: number | null; input: VendorInput };

function toInput({ id: _id, ...input }: Vendor): VendorInput {
  return input;
}

function VendorScreen() {
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [editing, setEditing] = useState<Editing | null>(null);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [error, setError] = useState<CoreError | undefined>();

  const selected = vendors.find((v) => v.id === selectedId) ?? null;

  async function reload() {
    try {
      setVendors(await vendorApi.listVendors());
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  useEffect(() => {
    reload();
  }, []);

  async function save() {
    if (!editing) return;
    setError(undefined);
    try {
      const saved =
        editing.id === null
          ? await vendorApi.createVendor(editing.input)
          : await vendorApi.updateVendor(editing.id, editing.input);
      setEditing(null);
      setSelectedId(saved.id);
      await reload();
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  function startEdit(next: Editing | null) {
    setError(undefined);
    setEditing(next);
  }

  return (
    <section className="vendors">
      <header>
        <h1>Vendors</h1>
        <button onClick={() => startEdit({ id: null, input: emptyVendorInput() })}>New Vendor</button>
      </header>

      {editing && (
        <VendorForm
          editing={editing}
          onChange={(input) => setEditing({ ...editing, input })}
          onSave={save}
          onCancel={() => startEdit(null)}
        />
      )}

      {error && <p className="error">{error.message}</p>}

      {vendors.length === 0 ? (
        <p className="muted">No Vendors yet.</p>
      ) : (
        <ul className="vendor-list">
          {vendors.map((v) => (
            <li key={v.id} className={v.id === selectedId ? "selected" : undefined}>
              <button className="name link" onClick={() => setSelectedId(v.id)}>{v.name}</button>
              <span className="muted">{v.roles.map(roleLabel).join(", ")}</span>
              <button onClick={() => startEdit({ id: v.id, input: toInput(v) })}>Edit</button>
            </li>
          ))}
        </ul>
      )}

      {selected && <VendorDetail vendor={selected} />}
    </section>
  );
}

function VendorForm(props: {
  editing: Editing;
  onChange(input: VendorInput): void;
  onSave(): void;
  onCancel(): void;
}) {
  const { editing, onChange } = props;
  const input = editing.input;
  const set = (patch: Partial<VendorInput>) => onChange({ ...input, ...patch });

  function toggleRole(role: VendorRole) {
    set({ roles: input.roles.includes(role) ? input.roles.filter((r) => r !== role) : [...input.roles, role] });
  }

  function setContact(i: number, patch: Partial<VendorContact>) {
    set({ contacts: input.contacts.map((c, j) => (j === i ? { ...c, ...patch } : c)) });
  }

  return (
    <form className="vendor-form" onSubmit={(e) => { e.preventDefault(); props.onSave(); }}>
      <label>
        Name
        <input autoFocus value={input.name} onChange={(e) => set({ name: e.target.value })} />
      </label>
      <fieldset>
        <legend>Roles</legend>
        {VENDOR_ROLES.map(({ role, label }) => (
          <label key={role} className="role">
            <input type="checkbox" checked={input.roles.includes(role)} onChange={() => toggleRole(role)} />
            {label}
          </label>
        ))}
      </fieldset>
      <label>
        Location
        <input value={input.location} onChange={(e) => set({ location: e.target.value })} />
      </label>
      <label>
        Minimum order quantity (pieces)
        <input
          type="number"
          min={1}
          step={1}
          value={input.minimumOrderQuantity ?? ""}
          onChange={(e) => set({ minimumOrderQuantity: e.target.value === "" ? null : Number(e.target.value) })}
        />
      </label>
      <label>
        Typical lead time
        <input
          placeholder="for example, 4-6 weeks"
          value={input.leadTime}
          onChange={(e) => set({ leadTime: e.target.value })}
        />
      </label>
      <fieldset className="contacts">
        <legend>Contacts</legend>
        {input.contacts.map((c, i) => (
          <div key={i} className="contact-row">
            <input placeholder="Name" value={c.name} onChange={(e) => setContact(i, { name: e.target.value })} />
            <input placeholder="Role" value={c.role} onChange={(e) => setContact(i, { role: e.target.value })} />
            <input placeholder="Email" value={c.email} onChange={(e) => setContact(i, { email: e.target.value })} />
            <input placeholder="Phone" value={c.phone} onChange={(e) => setContact(i, { phone: e.target.value })} />
            <button type="button" onClick={() => set({ contacts: input.contacts.filter((_, j) => j !== i) })}>
              Remove
            </button>
          </div>
        ))}
        <button
          type="button"
          onClick={() => set({ contacts: [...input.contacts, { name: "", role: "", email: "", phone: "" }] })}
        >
          Add contact
        </button>
      </fieldset>
      <div className="actions">
        <button type="submit">{editing.id === null ? "Create" : "Save"}</button>
        <button type="button" onClick={props.onCancel}>Cancel</button>
      </div>
    </form>
  );
}

function VendorDetail({ vendor }: { vendor: Vendor }) {
  return (
    <article className="vendor-detail">
      <h2>{vendor.name}</h2>
      <dl>
        <dt>Roles</dt>
        <dd>{vendor.roles.map(roleLabel).join(", ")}</dd>
        <dt>Location</dt>
        <dd>{vendor.location || "—"}</dd>
        <dt>Minimum order</dt>
        <dd>{vendor.minimumOrderQuantity === null ? "—" : `${vendor.minimumOrderQuantity} pieces`}</dd>
        <dt>Lead time</dt>
        <dd>{vendor.leadTime || "—"}</dd>
      </dl>
      <h3>Contacts</h3>
      {vendor.contacts.length === 0 ? (
        <p className="muted">No contacts yet.</p>
      ) : (
        <ul className="contact-list">
          {vendor.contacts.map((c, i) => (
            <li key={i}>
              <span className="name">{c.name}</span>
              {[c.role, c.email, c.phone].filter(Boolean).map((t) => (
                <span key={t} className="muted">{t}</span>
              ))}
            </li>
          ))}
        </ul>
      )}
      <NotesPanel owner={{ kind: "vendor", id: vendor.id }} />
    </article>
  );
}

export const vendorArea: Area = { id: "vendor", label: "Vendors", Screen: VendorScreen };
