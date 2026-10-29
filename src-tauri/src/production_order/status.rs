//! Production Order status and its lifecycle rules.
//!
//! Draft → Placed → In Production → Shipped → Received, plus Cancelled from
//! any status before Shipped. Placing freezes unit costs (see
//! `production_order::move_production_order`).

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProductionOrderStatus {
    Draft,
    Placed,
    InProduction,
    Shipped,
    Received,
    Cancelled,
}

impl ProductionOrderStatus {
    /// Every status, in lifecycle order.
    pub const ALL: [ProductionOrderStatus; 6] = [
        ProductionOrderStatus::Draft,
        ProductionOrderStatus::Placed,
        ProductionOrderStatus::InProduction,
        ProductionOrderStatus::Shipped,
        ProductionOrderStatus::Received,
        ProductionOrderStatus::Cancelled,
    ];

    /// Whether the lifecycle lets an order move from `self` to `to`. The
    /// guard on Placed (every Style has a Quantity Tier) lives in
    /// `production_order/mod.rs`.
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

    /// Every status the lifecycle lets an order move to from `self`.
    pub fn moves(self) -> Vec<ProductionOrderStatus> {
        ProductionOrderStatus::ALL.into_iter().filter(|to| self.move_allowed(*to)).collect()
    }

    /// The founder-facing name, for example "In Production".
    pub fn label(self) -> &'static str {
        match self {
            ProductionOrderStatus::Draft => "Draft",
            ProductionOrderStatus::Placed => "Placed",
            ProductionOrderStatus::InProduction => "In Production",
            ProductionOrderStatus::Shipped => "Shipped",
            ProductionOrderStatus::Received => "Received",
            ProductionOrderStatus::Cancelled => "Cancelled",
        }
    }

    pub(crate) fn as_db(self) -> &'static str {
        match self {
            ProductionOrderStatus::Draft => "draft",
            ProductionOrderStatus::Placed => "placed",
            ProductionOrderStatus::InProduction => "in_production",
            ProductionOrderStatus::Shipped => "shipped",
            ProductionOrderStatus::Received => "received",
            ProductionOrderStatus::Cancelled => "cancelled",
        }
    }

    pub(crate) fn from_db(s: &str) -> Option<ProductionOrderStatus> {
        ProductionOrderStatus::ALL.into_iter().find(|st| st.as_db() == s)
    }
}

impl fmt::Display for ProductionOrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}
