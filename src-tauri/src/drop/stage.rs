//! Drop stage and its lifecycle rules.
//!
//! Design → Sampling → Production → Released, plus Cancelled from any stage
//! before Released. Some stages cascade onto the Thesis
//! (see [`DropStage::thesis_cascade`]).

use crate::lifecycle::status_enum;
use crate::thesis::ThesisStatus;

status_enum! {
    pub enum DropStage for "Drop" {
        Design = ("Design", "design"),
        Sampling = ("Sampling", "sampling"),
        Production = ("Production", "production"),
        Released = ("Released", "released"),
        Cancelled = ("Cancelled", "cancelled"),
    }
}

impl DropStage {
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
}
