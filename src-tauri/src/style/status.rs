//! Style status and its lifecycle rules.
//!
//! Draft → Sampling → Approved, plus Cut from any status. Cut is final, and
//! a Cut Style stays visible on its Drop.

use crate::lifecycle::status_enum;

status_enum! {
    pub enum StyleStatus for "Style" {
        Draft = ("Draft", "draft"),
        Sampling = ("Sampling", "sampling"),
        Approved = ("Approved", "approved"),
        Cut = ("Cut", "cut"),
    }
}

impl StyleStatus {
    /// Whether the lifecycle lets a Style move from `self` to `to`.
    pub fn move_allowed(self, to: StyleStatus) -> bool {
        use StyleStatus::*;
        matches!((self, to), (Draft, Sampling) | (Sampling, Approved) | (Draft | Sampling | Approved, Cut))
    }

    /// Whether a Style in this status lets its Drop move to Production.
    pub fn ready_for_production(self) -> bool {
        matches!(self, StyleStatus::Approved | StyleStatus::Cut)
    }
}
