//! Thesis status and its lifecycle rules.
//!
//! A Thesis is its Drop, so one lifecycle covers the idea and its release:
//! Seed → Developing → Design → Sampling → Production → Released →
//! Archived. Abandoned is reached from any status before Released. Every
//! move is manual. The guards that need more than the two statuses (Style
//! statuses, the Documentary) live in `guard_move` in `thesis/mod.rs`.

use crate::lifecycle::status_enum;

status_enum! {
    /// The grouped Thesis list uses the lifecycle order of `ALL`.
    pub enum ThesisStatus for "Thesis" {
        Seed = ("Seed", "seed"),
        Developing = ("Developing", "developing"),
        Design = ("Design", "design"),
        Sampling = ("Sampling", "sampling"),
        Production = ("Production", "production"),
        Released = ("Released", "released"),
        Archived = ("Archived", "archived"),
        Abandoned = ("Abandoned", "abandoned"),
    }
}

impl ThesisStatus {
    /// Whether the founder may move a Thesis from `self` to `to`.
    pub fn move_allowed(self, to: ThesisStatus) -> bool {
        use ThesisStatus::*;
        matches!(
            (self, to),
            (Seed, Developing)
                | (Developing, Design)
                | (Design, Sampling)
                | (Sampling, Production)
                | (Production, Released)
                | (Released, Archived)
                | (Seed | Developing | Design | Sampling | Production, Abandoned)
        )
    }

    /// Whether a Thesis in this status takes new or duplicated Styles. Only
    /// Developing, Design and Sampling do: from Production on, the line is
    /// fixed.
    pub fn takes_new_styles(self) -> bool {
        matches!(self, ThesisStatus::Developing | ThesisStatus::Design | ThesisStatus::Sampling)
    }
}
