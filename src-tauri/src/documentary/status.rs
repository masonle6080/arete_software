//! Documentary status and its lifecycle rules.
//!
//! Planning → Filming → Editing → Released, plus Abandoned from any status
//! before Released.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DocumentaryStatus {
    Planning,
    Filming,
    Editing,
    Released,
    Abandoned,
}

impl DocumentaryStatus {
    /// Every status, in lifecycle order.
    pub const ALL: [DocumentaryStatus; 5] = [
        DocumentaryStatus::Planning,
        DocumentaryStatus::Filming,
        DocumentaryStatus::Editing,
        DocumentaryStatus::Released,
        DocumentaryStatus::Abandoned,
    ];

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

    /// Every status the lifecycle lets a Documentary move to from `self`.
    pub fn moves(self) -> Vec<DocumentaryStatus> {
        DocumentaryStatus::ALL.into_iter().filter(|to| self.move_allowed(*to)).collect()
    }

    /// The founder-facing name, for example "Filming".
    pub fn label(self) -> &'static str {
        match self {
            DocumentaryStatus::Planning => "Planning",
            DocumentaryStatus::Filming => "Filming",
            DocumentaryStatus::Editing => "Editing",
            DocumentaryStatus::Released => "Released",
            DocumentaryStatus::Abandoned => "Abandoned",
        }
    }

    pub(crate) fn as_db(self) -> &'static str {
        match self {
            DocumentaryStatus::Planning => "planning",
            DocumentaryStatus::Filming => "filming",
            DocumentaryStatus::Editing => "editing",
            DocumentaryStatus::Released => "released",
            DocumentaryStatus::Abandoned => "abandoned",
        }
    }

    pub(crate) fn from_db(s: &str) -> Option<DocumentaryStatus> {
        DocumentaryStatus::ALL.into_iter().find(|st| st.as_db() == s)
    }
}

impl fmt::Display for DocumentaryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}
