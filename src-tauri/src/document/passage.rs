//! Passages and save reports (ADR-0001).
//!
//! The text owns Passage boundaries: the Markdown holds a pair of markers per
//! Passage. On every save, the document module (`src/document/`) reports the
//! Passages present and their current text. The core keeps, for each Passage,
//! its document, its last known text, and whether it is orphaned:
//!
//! - A reported Passage gets its text as last known text. A new ID makes a new
//!   Passage. An orphaned Passage that is reported again is restored.
//! - A Passage of the document that the report leaves out is orphaned. It is
//!   never deleted, so it keeps its last known text and its links.
//!
//! Passages live in documents of any kind (Essay or Script).

use std::collections::HashSet;

use rusqlite::{params, Connection, OptionalExtension, Row};

use super::{find, save_document, DocumentEdit, DocumentKind, DocumentSummary};
use crate::error::{CoreError, CoreResult};
use crate::library::Library;

/// One Passage in a save report: its ID and its current text.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassageText {
    pub id: String,
    pub text: String,
}

/// A Passage's record.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Passage {
    pub id: String,
    pub document_id: i64,
    pub document_kind: DocumentKind,
    pub document_title: String,
    /// The text at the last save that had this Passage.
    pub last_known_text: String,
    /// True when the last save did not have this Passage (its text was fully
    /// deleted, or its mark removed).
    pub orphaned: bool,
    pub created_at: String,
    pub updated_at: String,
}

const SELECT: &str = "SELECT p.id, p.document_id, d.kind, d.title, p.last_known_text, p.orphaned,
                             p.created_at, p.updated_at
                      FROM passage p JOIN document d ON d.id = p.document_id";

fn passage(row: &Row) -> rusqlite::Result<Passage> {
    let kind: String = row.get(2)?;
    Ok(Passage {
        id: row.get(0)?,
        document_id: row.get(1)?,
        document_kind: DocumentKind::parse(&kind)?,
        document_title: row.get(3)?,
        last_known_text: row.get(4)?,
        orphaned: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

/// The characters of a Passage ID. Mirrors `PASSAGE_ID` in
/// `src/document/passage.ts`; the Markdown markers depend on it.
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn invalid(reason: impl Into<String>) -> CoreError {
    CoreError::InvalidInput { reason: reason.into() }
}

/// Refuses a report that the core cannot apply to `document_id`.
fn check_report(db: &Connection, document_id: i64, report: &[PassageText]) -> CoreResult<()> {
    let mut seen = HashSet::new();
    for p in report {
        if !valid_id(&p.id) {
            return Err(invalid(format!("{:?} is not a valid Passage ID", p.id)));
        }
        if !seen.insert(p.id.as_str()) {
            return Err(invalid(format!("Passage {} is in the save report twice", p.id)));
        }
        let owner: Option<i64> = db
            .query_row("SELECT document_id FROM passage WHERE id = ?1", [&p.id], |r| r.get(0))
            .optional()?;
        if owner.is_some_and(|owner| owner != document_id) {
            return Err(invalid(format!("Passage {} belongs to another document", p.id)));
        }
    }
    Ok(())
}

/// Applies a checked report to the `passage` table.
fn apply_report(db: &Connection, document_id: i64, report: &[PassageText]) -> CoreResult<()> {
    for p in report {
        db.execute(
            "INSERT INTO passage (id, document_id, last_known_text) VALUES (?1, ?2, ?3)
             ON CONFLICT (id) DO UPDATE
             SET last_known_text = excluded.last_known_text,
                 orphaned = 0,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE last_known_text IS NOT excluded.last_known_text OR orphaned = 1",
            params![p.id, document_id, p.text],
        )?;
    }
    let present: HashSet<&str> = report.iter().map(|p| p.id.as_str()).collect();
    let mut stmt = db.prepare("SELECT id FROM passage WHERE document_id = ?1 AND orphaned = 0")?;
    let live: Vec<String> = stmt.query_map([document_id], |r| r.get(0))?.collect::<Result<_, _>>()?;
    for id in live.iter().filter(|id| !present.contains(id.as_str())) {
        db.execute(
            "UPDATE passage SET orphaned = 1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?1",
            [id],
        )?;
    }
    Ok(())
}

/// Saves a document (as [`save_document`] does) together with its save
/// report: `passages` is every Passage present in `edit.markdown` with its
/// current text. Refuses the whole save, before anything is written, when a
/// Passage ID is malformed, repeated, or belongs to another document.
pub fn save_document_with_passages(
    lib: &Library,
    id: i64,
    edit: &DocumentEdit,
    passages: &[PassageText],
) -> CoreResult<DocumentSummary> {
    let db = lib.writer()?;
    find(lib, id)?;
    check_report(db, id, passages)?;
    let tx = db.unchecked_transaction()?;
    let summary = save_document(lib, id, edit)?;
    apply_report(&tx, id, passages)?;
    tx.commit()?;
    Ok(summary)
}

/// One Passage, orphaned or not.
pub fn get_passage(lib: &Library, id: &str) -> CoreResult<Passage> {
    lib.db()
        .query_row(&format!("{SELECT} WHERE p.id = ?1"), [id], passage)
        .optional()?
        .ok_or_else(|| CoreError::PassageNotFound { id: id.into() })
}

/// Every Passage of a document, orphaned ones included, oldest first.
pub fn passages_of_document(lib: &Library, document_id: i64) -> CoreResult<Vec<Passage>> {
    find(lib, document_id)?;
    let mut stmt = lib.db().prepare(&format!(
        "{SELECT} WHERE p.document_id = ?1 ORDER BY p.created_at, p.rowid"
    ))?;
    let rows = stmt.query_map([document_id], passage)?;
    Ok(rows.collect::<Result<_, _>>()?)
}
