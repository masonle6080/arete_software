import { call } from "../../shell/invoke";

/** Mirrors `Blank` in `src-tauri/src/blank/mod.rs`. */
export interface Blank {
  id: number;
  brand: string;
  model: string;
  vendorId: number;
  /** The Vendor's name, for display. */
  vendorName: string;
  colors: string[];
  sizes: string[];
  /** Unit cost in minor units of `currency` (cents); `null` when unknown. */
  costCents: number | null;
  /** Free text such as "USD". */
  currency: string;
  notes: string;
}

export type BlankInput = Omit<Blank, "id" | "vendorName">;

export function blankLabel(b: Pick<Blank, "brand" | "model">): string {
  return `${b.brand} ${b.model}`;
}

/** "12.50 USD", or "—" when the cost is unknown. */
export function formatCost(b: Pick<Blank, "costCents" | "currency">): string {
  if (b.costCents === null) return "—";
  return `${(b.costCents / 100).toFixed(2)}${b.currency ? ` ${b.currency}` : ""}`;
}

/** IPC wrappers for `src-tauri/src/ipc/blank.rs`. */
export const blankApi = {
  listBlanks: () => call<Blank[]>("list_blanks"),
  createBlank: (input: BlankInput) => call<Blank>("create_blank", { input }),
  updateBlank: (id: number, input: BlankInput) => call<Blank>("update_blank", { id, input }),
};
