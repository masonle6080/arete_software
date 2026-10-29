import { useCallback, useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import { blankApi, blankLabel, formatCost, type Blank } from "../../blank/api";
import { styleApi } from "../api";
import type { StyleBase } from "../types";
import type { StyleSection, StyleSectionProps } from ".";

/**
 * The Style's base and its sizes. A base is a Blank or a Tech Pack. A
 * Blank-based Style picks its sizes from the Blank's sizes; a Tech Pack
 * Style takes them from its Point of Measure table (edited in the Tech Pack
 * section). The core owns these rules; this view only offers the choices
 * and shows refusals.
 */
function BaseSectionView({ style, onStyleChanged }: StyleSectionProps) {
  const [base, setBase] = useState<StyleBase | null>(null);
  const [sizes, setSizes] = useState<string[]>([]);
  const [blanks, setBlanks] = useState<Blank[]>([]);
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    Promise.all([styleApi.styleBase(style.id), styleApi.styleSizes(style.id), blankApi.listBlanks()]).then(
      ([b, s, all]) => {
        setBase(b);
        setSizes(s);
        setBlanks(all);
      },
      (raw) => setError(toCoreError(raw)),
    );
    // `style` (not only its id): the Tech Pack section reports size changes
    // through `onStyleChanged`, which hands every section a fresh Style.
  }, [style]);
  useEffect(reload, [reload]);

  async function run(action: () => Promise<unknown>) {
    setError(undefined);
    try {
      await action();
    } catch (raw) {
      setError(toCoreError(raw));
    }
    reload();
  }

  const blank = base?.kind === "blank" ? blanks.find((b) => b.id === base.blankId) : undefined;

  async function chooseBase(value: string) {
    if (value === "") return;
    const next: StyleBase =
      value === "techPack" ? { kind: "techPack" } : { kind: "blank", blankId: Number(value) };
    await run(() => styleApi.setStyleBase(style.id, next));
    onStyleChanged();
  }

  function toggleSize(size: string) {
    const next = sizes.includes(size) ? sizes.filter((s) => s !== size) : [...sizes, size];
    run(() => styleApi.setStyleSizes(style.id, next));
  }

  return (
    <div className="style-base">
      {error && <p className="error">{error.message}</p>}
      <label>
        Base
        <select
          value={base?.kind === "techPack" ? "techPack" : (blank?.id ?? "")}
          onChange={(e) => chooseBase(e.target.value)}
        >
          <option value="">Choose a Blank or a Tech Pack</option>
          <option value="techPack">Tech Pack (custom)</option>
          <optgroup label={blanks.length === 0 ? "No Blanks yet: add one under Blanks" : "Blanks"}>
            {blanks.map((b) => (
              <option key={b.id} value={b.id}>
                {blankLabel(b)} ({b.vendorName})
              </option>
            ))}
          </optgroup>
        </select>
      </label>
      {base?.kind === "techPack" && (
        <p className="empty">
          {sizes.length === 0
            ? "No sizes yet. Add size columns to the Point of Measure table in the Tech Pack."
            : `Sizes from the Tech Pack's Point of Measure table: ${sizes.join(", ")}`}
        </p>
      )}
      {blank && (
        <>
          <p className="empty">
            {blank.vendorName} · {formatCost(blank)}
            {blank.colors.length > 0 && <> · comes in {blank.colors.join(", ")}</>}
          </p>
          <fieldset className="sizes">
            <legend>Sizes</legend>
            {blank.sizes.map((size) => (
              <label key={size} className="size">
                <input type="checkbox" checked={sizes.includes(size)} onChange={() => toggleSize(size)} />
                {size}
              </label>
            ))}
          </fieldset>
        </>
      )}
      {!base && <p className="empty">No base yet.</p>}
    </div>
  );
}

export const baseSection: StyleSection = { id: "base", title: "Base and sizes", Component: BaseSectionView };
