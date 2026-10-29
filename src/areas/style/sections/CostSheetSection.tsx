import { useCallback, useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import {
  COST_CATEGORIES,
  costSheetApi,
  type CostCategory,
  type CostLine,
  type CostLineInput,
  type CostSheet,
} from "../../costSheet/api";
import { vendorApi, type Vendor } from "../../vendor/api";
import type { StyleSection, StyleSectionProps } from ".";
import "./costSheet.css";

function toInput(line: CostLine): CostLineInput {
  return { category: line.category, vendorId: line.vendorId, description: line.description, currency: line.currency };
}

/**
 * The Cost Sheet of the Style: Quantity Tiers as columns, one row per line,
 * and a total per tier in the home currency. Each field saves when it loses
 * focus or changes. The core computes every total.
 */
function CostSheetSectionView({ style }: StyleSectionProps) {
  const [sheet, setSheet] = useState<CostSheet>();
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [newQuantity, setNewQuantity] = useState("");
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    costSheetApi.ofStyle(style.id).then(setSheet, (raw) => setError(toCoreError(raw)));
  }, [style.id]);
  useEffect(reload, [reload]);
  useEffect(() => {
    vendorApi.listVendors().then(setVendors, (raw) => setError(toCoreError(raw)));
  }, []);

  async function run(action: () => Promise<CostSheet>): Promise<boolean> {
    setError(undefined);
    try {
      setSheet(await action());
      return true;
    } catch (raw) {
      setError(toCoreError(raw));
      // Put back what the core holds, so the field shows the saved value.
      reload();
      return false;
    }
  }

  async function addTier(e: FormEvent) {
    e.preventDefault();
    if (await run(() => costSheetApi.addTier(style.id, Number(newQuantity)))) setNewQuantity("");
  }

  function addLine() {
    const currency = sheet?.homeCurrency ?? "USD";
    run(() => costSheetApi.addLine(style.id, { category: "base", vendorId: null, description: "", currency }));
  }

  function updateLine(line: CostLine, change: Partial<CostLineInput>) {
    run(() => costSheetApi.updateLine(line.id, { ...toInput(line), ...change }));
  }

  if (!sheet) return error ? <p className="error">{error.message}</p> : null;
  const home = sheet.homeCurrency;

  return (
    <div className="cost-sheet">
      {error && <p className="error">{error.message}</p>}
      {!home && <p className="hint">Set the home currency in Settings to see totals.</p>}
      {home && sheet.missingRates.length > 0 && (
        <p className="hint">Set an exchange rate in Settings for {sheet.missingRates.join(", ")} to see totals.</p>
      )}
      <div className="cost-scroll">
        <table className="cost-table">
          <thead>
            <tr>
              <th>Category</th>
              <th>Vendor</th>
              <th>Description</th>
              <th>Currency</th>
              {sheet.tiers.map((tier) => (
                <th key={`${tier.id}-${tier.quantity}`} className="tier">
                  <input
                    className="number"
                    aria-label="Quantity Tier"
                    inputMode="numeric"
                    defaultValue={tier.quantity}
                    onBlur={(e) => {
                      const quantity = Number(e.target.value);
                      if (quantity !== tier.quantity) run(() => costSheetApi.setTier(tier.id, quantity));
                    }}
                  />
                  <span className="unit">pcs</span>
                  <button className="remove" title="Remove this tier" onClick={() => run(() => costSheetApi.removeTier(tier.id))}>
                    ×
                  </button>
                </th>
              ))}
              <th />
            </tr>
          </thead>
          <tbody>
            {sheet.lines.map((line) => (
              <tr key={line.id}>
                <td>
                  <select
                    value={line.category}
                    onChange={(e) => updateLine(line, { category: e.target.value as CostCategory })}
                  >
                    {COST_CATEGORIES.map((c) => (
                      <option key={c.category} value={c.category}>
                        {c.label}
                      </option>
                    ))}
                  </select>
                </td>
                <td>
                  <select
                    value={line.vendorId ?? ""}
                    onChange={(e) => updateLine(line, { vendorId: e.target.value ? Number(e.target.value) : null })}
                  >
                    <option value="">No Vendor</option>
                    {vendors.map((v) => (
                      <option key={v.id} value={v.id}>
                        {v.name}
                      </option>
                    ))}
                  </select>
                </td>
                <td>
                  <input
                    key={line.description}
                    defaultValue={line.description}
                    placeholder="Description"
                    onBlur={(e) => {
                      if (e.target.value.trim() !== line.description) updateLine(line, { description: e.target.value });
                    }}
                  />
                </td>
                <td>
                  <input
                    key={line.currency}
                    className="currency"
                    aria-label="Quote currency"
                    defaultValue={line.currency}
                    maxLength={3}
                    onBlur={(e) => {
                      if (e.target.value.trim().toUpperCase() !== line.currency) updateLine(line, { currency: e.target.value });
                    }}
                  />
                </td>
                {sheet.tiers.map((tier, i) => {
                  const cost = line.unitCosts[i];
                  return (
                    <td key={tier.id}>
                      <input
                        key={cost ?? ""}
                        className="number"
                        aria-label={`Unit cost at ${tier.quantity}`}
                        inputMode="decimal"
                        defaultValue={cost ?? ""}
                        placeholder="—"
                        onBlur={(e) => {
                          if (e.target.value.trim() !== (cost ?? ""))
                            run(() => costSheetApi.setUnitCost(line.id, tier.id, e.target.value));
                        }}
                      />
                    </td>
                  );
                })}
                <td>
                  <button className="remove" title="Remove this line" onClick={() => run(() => costSheetApi.removeLine(line.id))}>
                    ×
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
          {sheet.tiers.length > 0 && (
            <tfoot>
              <tr>
                <th colSpan={4}>Total per piece{home ? ` (${home})` : ""}</th>
                {sheet.totals.map((t) => (
                  <td key={t.tierId} className="total">
                    {t.total ?? "—"}
                  </td>
                ))}
                <td />
              </tr>
            </tfoot>
          )}
        </table>
      </div>
      <div className="cost-add">
        <button onClick={addLine}>Add line</button>
        <form onSubmit={addTier}>
          <input
            className="number"
            aria-label="New Quantity Tier"
            inputMode="numeric"
            placeholder="Pieces"
            value={newQuantity}
            onChange={(e) => setNewQuantity(e.target.value)}
          />
          <button type="submit" disabled={!/^\d+$/.test(newQuantity.trim())}>
            Add Quantity Tier
          </button>
        </form>
      </div>
      <p className="hint">
        Totals convert each line at the rates in Settings, then round to cents. A blank cell counts as zero.
      </p>
    </div>
  );
}

export const costSheetSection: StyleSection = {
  id: "cost-sheet",
  title: "Cost Sheet",
  Component: CostSheetSectionView,
};
