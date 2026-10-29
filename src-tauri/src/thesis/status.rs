//! Thesis status and its lifecycle rules.
//!
//! Seed → Developing → Committed → Released → Archived. Abandoned is reached
//! from Seed or Developing. Only some moves are manual; the rest are cascades
//! from the Drop (see [`ThesisStatus::move_allowed`]).

use crate::lifecycle::status_enum;

status_enum! {
    /// The grouped Thesis list uses the lifecycle order of `ALL`.
    pub enum ThesisStatus for "Thesis" {
        Seed = ("Seed", "seed"),
        Developing = ("Developing", "developing"),
        Committed = ("Committed", "committed"),
        Released = ("Released", "released"),
        Archived = ("Archived", "archived"),
        Abandoned = ("Abandoned", "abandoned"),
    }
}

impl ThesisStatus {
    /// Whether the founder may move a Thesis from `self` to `to` by hand.
    /// Committed, Released, and the return to Developing come only from the
    /// Drop (`set_status_from_drop`), so they are not manual moves.
    pub fn move_allowed(self, to: ThesisStatus) -> bool {
        use ThesisStatus::*;
        matches!(
            (self, to),
            (Seed, Developing) | (Seed, Abandoned) | (Developing, Abandoned) | (Released, Archived)
        )
    }

    /// Whether a new Drop may be made from a Thesis in this status. Creating
    /// the Drop moves the Thesis to Committed.
    pub fn can_take_drop(self) -> bool {
        matches!(self, ThesisStatus::Seed | ThesisStatus::Developing)
    }
}
