//! Style status and its lifecycle rules.
//!
//! Draft → Sampling → Approved, plus Cut from any status. Cut is final, and
//! a Cut Style stays visible on its Drop.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StyleStatus {
    Draft,
    Sampling,
    Approved,
    Cut,
}

impl StyleStatus {
    /// Every status, in lifecycle order.
    pub const ALL: [StyleStatus; 4] = [StyleStatus::Draft, StyleStatus::Sampling, StyleStatus::Approved, StyleStatus::Cut];

    /// Whether the lifecycle lets a Style move from `self` to `to`.
    pub fn move_allowed(self, to: StyleStatus) -> bool {
        use StyleStatus::*;
        matches!((self, to), (Draft, Sampling) | (Sampling, Approved) | (Draft | Sampling | Approved, Cut))
    }

    /// Every status the lifecycle lets a Style move to from `self`.
    pub fn moves(self) -> Vec<StyleStatus> {
        StyleStatus::ALL.into_iter().filter(|to| self.move_allowed(*to)).collect()
    }

    /// Whether a Style in this status lets its Drop move to Production.
    pub fn ready_for_production(self) -> bool {
        matches!(self, StyleStatus::Approved | StyleStatus::Cut)
    }

    /// The founder-facing name, for example "Approved".
    pub fn label(self) -> &'static str {
        match self {
            StyleStatus::Draft => "Draft",
            StyleStatus::Sampling => "Sampling",
            StyleStatus::Approved => "Approved",
            StyleStatus::Cut => "Cut",
        }
    }

    pub(crate) fn as_db(self) -> &'static str {
        match self {
            StyleStatus::Draft => "draft",
            StyleStatus::Sampling => "sampling",
            StyleStatus::Approved => "approved",
            StyleStatus::Cut => "cut",
        }
    }

    pub(crate) fn from_db(s: &str) -> Option<StyleStatus> {
        StyleStatus::ALL.into_iter().find(|st| st.as_db() == s)
    }
}

impl fmt::Display for StyleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}
