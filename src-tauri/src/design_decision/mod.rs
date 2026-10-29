//! Design Decision: a choice made on a Style, classified by aspect, with a
//! written rationale and, optionally, the Passages that motivate it
//! (GLOSSARY: Design Decision).
//!
//! Domain command functions for this area: [`add_design_decision`],
//! [`get_design_decision`], [`style_design_decisions`], [`link_passage`],
//! [`unlink_passage`], [`set_decoration`], and, for the Passage side,
//! [`passage_design_decisions`].
//!
//! A Design Decision links to at most one Decoration, on the same Style. The
//! link is the `design_decision.decoration_id` column; removing the
//! Decoration clears it.

use rusqlite::{params, OptionalExtension, Row};

use crate::decoration::{self, DecorationId};
use crate::document::{self, Passage};
use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::style::{self, StyleId, StyleStatus};

pub type DesignDecisionId = i64;

/// What part of the garment a Design Decision is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DesignAspect {
    Decoration,
    Fabric,
    Color,
    Silhouette,
    Other,
}

impl DesignAspect {
    pub const ALL: [DesignAspect; 5] =
        [DesignAspect::Decoration, DesignAspect::Fabric, DesignAspect::Color, DesignAspect::Silhouette, DesignAspect::Other];

    fn as_db(self) -> &'static str {
        match self {
            DesignAspect::Decoration => "decoration",
            DesignAspect::Fabric => "fabric",
            DesignAspect::Color => "color",
            DesignAspect::Silhouette => "silhouette",
            DesignAspect::Other => "other",
        }
    }

    fn from_db(s: &str) -> Option<DesignAspect> {
        DesignAspect::ALL.into_iter().find(|a| a.as_db() == s)
    }
}

/// A Design Decision with its linked Decoration and Passages (in the order
/// linked).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignDecision {
    pub id: DesignDecisionId,
    pub style_id: StyleId,
    pub aspect: DesignAspect,
    pub rationale: String,
    /// The one Decoration, on the same Style, this choice is about.
    pub decoration_id: Option<DecorationId>,
    pub passages: Vec<Passage>,
}

const COLUMNS: &str = "id, style_id, aspect, rationale, decoration_id";

fn from_row(row: &Row) -> rusqlite::Result<DesignDecision> {
    let raw: String = row.get(2)?;
    let aspect = DesignAspect::from_db(&raw).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, format!("unknown aspect {raw}").into())
    })?;
    Ok(DesignDecision {
        id: row.get(0)?,
        style_id: row.get(1)?,
        aspect,
        rationale: row.get(3)?,
        decoration_id: row.get(4)?,
        passages: Vec::new(),
    })
}

/// Adds a Design Decision to a Style, with no Passages. The rationale must
/// not be blank.
pub fn add_design_decision(
    lib: &Library,
    style_id: StyleId,
    aspect: DesignAspect,
    rationale: &str,
) -> CoreResult<DesignDecision> {
    let rationale = rationale.trim();
    if rationale.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Design Decision needs a rationale".into() });
    }
    style::get_style(lib, style_id)?;
    let conn = lib.writer()?;
    conn.execute(
        "INSERT INTO design_decision (style_id, aspect, rationale) VALUES (?1, ?2, ?3)",
        params![style_id, aspect.as_db(), rationale],
    )?;
    get_design_decision(lib, conn.last_insert_rowid())
}

