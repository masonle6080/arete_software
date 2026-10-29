//! Production Order status and its lifecycle rules.
//!
//! Draft → Placed → In Production → Shipped → Received, plus Cancelled from
//! any status before Shipped. Placing freezes unit costs (see
//! `production_order::move_production_order`).

use crate::lifecycle::status_enum;

status_enum! {
    pub enum ProductionOrderStatus for "Production Order" {
        Draft = ("Draft", "draft"),
        Placed = ("Placed", "placed"),
        InProduction = ("In Production", "in_production"),
        Shipped = ("Shipped", "shipped"),
        Received = ("Received", "received"),
        Cancelled = ("Cancelled", "cancelled"),
    }
}

impl ProductionOrderStatus {
    /// Whether the lifecycle lets an order move from `self` to `to`. The
    /// guards on Placed (every Style Approved and with a Quantity Tier) live
    /// in `production_order/mod.rs`.
    pub fn move_allowed(self, to: ProductionOrderStatus) -> bool {
        use ProductionOrderStatus::*;
        matches!(
            (self, to),
            (Draft, Placed)
                | (Placed, InProduction)
                | (InProduction, Shipped)
                | (Shipped, Received)
                | (Draft, Cancelled)
                | (Placed, Cancelled)
                | (InProduction, Cancelled)
        )
    }
}
