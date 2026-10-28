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

export interface Vendor {
  id: number;
  name: string;
  roles: VendorRole[];
}

export interface VendorInput {
  name: string;
  roles: VendorRole[];
}

/** IPC wrappers for `src-tauri/src/ipc/vendor.rs`. */
export const vendorApi = {
  listVendors: () => call<Vendor[]>("list_vendors"),
  createVendor: (input: VendorInput) => call<Vendor>("create_vendor", { input }),
  updateVendor: (id: number, input: VendorInput) => call<Vendor>("update_vendor", { id, input }),
};
