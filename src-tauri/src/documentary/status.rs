//! Documentary status and its lifecycle rules.
//!
//! Planning → Filming → Editing → Released, plus Abandoned from any status
//! before Released.

use crate::lifecycle::status_enum;

status_enum! {
    pub enum DocumentaryStatus for "Documentary" {
        Planning = ("Planning", "planning"),
        Filming = ("Filming", "filming"),
        Editing = ("Editing", "editing"),
        Released = ("Released", "released"),
        Abandoned = ("Abandoned", "abandoned"),
    }
}

impl DocumentaryStatus {
    /// Whether the lifecycle lets a Documentary move from `self` to `to`.
    pub fn move_allowed(self, to: DocumentaryStatus) -> bool {
        use DocumentaryStatus::*;
        matches!(
            (self, to),
            (Planning, Filming)
                | (Filming, Editing)
                | (Editing, Released)
                | (Planning, Abandoned)
                | (Filming, Abandoned)
                | (Editing, Abandoned)
        )
    }
}
