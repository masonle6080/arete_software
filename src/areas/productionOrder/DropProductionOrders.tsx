import { useCallback, useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import type { Drop } from "../drop/types";
import { NotesPanel } from "../note/NotesPanel";
import { styleApi } from "../style/api";
import type { Style } from "../style/types";
import { vendorApi, type Vendor } from "../vendor/api";
import { orderStatusLabel, productionOrderApi, type ProductionOrder, type ProductionOrderStatus } from "./api";
import "./productionOrder.css";

/**
 * The Drop's production overview: every Production Order with its Vendor,
 * status, delivery date, and total cost, a form to add one, and the open
 * order's details. The core owns every rule and computes every cost.
 */
export function DropProductionOrders({ drop }: { drop: Drop }) {
  const [orders, setOrders] = useState<ProductionOrder[]>([]);
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [vendorId, setVendorId] = useState<number>();
  const [openId, setOpenId] = useState<number>();
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    productionOrderApi.dropProductionOrders(drop.id).then(setOrders, (raw) => setError(toCoreError(raw)));
  }, [drop.id]);
  useEffect(reload, [reload]);
  useEffect(() => {
    vendorApi.listVendors().then(setVendors, (raw) => setError(toCoreError(raw)));
  }, []);

  async function add(e: FormEvent) {
    e.preventDefault();
    if (vendorId === undefined) return;
    setError(undefined);
    try {
      const order = await productionOrderApi.createProductionOrder(drop.id, vendorId);
      setOpenId(order.id);
    } catch (raw) {
      setError(toCoreError(raw));
    }
    reload();
  }

  const open = orders.find((o) => o.id === openId);

  return (
    <div className="drop-orders">
      <h3>Production Orders</h3>
      {error && <p className="error">{error.message}</p>}
      {orders.length === 0 ? (
        <p className="empty">No Production Orders yet.</p>
      ) : (
        <table className="order-table">
          <thead>
            <tr>
              <th>Vendor</th>
              <th>Status</th>
              <th>Delivery</th>
              <th className="num">Total</th>
            </tr>
          </thead>
          <tbody>
            {orders.map((o) => (
              <tr
                key={o.id}
                className={o.id === openId ? "open" : undefined}
                onClick={() => setOpenId(o.id === openId ? undefined : o.id)}
              >
                <td>{o.vendorName}</td>
                <td>{orderStatusLabel[o.status]}</td>
                <td>{o.deliveryDate ?? "—"}</td>
                <td className="num">
                  <Money amount={o.totalCost} currency={o.currency} estimate={!o.costsFrozen} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {drop.stage !== "cancelled" && (
        <form className="order-create" onSubmit={add}>
          <select value={vendorId ?? ""} onChange={(e) => setVendorId(e.target.value ? Number(e.target.value) : undefined)}>
            <option value="">Choose a Vendor</option>
            {vendors.map((v) => (
              <option key={v.id} value={v.id}>
                {v.name}
              </option>
            ))}
          </select>
          <button type="submit" disabled={vendorId === undefined}>
            Add Production Order
          </button>
        </form>
      )}
      {open && <OrderDetail key={open.id} order={open} onChanged={reload} />}
    </div>
  );
}

function Money({ amount, currency, estimate }: { amount: string | null; currency: string | null; estimate: boolean }) {
  if (amount === null) return <span className="muted">—</span>;
  return (
    <span title={estimate ? "Estimate from the current Cost Sheets" : "Frozen when placed"}>
      {estimate && "≈ "}
      {amount} {currency ?? ""}
    </span>
  );
}

function OrderDetail({ order, onChanged }: { order: ProductionOrder; onChanged(): void }) {
  const [styles, setStyles] = useState<Style[]>([]);
  const [sizes, setSizes] = useState<Record<number, string[]>>({});
  const [error, setError] = useState<CoreError>();
  const [confirmCancel, setConfirmCancel] = useState(false);
  const draft = order.status === "draft";

  useEffect(() => {
    if (!draft) return;
    styleApi.dropStyles(order.dropId).then(async (all) => {
      const live = all.filter((s) => s.status !== "cut");
      const entries = await Promise.all(live.map(async (s) => [s.id, await styleApi.styleSizes(s.id)] as const));
      setStyles(live);
      setSizes(Object.fromEntries(entries));
    }, (raw) => setError(toCoreError(raw)));
  }, [order.dropId, draft]);

  async function run(action: () => Promise<unknown>) {
    setError(undefined);
    try {
      await action();
    } catch (raw) {
      setError(toCoreError(raw));
    }
    onChanged();
  }

  function move(to: ProductionOrderStatus) {
    setConfirmCancel(false);
    run(() => productionOrderApi.moveProductionOrder(order.id, to));
  }

  function quantityOf(styleId: number, size: string): number {
    const style = order.styles.find((s) => s.styleId === styleId);
    return style?.quantities.find((q) => q.size === size)?.quantity ?? 0;
  }

  function saveQuantity(styleId: number, size: string, text: string) {
    const quantity = text.trim() === "" ? 0 : Number(text);
    if (!Number.isInteger(quantity) || quantity < 0) {
      setError({ kind: "InvalidInput", message: "a quantity is a whole number of zero or more" });
      return;
    }
    if (quantity === quantityOf(styleId, size)) return;
    run(() => productionOrderApi.setOrderQuantity(order.id, styleId, size, quantity));
  }

  return (
    <div className="order-detail">
      <p className="status">{orderStatusLabel[order.status]}</p>
      <strong>{order.vendorName}</strong>
      {error && <p className="error">{error.message}</p>}

      {order.moves.length > 0 && (
        <div className="actions">
          {order.moves
            .filter((to) => to !== "cancelled")
            .map((to) => (
              <button key={to} onClick={() => move(to)}>
                Move to {orderStatusLabel[to]}
              </button>
            ))}
          {order.moves.includes("cancelled") &&
            (confirmCancel ? (
              <button onClick={() => move("cancelled")}>Confirm: cancel this order</button>
            ) : (
              <button onClick={() => setConfirmCancel(true)}>Cancel order</button>
            ))}
        </div>
      )}

      <label className="order-date">
        Delivery date{" "}
        <input
          type="date"
          value={order.deliveryDate ?? ""}
          onChange={(e) => run(() => productionOrderApi.setDeliveryDate(order.id, e.target.value || null))}
        />
      </label>

      {draft && (
        <div className="order-grid">
          {styles.length === 0 && <p className="empty">The Drop has no Styles to order.</p>}
          {styles.map((s) => (
            <div key={s.id} className="order-grid-style">
              <span>
                {s.name} <span className="muted">· {s.color}</span>
              </span>
              {(sizes[s.id] ?? []).length === 0 ? (
                <span className="muted">Set the Style's base and sizes first.</span>
              ) : (
                <div className="order-sizes">
                  {(sizes[s.id] ?? []).map((size) => {
                    const current = quantityOf(s.id, size);
                    return (
                      <label key={`${size}-${current}`}>
                        {size}
                        <input
                          type="number"
                          min={0}
                          step={1}
                          defaultValue={current || ""}
                          onBlur={(e) => saveQuantity(s.id, size, e.target.value)}
                        />
                      </label>
                    );
                  })}
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {order.styles.length > 0 && (
        <table className="order-costs">
          <thead>
            <tr>
              <th>Style</th>
              {!draft && <th>Sizes</th>}
              <th className="num">Pieces</th>
              <th className="num">Tier</th>
              <th className="num">Unit cost</th>
              <th className="num">Cost</th>
            </tr>
          </thead>
          <tbody>
            {order.styles.map((s) => (
              <tr key={s.styleId}>
                <td>
                  {s.name} <span className="muted">· {s.color}</span>
                </td>
                {!draft && <td>{s.quantities.map((q) => `${q.size} ${q.quantity}`).join(", ")}</td>}
                <td className="num">{s.quantity}</td>
                <td className="num">{s.tierQuantity ?? "—"}</td>
                <td className="num">{s.unitCost ?? "—"}</td>
                <td className="num">{s.cost ?? "—"}</td>
              </tr>
            ))}
          </tbody>
          <tfoot>
            <tr>
              <td colSpan={draft ? 1 : 2}>Total</td>
              <td className="num">{order.totalQuantity}</td>
              <td colSpan={2} />
              <td className="num">
                <Money amount={order.totalCost} currency={order.currency} estimate={!order.costsFrozen} />
              </td>
            </tr>
          </tfoot>
        </table>
      )}
      {draft && order.styles.length > 0 && (
        <p className="muted">Costs are an estimate. Placing the order freezes them from the Cost Sheets.</p>
      )}

      <NotesPanel owner={{ kind: "productionOrder", id: order.id }} />
    </div>
  );
}
