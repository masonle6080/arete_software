//! Links of an item: what a Thesis, an Essay, or a Style is linked to, as its
//! screen header shows them in chips (spec #34). Each function reads one
//! item's links and counts in one call, so the UI never loads full lists to
//! count them.

use crate::blank::{self, Blank};
use crate::document;
use crate::documentary::{self, Documentary};
use crate::drop::{self, Drop, DropStage};
use crate::error::CoreResult;
use crate::library::Library;
use crate::style::{self, StyleBase, StyleId};
use crate::thesis::{self, Thesis, ThesisId};

/// The links of a Thesis.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThesisLinks {
    /// The one Drop of the Thesis that is not Cancelled, if any.
    pub drop: Option<Drop>,
    /// The Essays that support the Thesis.
    pub essay_count: i64,
    pub documentary: Option<Documentary>,
}

/// The links of a Thesis.
pub fn thesis_links(lib: &Library, thesis_id: ThesisId) -> CoreResult<ThesisLinks> {
    let drop = drop::thesis_drops(lib, thesis_id)?.drops.into_iter().find(|d| d.stage != DropStage::Cancelled);
    let essay_count =
        lib.db().query_row("SELECT COUNT(*) FROM essay_thesis WHERE thesis_id = ?1", [thesis_id], |r| r.get(0))?;
    let documentary = documentary::thesis_documentary(lib, thesis_id)?;
    Ok(ThesisLinks { drop, essay_count, documentary })
}

/// The links of an Essay.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EssayLinks {
    /// The Theses the Essay supports, oldest first.
    pub theses: Vec<Thesis>,
    /// The Passages in the Essay's text. Orphaned Passages are not counted.
    pub passage_count: i64,
    /// The Design Decisions that those Passages motivate, each counted once.
    pub design_decision_count: i64,
}

/// The links of an Essay. Refuses a document that is not an Essay.
pub fn essay_links(lib: &Library, essay_id: i64) -> CoreResult<EssayLinks> {
    let theses = document::theses_of_essay(lib, essay_id)?;
    let db = lib.db();
    let passage_count = db.query_row(
        "SELECT COUNT(*) FROM passage WHERE document_id = ?1 AND orphaned = 0",
        [essay_id],
        |r| r.get(0),
    )?;
    let design_decision_count = db.query_row(
        "SELECT COUNT(DISTINCT l.design_decision_id)
         FROM design_decision_passage l JOIN passage p ON p.id = l.passage_id
         WHERE p.document_id = ?1 AND p.orphaned = 0",
        [essay_id],
        |r| r.get(0),
    )?;
    Ok(EssayLinks { theses, passage_count, design_decision_count })
}

/// The links of a Style.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleLinks {
    pub drop: Drop,
    /// The Thesis of the Style's Drop.
    pub thesis: Thesis,
    /// What the Style is built on, or `None` when the founder has not set it.
    pub base: Option<BaseLink>,
    /// The Passages that motivate the Style's Design Decisions, each counted
    /// once. An orphaned Passage still counts: its links stay (ADR-0001).
    pub passage_count: i64,
}

/// A Style's base, with the Blank record so the chip can name it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BaseLink {
    Blank { blank: Blank },
    TechPack,
}

/// The links of a Style.
pub fn style_links(lib: &Library, style_id: StyleId) -> CoreResult<StyleLinks> {
    let found = style::get_style(lib, style_id)?;
    let drop = drop::get_drop(lib, found.drop_id)?;
    let thesis = thesis::get_thesis(lib, drop.thesis_id)?;
    let base = match style::style_base(lib, style_id)? {
        None => None,
        Some(StyleBase::TechPack) => Some(BaseLink::TechPack),
        Some(StyleBase::Blank { blank_id }) => Some(BaseLink::Blank { blank: blank::get_blank(lib, blank_id)? }),
    };
    let passage_count = lib.db().query_row(
        "SELECT COUNT(DISTINCT l.passage_id)
         FROM design_decision_passage l JOIN design_decision d ON d.id = l.design_decision_id
         WHERE d.style_id = ?1",
        [style_id],
        |r| r.get(0),
    )?;
    Ok(StyleLinks { drop, thesis, base, passage_count })
}
