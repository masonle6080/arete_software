//! Drop stage and its lifecycle rules.
//!
//! Design → Sampling → Production → Released, plus Cancelled from any stage
//! before Released. Some stages cascade onto the Thesis
//! (see [`DropStage::thesis_cascade`]).

use std::fmt;

use crate::thesis::ThesisStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DropStage {
    Design,
    Sampling,
    Production,
    Released,
    Cancelled,
}

impl DropStage {
    /// Every stage, in lifecycle order.
    pub const ALL: [DropStage; 5] =
        [DropStage::Design, DropStage::Sampling, DropStage::Production, DropStage::Released, DropStage::Cancelled];

    /// Whether the lifecycle lets a Drop move from `self` to `to`. Stage
    /// guards that need more than the two stages (for example, Style
    /// statuses) live in `guard_move` in `drop/mod.rs`.
    pub fn move_allowed(self, to: DropStage) -> bool {
        use DropStage::*;
        matches!(
            (self, to),
            (Design, Sampling)
                | (Sampling, Production)
                | (Production, Released)
                | (Design, Cancelled)
                | (Sampling, Cancelled)
                | (Production, Cancelled)
        )
    }

    /// Every stage the lifecycle lets a Drop move to from `self`.
    pub fn moves(self) -> Vec<DropStage> {
        DropStage::ALL.into_iter().filter(|to| self.move_allowed(*to)).collect()
    }

    /// Whether a Drop in this stage takes new or duplicated Styles. Only
    /// Design and Sampling do: from Production on, every Style must already
    /// be Approved or Cut.
    pub fn takes_new_styles(self) -> bool {
        matches!(self, DropStage::Design | DropStage::Sampling)
    }

    /// The Thesis status that reaching this stage sets, if any. Releasing the
    /// Drop releases the Thesis; cancelling it returns the Thesis to
    /// Developing so it can get a new Drop.
    pub fn thesis_cascade(self) -> Option<ThesisStatus> {
        match self {
            DropStage::Released => Some(ThesisStatus::Released),
            DropStage::Cancelled => Some(ThesisStatus::Developing),
            DropStage::Design | DropStage::Sampling | DropStage::Production => None,
        }
    }

    /// The founder-facing name, for example "Sampling".
    pub fn label(self) -> &'static str {
        match self {
            DropStage::Design => "Design",
            DropStage::Sampling => "Sampling",
            DropStage::Production => "Production",
            DropStage::Released => "Released",
            DropStage::Cancelled => "Cancelled",
        }
    }

    pub(crate) fn as_db(self) -> &'static str {
        match self {
            DropStage::Design => "design",
            DropStage::Sampling => "sampling",
            DropStage::Production => "production",
            DropStage::Released => "released",
            DropStage::Cancelled => "cancelled",
        }
    }

    pub(crate) fn from_db(s: &str) -> Option<DropStage> {
        DropStage::ALL.into_iter().find(|st| st.as_db() == s)
    }
}

impl fmt::Display for DropStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}
