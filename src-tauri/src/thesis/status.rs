//! Thesis status and its lifecycle rules.
//!
//! Seed → Developing → Committed → Released → Archived. Abandoned is reached
//! from Seed or Developing. Only some moves are manual; the rest are cascades
//! from the Drop (see [`ThesisStatus::manual_move_allowed`]).

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThesisStatus {
    Seed,
    Developing,
    Committed,
    Released,
    Archived,
    Abandoned,
}

impl ThesisStatus {
    /// Every status, in lifecycle order. The grouped list uses this order.
    pub const ALL: [ThesisStatus; 6] = [
        ThesisStatus::Seed,
        ThesisStatus::Developing,
        ThesisStatus::Committed,
        ThesisStatus::Released,
        ThesisStatus::Archived,
        ThesisStatus::Abandoned,
    ];

    /// Whether the founder may move a Thesis from `self` to `to` by hand.
    /// Committed, Released, and the return to Developing come only from the
    /// Drop, so they are not manual moves.
    pub fn manual_move_allowed(self, to: ThesisStatus) -> bool {
        use ThesisStatus::*;
        matches!(
            (self, to),
            (Seed, Developing) | (Seed, Abandoned) | (Developing, Abandoned) | (Released, Archived)
        )
    }

    /// Every status the founder may move to by hand from `self`.
    pub fn manual_moves(self) -> Vec<ThesisStatus> {
        ThesisStatus::ALL.into_iter().filter(|to| self.manual_move_allowed(*to)).collect()
    }

    /// The founder-facing name, for example "Seed".
    pub fn label(self) -> &'static str {
        match self {
            ThesisStatus::Seed => "Seed",
            ThesisStatus::Developing => "Developing",
            ThesisStatus::Committed => "Committed",
            ThesisStatus::Released => "Released",
            ThesisStatus::Archived => "Archived",
            ThesisStatus::Abandoned => "Abandoned",
        }
    }

    pub(crate) fn as_db(self) -> &'static str {
        match self {
            ThesisStatus::Seed => "seed",
            ThesisStatus::Developing => "developing",
            ThesisStatus::Committed => "committed",
            ThesisStatus::Released => "released",
            ThesisStatus::Archived => "archived",
            ThesisStatus::Abandoned => "abandoned",
        }
    }

    pub(crate) fn from_db(s: &str) -> Option<ThesisStatus> {
        ThesisStatus::ALL.into_iter().find(|st| st.as_db() == s)
    }
}

impl fmt::Display for ThesisStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}
