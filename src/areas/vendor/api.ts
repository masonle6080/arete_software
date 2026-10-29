import { call } from "../../shell/invoke";

/** A role a Vendor fills. Mirrors `VendorRole` in `src-tauri/src/vendor/mod.rs`. */
export type VendorRole = "blankSupplier" | "decorator" | "manufacturer" | "mill";

/** All roles, in display order. */
export const VENDOR_ROLES: { role: VendorRole; label: string }[] = [
  { role: "blankSupplier", label: "Blank supplier" },
  { role: "decorator", label: "Decorator" },
  { role: "manufacturer", label: "Manufacturer" },
  { role: "mill", label: "Mill" },
];

export function roleLabel(role: VendorRole): string {
  return VENDOR_ROLES.find((r) => r.role === role)?.label ?? role;
}

/** A contact person at a Vendor. Only `name` is required; the rest may be "". */
export interface VendorContact {
  name: string;
  role: string;
  email: string;
  phone: string;
}

export interface Vendor {
  id: number;
  name: string;
  roles: VendorRole[];
  contacts: VendorContact[];
  location: string;
  /** In pieces; `null` when unknown. */
  minimumOrderQuantity: number | null;
  /** Free text such as "4-6 weeks". */
  leadTime: string;
}

export type VendorInput = Omit<Vendor, "id">;

export function emptyVendorInput(): VendorInput {
  return { name: "", roles: [], contacts: [], location: "", minimumOrderQuantity: null, leadTime: "" };
}

/** IPC wrappers for `src-tauri/src/ipc/vendor.rs`. */
export const vendorApi = {
  listVendors: () => call<Vendor[]>("list_vendors"),
  createVendor: (input: VendorInput) => call<Vendor>("create_vendor", { input }),
  updateVendor: (id: number, input: VendorInput) => call<Vendor>("update_vendor", { id, input }),
};
