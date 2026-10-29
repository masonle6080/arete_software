import { call } from "../../shell/invoke";

/** What a line pays for. Mirrors `CostCategory` in `src-tauri/src/cost_sheet/mod.rs`. */
export type CostCategory = "base" | "decoration" | "trims" | "labels" | "packaging" | "shipping" | "duties" | "other";

/** All categories, in display order. */
export const COST_CATEGORIES: { category: CostCategory; label: string }[] = [
  { category: "base", label: "Base" },
  { category: "decoration", label: "Decoration" },
  { category: "trims", label: "Trims" },
  { category: "labels", label: "Labels" },
  { category: "packaging", label: "Packaging" },
  { category: "shipping", label: "Shipping" },
  { category: "duties", label: "Duties" },
  { category: "other", label: "Other" },
];

export function categoryLabel(category: CostCategory): string {
  return COST_CATEGORIES.find((c) => c.category === category)?.label ?? category;
}

/**
 * Money crosses IPC as exact decimal strings (for example "12.50"). The UI
 * shows them as they come and never does arithmetic on them: the core
 * computes every total.
 */
export type Decimal = string;

export interface QuantityTier {
  id: number;
  /** In pieces. */
  quantity: number;
}

export interface CostLine {
  id: number;
  category: CostCategory;
  vendorId: number | null;
  vendorName: string | null;
  description: string;
  /** The quote currency, for example "EUR". */
  currency: string;
  /** One per tier, in tier order, in `currency`. `null` is a blank cell. */
  unitCosts: (Decimal | null)[];
}

export interface TierTotal {
  tierId: number;
  quantity: number;
  /** In the home currency, rounded to cents; `null` while a rate is missing. */
  total: Decimal | null;
}

/** Mirrors `CostSheet`. */
export interface CostSheet {
  styleId: number;
  homeCurrency: string | null;
  tiers: QuantityTier[];
  lines: CostLine[];
  totals: TierTotal[];
  /** Line currencies with no exchange rate. */
  missingRates: string[];
}

export interface CostLineInput {
  category: CostCategory;
  vendorId: number | null;
  description: string;
  currency: string;
}

export interface CurrencyRate {
  currency: string;
  /** Home-currency units per one unit of `currency`. */
  rate: Decimal;
}

export interface CurrencySettings {
  homeCurrency: string | null;
  rates: CurrencyRate[];
}

/** IPC wrappers for `src-tauri/src/ipc/cost_sheet.rs`. Each edit returns the whole sheet. */
export const costSheetApi = {
  ofStyle: (styleId: number) => call<CostSheet>("cost_sheet", { styleId }),
  addTier: (styleId: number, quantity: number) => call<CostSheet>("add_quantity_tier", { styleId, quantity }),
  setTier: (tierId: number, quantity: number) => call<CostSheet>("set_quantity_tier", { tierId, quantity }),
  removeTier: (tierId: number) => call<CostSheet>("remove_quantity_tier", { tierId }),
  addLine: (styleId: number, input: CostLineInput) => call<CostSheet>("add_cost_line", { styleId, input }),
  updateLine: (lineId: number, input: CostLineInput) => call<CostSheet>("update_cost_line", { lineId, input }),
  removeLine: (lineId: number) => call<CostSheet>("remove_cost_line", { lineId }),
  /** Blank text clears the cell. */
  setUnitCost: (lineId: number, tierId: number, unitCost: string) =>
    call<CostSheet>("set_unit_cost", { lineId, tierId, unitCost }),
};

/** IPC wrappers for the library's currency settings. */
export const currencyApi = {
  settings: () => call<CurrencySettings>("currency_settings"),
  setHomeCurrency: (currency: string) => call<CurrencySettings>("set_home_currency", { currency }),
  setRate: (currency: string, rate: string) => call<CurrencySettings>("set_exchange_rate", { currency, rate }),
  removeRate: (currency: string) => call<CurrencySettings>("remove_exchange_rate", { currency }),
};
