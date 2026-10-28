import { useCallback, useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import { blankApi, blankLabel, formatCost, type Blank } from "../../blank/api";
import { styleApi } from "../api";
import type { StyleBase } from "../types";
import type { StyleSection, StyleSectionProps } from ".";

/**
 * The Style's base and its sizes. A Blank-based Style picks its sizes from
 * the Blank's sizes. The core owns both rules; this view only offers the
 * choices and shows refusals.
 */
function BaseSectionView({ style }: StyleSectionProps) {
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
  }, [style.id]);
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

  function chooseBlank(value: string) {
    if (value === "") return;
    run(() => styleApi.setStyleBase(style.id, { kind: "blank", blankId: Number(value) }));
  }

  function toggleSize(size: string) {
    const next = sizes.includes(size) ? sizes.filter((s) => s !== size) : [...sizes, size];
    run(() => styleApi.setStyleSizes(style.id, next));
  }

  return (
    <div className="style-base">
      {error && <p className="error">{error.message}</p>}
      <label>
        Blank
        <select value={blank?.id ?? ""} onChange={(e) => chooseBlank(e.target.value)}>
          <option value="">{blanks.length === 0 ? "No Blanks yet: add one under Blanks" : "Choose a Blank"}</option>
          {blanks.map((b) => (
            <option key={b.id} value={b.id}>
              {blankLabel(b)} ({b.vendorName})
            </option>
          ))}
        </select>
      </label>
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
