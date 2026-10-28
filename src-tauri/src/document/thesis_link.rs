//! Essay <-> Thesis links. The link is many-to-many: an Essay can support any
//! number of Theses, including none (GLOSSARY.md).

use rusqlite::params;

use super::{find, record, DocumentKind, DocumentSummary, COLUMNS};
use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::thesis::{self, Thesis, ThesisId};

/// Finds `id` and refuses it unless it is an Essay.
fn find_essay(lib: &Library, id: i64) -> CoreResult<()> {
    let rec = find(lib, id)?;
    if rec.summary.kind != DocumentKind::Essay {
        return Err(CoreError::InvalidInput { reason: "only an Essay can be linked to a Thesis".into() });
    }
    Ok(())
}

/// Links an Essay to a Thesis. Linking an Essay that is already linked to
/// that Thesis changes nothing.
pub fn link_essay_to_thesis(lib: &Library, essay_id: i64, thesis_id: ThesisId) -> CoreResult<()> {
    let db = lib.writer()?;
    find_essay(lib, essay_id)?;
    thesis::get_thesis(lib, thesis_id)?;
    db.execute(
        "INSERT OR IGNORE INTO essay_thesis (document_id, thesis_id) VALUES (?1, ?2)",
        params![essay_id, thesis_id],
    )?;
    Ok(())
}

/// Removes the link between an Essay and a Thesis. The Essay, the Thesis,
/// and the Essay's other links stay. Unlinking a pair that is not linked
/// changes nothing.
pub fn unlink_essay_from_thesis(lib: &Library, essay_id: i64, thesis_id: ThesisId) -> CoreResult<()> {
    let db = lib.writer()?;
    find_essay(lib, essay_id)?;
    thesis::get_thesis(lib, thesis_id)?;
    db.execute(
        "DELETE FROM essay_thesis WHERE document_id = ?1 AND thesis_id = ?2",
        params![essay_id, thesis_id],
    )?;
    Ok(())
}

/// The Theses an Essay supports, oldest Thesis first.
pub fn theses_of_essay(lib: &Library, essay_id: i64) -> CoreResult<Vec<Thesis>> {
    find_essay(lib, essay_id)?;
    let mut stmt = lib
        .db()
        .prepare("SELECT thesis_id FROM essay_thesis WHERE document_id = ?1 ORDER BY thesis_id")?;
    let ids: Vec<ThesisId> = stmt.query_map([essay_id], |r| r.get(0))?.collect::<Result<_, _>>()?;
    ids.into_iter().map(|id| thesis::get_thesis(lib, id)).collect()
}

/// The Essays linked to a Thesis, most recently edited first.
pub fn essays_of_thesis(lib: &Library, thesis_id: ThesisId) -> CoreResult<Vec<DocumentSummary>> {
    thesis::get_thesis(lib, thesis_id)?;
    let cols = COLUMNS.split(", ").map(|c| format!("d.{c}")).collect::<Vec<_>>().join(", ");
    let mut stmt = lib.db().prepare(&format!(
        "SELECT {cols} FROM document d JOIN essay_thesis l ON l.document_id = d.id
         WHERE l.thesis_id = ?1 ORDER BY d.edit_seq DESC"
    ))?;
    let rows = stmt.query_map([thesis_id], record)?;
    Ok(rows.map(|r| r.map(|rec| rec.summary)).collect::<Result<_, _>>()?)
}