/// Reads one Design Decision.
pub fn get_design_decision(lib: &Library, id: DesignDecisionId) -> CoreResult<DesignDecision> {
    let decision = lib
        .db()
        .query_row(&format!("SELECT {COLUMNS} FROM design_decision WHERE id = ?1"), [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { record: "Design Decision", id })?;
    complete(lib, decision)
}

/// Every Design Decision of a Style, Cut or not, oldest first.
pub fn style_design_decisions(lib: &Library, style_id: StyleId) -> CoreResult<Vec<DesignDecision>> {
    style::get_style(lib, style_id)?;
    let mut stmt =
        lib.db().prepare(&format!("SELECT {COLUMNS} FROM design_decision WHERE style_id = ?1 ORDER BY id"))?;
    let rows: Vec<DesignDecision> = stmt.query_map([style_id], from_row)?.collect::<Result<_, _>>()?;
    rows.into_iter().map(|d| complete(lib, d)).collect()
}

/// Links a Passage, from any document, to a Design Decision. Linking a
/// Passage that is already linked changes nothing. An orphaned Passage can
/// be linked; it keeps its last known text.
pub fn link_passage(lib: &Library, id: DesignDecisionId, passage_id: &str) -> CoreResult<DesignDecision> {
    get_design_decision(lib, id)?;
    document::get_passage(lib, passage_id)?;
    lib.writer()?.execute(
        "INSERT OR IGNORE INTO design_decision_passage (design_decision_id, passage_id) VALUES (?1, ?2)",
        params![id, passage_id],
    )?;
    get_design_decision(lib, id)
}

/// Removes the link between a Passage and a Design Decision. A Design
/// Decision with no Passages is valid.
pub fn unlink_passage(lib: &Library, id: DesignDecisionId, passage_id: &str) -> CoreResult<DesignDecision> {
    get_design_decision(lib, id)?;
    lib.writer()?.execute(
        "DELETE FROM design_decision_passage WHERE design_decision_id = ?1 AND passage_id = ?2",
        params![id, passage_id],
    )?;
    get_design_decision(lib, id)
}

/// Links a Design Decision to one Decoration, in place of any earlier one,
/// or clears the link with `None`. Refuses a Decoration on another Style.
pub fn set_decoration(
    lib: &Library,
    id: DesignDecisionId,
    decoration_id: Option<DecorationId>,
) -> CoreResult<DesignDecision> {
    let decision = get_design_decision(lib, id)?;
    if let Some(decoration_id) = decoration_id {
        if decoration::get_decoration(lib, decoration_id)?.style_id != decision.style_id {
            return Err(CoreError::InvalidInput {
                reason: "a Design Decision can link only to a Decoration on its own Style".into(),
            });
        }
    }
    lib.writer()?
        .execute("UPDATE design_decision SET decoration_id = ?1 WHERE id = ?2", params![decoration_id, id])?;
    get_design_decision(lib, id)
}

/// A Design Decision as a Passage lists it: with the Style it belongs to.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassageDesignDecision {
    pub id: DesignDecisionId,
    pub style_id: StyleId,
    pub style_name: String,
    pub style_color: String,
    pub style_status: StyleStatus,
    pub aspect: DesignAspect,
    pub rationale: String,
}

/// Every Design Decision linked to a Passage, oldest first, on any Style
/// (Cut ones included).
pub fn passage_design_decisions(lib: &Library, passage_id: &str) -> CoreResult<Vec<PassageDesignDecision>> {
    document::get_passage(lib, passage_id)?;
    let mut stmt = lib.db().prepare(
        "SELECT d.id, d.style_id, d.aspect, d.rationale, d.decoration_id
         FROM design_decision_passage l JOIN design_decision d ON d.id = l.design_decision_id
         WHERE l.passage_id = ?1 ORDER BY d.id",
    )?;
    let rows: Vec<DesignDecision> = stmt.query_map([passage_id], from_row)?.collect::<Result<_, _>>()?;
    rows.into_iter()
        .map(|d| {
            let s = style::get_style(lib, d.style_id)?;
            Ok(PassageDesignDecision {
                id: d.id,
                style_id: s.id,
                style_name: s.name,
                style_color: s.color,
                style_status: s.status,
                aspect: d.aspect,
                rationale: d.rationale,
            })
        })
        .collect()
}

/// Fills in the linked Passages, in the order linked.
fn complete(lib: &Library, mut decision: DesignDecision) -> CoreResult<DesignDecision> {
    let mut stmt =
        lib.db().prepare("SELECT passage_id FROM design_decision_passage WHERE design_decision_id = ?1 ORDER BY id")?;
    let ids: Vec<String> = stmt.query_map([decision.id], |r| r.get(0))?.collect::<Result<_, _>>()?;
    decision.passages = ids.iter().map(|pid| document::get_passage(lib, pid)).collect::<CoreResult<_>>()?;
    Ok(decision)
}
