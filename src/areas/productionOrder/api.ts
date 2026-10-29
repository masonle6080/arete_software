import { call } from "../../shell/invoke";
import type { Decimal } from "../costSheet/api";

/** Mirrors `ProductionOrderStatus` in `src-tauri/src/production_order/status.rs`. */
export type ProductionOrderStatus = "draft" | "placed" | "inProduction" | "shipped" | "received" | "cancelled";

export const orderStatusLabel: Record<ProductionOrderStatus, string> = {
  draft: "Draft",
  placed: "Placed",
  inProduction: "In Production",
  shipped: "Shipped",
  received: "Received",
  cancelled: "Cancelled",
};

/** Mirrors `SizeQuantity`. */
export interface SizeQuantity {
  size: string;
  quantity: number;
}

/** Mirrors `OrderStyle`: one Style on an order. Money is in the order's currency. */
export interface OrderStyle {
  styleId: number;
  name: string;
  color: string;
  /** Sizes with a quantity, in the Style's size order. */
  quantities: SizeQuantity[];
  /** Pieces of this Style across its sizes. Tier selection uses it. */
  quantity: number;
  tierQuantity: number | null;
  unitCost: Decimal | null;
  cost: Decimal | null;
}

/** Mirrors `ProductionOrder` in `src-tauri/src/production_order/mod.rs`. */
export interface ProductionOrder {
  id: number;
  dropId: number;
  vendorId: number;
  vendorName: string;
  status: ProductionOrderStatus;
  /** Statuses the lifecycle allows next. The core owns this rule. */
  moves: ProductionOrderStatus[];
  /** `YYYY-MM-DD`. */
  deliveryDate: string | null;
  styles: OrderStyle[];
  totalQuantity: number;
  /** True once Placed. While false, costs are an estimate from the current Cost Sheets. */
  costsFrozen: boolean;
  currency: string | null;
  totalCost: Decimal | null;
}

/** IPC wrappers for `src-tauri/src/ipc/production_order.rs`. */
export const productionOrderApi = {
  createProductionOrder: (dropId: number, vendorId: number) =>
    call<ProductionOrder>("create_production_order", { dropId, vendorId }),
  getProductionOrder: (id: number) => call<ProductionOrder>("get_production_order", { id }),
  dropProductionOrders: (dropId: number) => call<ProductionOrder[]>("drop_production_orders", { dropId }),
  /** Zero removes the size. Draft orders only. */
  setOrderQuantity: (id: number, styleId: number, size: string, quantity: number) =>
    call<ProductionOrder>("set_order_quantity", { id, styleId, size, quantity }),
  /** `date` is `YYYY-MM-DD`; null clears it. */
  setDeliveryDate: (id: number, date: string | null) => call<ProductionOrder>("set_delivery_date", { id, date }),
  /** Moving to Placed freezes unit costs in the core, or is refused. */
  moveProductionOrder: (id: number, to: ProductionOrderStatus) =>
    call<ProductionOrder>("move_production_order", { id, to }),
};
