import { useEffect, useState } from "react";
import type { Area } from "../../shell/area";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { roleLabel, vendorApi, VENDOR_ROLES, type Vendor, type VendorInput, type VendorRole } from "./api";
import "./vendor.css";

/** `null` id: a new Vendor. */
type Editing = { id: number | null; input: VendorInput };

function VendorScreen() {
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [editing, setEditing] = useState<Editing | null>(null);
  const [error, setError] = useState<CoreError | undefined>();

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
      if (editing.id === null) await vendorApi.createVendor(editing.input);
      else await vendorApi.updateVendor(editing.id, editing.input);
      setEditing(null);
      await reload();
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  function toggleRole(role: VendorRole) {
    if (!editing) return;
    const roles = editing.input.roles.includes(role)
      ? editing.input.roles.filter((r) => r !== role)
      : [...editing.input.roles, role];
    setEditing({ ...editing, input: { ...editing.input, roles } });
  }

  return (
    <section className="vendors">
      <header>
        <h1>Vendors</h1>
        <button onClick={() => { setError(undefined); setEditing({ id: null, input: { name: "", roles: [] } }); }}>
          New Vendor
        </button>
      </header>

      {editing && (
        <form className="vendor-form" onSubmit={(e) => { e.preventDefault(); save(); }}>
          <label>
            Name
            <input
              autoFocus
              value={editing.input.name}
              onChange={(e) => setEditing({ ...editing, input: { ...editing.input, name: e.target.value } })}
            />
          </label>
          <fieldset>
            <legend>Roles</legend>
            {VENDOR_ROLES.map(({ role, label }) => (
              <label key={role} className="role">
                <input type="checkbox" checked={editing.input.roles.includes(role)} onChange={() => toggleRole(role)} />
                {label}
              </label>
            ))}
          </fieldset>
          <div className="actions">
            <button type="submit">{editing.id === null ? "Create" : "Save"}</button>
            <button type="button" onClick={() => { setError(undefined); setEditing(null); }}>
              Cancel
            </button>
          </div>
        </form>
      )}

      {error && <p className="error">{error.message}</p>}

      {vendors.length === 0 ? (
        <p className="muted">No Vendors yet.</p>
      ) : (
        <ul className="vendor-list">
          {vendors.map((v) => (
            <li key={v.id}>
              <span className="name">{v.name}</span>
              <span className="muted">{v.roles.map(roleLabel).join(", ")}</span>
              <button
                onClick={() => {
                  setError(undefined);
                  setEditing({ id: v.id, input: { name: v.name, roles: v.roles } });
                }}
              >
                Edit
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

export const vendorArea: Area = { id: "vendor", label: "Vendors", Screen: VendorScreen };
