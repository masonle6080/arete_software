//! Documentary: a film that develops one Thesis. A Thesis has at most one.
//!
//! Domain command functions for this area: [`create_documentary`],
//! [`move_documentary`], [`get_documentary`], [`thesis_documentary`],
//! [`list_documentaries`], and the Clip links [`add_clip`], [`remove_clip`],
//! [`documentary_clips`], [`clip_documentaries`], and the Script
//! [`documentary_script`]. The lifecycle rules live on
//! [`DocumentaryStatus`]. Releasing a Drop before its Thesis's Documentary
//! is Released warns (see `guard_move` in `drop/mod.rs`).

mod status;

use rusqlite::{params, OptionalExtension, Row};

use crate::clip::{self, Clip};
use crate::document::{self, Document, DocumentKind};
use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::lifecycle;
use crate::thesis::{self, ThesisId};

pub use status::DocumentaryStatus;

pub type DocumentaryId = i64;
type ClipId = i64;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Documentary {
    pub id: DocumentaryId,
    pub thesis_id: ThesisId,
    pub title: String,
    pub status: DocumentaryStatus,
    /// The statuses the lifecycle lets this Documentary move to now, so the
    /// UI never repeats the rules.
    pub moves: Vec<DocumentaryStatus>,
}

const COLUMNS: &str = "id, thesis_id, title, status";

fn from_row(row: &Row) -> rusqlite::Result<Documentary> {
    let raw: String = row.get(3)?;
    let status = DocumentaryStatus::from_db(&raw).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, format!("unknown status {raw}").into())
    })?;
    Ok(Documentary { id: row.get(0)?, thesis_id: row.get(1)?, title: row.get(2)?, status, moves: status.moves() })
}

/// Creates the Documentary of a Thesis. A new Documentary starts in
/// Planning. The title must not be blank. A Thesis that already has a
/// Documentary (in any status) cannot get another.
pub fn create_documentary(lib: &Library, thesis_id: ThesisId, title: &str) -> CoreResult<Documentary> {
    let title = title.trim();
    if title.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Documentary needs a title".into() });
    }
    thesis::get_thesis(lib, thesis_id)?;
    if thesis_documentary(lib, thesis_id)?.is_some() {
        return Err(CoreError::DocumentaryExists { thesis_id });
    }
    let db = lib.writer()?;
    db.execute(
        "INSERT INTO documentary (thesis_id, title, status) VALUES (?1, ?2, ?3)",
        params![thesis_id, title, DocumentaryStatus::Planning.as_db()],
    )?;
    get_documentary(lib, db.last_insert_rowid())
}

/// Moves a Documentary to `to`. Refuses moves that
/// [`DocumentaryStatus::move_allowed`] does not allow.
pub fn move_documentary(lib: &Library, id: DocumentaryId, to: DocumentaryStatus) -> CoreResult<Documentary> {
    let from = get_documentary(lib, id)?.status;
    lifecycle::move_status(lib, "documentary", id, from, to)?;
    get_documentary(lib, id)
}

/// Adds a Clip to a Documentary. A Clip can be in several Documentaries.
/// Adding a Clip that is already in the Documentary changes nothing.
pub fn add_clip(lib: &Library, id: DocumentaryId, clip_id: ClipId) -> CoreResult<()> {
    get_documentary(lib, id)?;
    clip::get_clip(lib, clip_id)?;
    lib.writer()?.execute(
        "INSERT OR IGNORE INTO documentary_clip (documentary_id, clip_id) VALUES (?1, ?2)",
        params![id, clip_id],
    )?;
    Ok(())
}

/// Takes a Clip out of a Documentary. The Clip stays in the pool.
pub fn remove_clip(lib: &Library, id: DocumentaryId, clip_id: ClipId) -> CoreResult<()> {
    lib.writer()?.execute(
        "DELETE FROM documentary_clip WHERE documentary_id = ?1 AND clip_id = ?2",
        params![id, clip_id],
    )?;
    Ok(())
}

/// The Clips of one Documentary: the Clip pool filtered by Documentary, in
/// the pool's order (newest date shot first; no date last).
pub fn documentary_clips(lib: &Library, id: DocumentaryId) -> CoreResult<Vec<Clip>> {
    get_documentary(lib, id)?;
    let mut stmt = lib.db().prepare(
        "SELECT c.id FROM clip c JOIN documentary_clip dc ON dc.clip_id = c.id
         WHERE dc.documentary_id = ?1
         ORDER BY c.date_shot IS NULL, c.date_shot DESC, c.id DESC",
    )?;
    let ids: Vec<ClipId> = stmt.query_map([id], |r| r.get(0))?.collect::<Result<_, _>>()?;
    ids.into_iter().map(|clip_id| clip::get_clip(lib, clip_id)).collect()
}

/// The Documentaries a Clip is in, oldest first.
pub fn clip_documentaries(lib: &Library, clip_id: ClipId) -> CoreResult<Vec<Documentary>> {
    let cols = COLUMNS.split(", ").map(|c| format!("d.{c}")).collect::<Vec<_>>().join(", ");
    let mut stmt = lib.db().prepare(&format!(
        "SELECT {cols} FROM documentary d JOIN documentary_clip dc ON dc.documentary_id = d.id
         WHERE dc.clip_id = ?1 ORDER BY d.id"
    ))?;
    let docs = stmt.query_map([clip_id], from_row)?.collect::<Result<_, _>>()?;
    Ok(docs)
}

/// Every Documentary, oldest first, so the Clip pool can offer them as filters.
pub fn list_documentaries(lib: &Library) -> CoreResult<Vec<Documentary>> {
    let mut stmt = lib.db().prepare(&format!("SELECT {COLUMNS} FROM documentary ORDER BY id"))?;
    let docs = stmt.query_map([], from_row)?.collect::<Result<_, _>>()?;
    Ok(docs)
}

/// The Documentary of a Thesis, if it has one.
pub fn thesis_documentary(lib: &Library, thesis_id: ThesisId) -> CoreResult<Option<Documentary>> {
    Ok(lib
        .db()
        .query_row(&format!("SELECT {COLUMNS} FROM documentary WHERE thesis_id = ?1"), [thesis_id], from_row)
        .optional()?)
}

/// Reads one Documentary.
pub fn get_documentary(lib: &Library, id: DocumentaryId) -> CoreResult<Documentary> {
    lib.db()
        .query_row(&format!("SELECT {COLUMNS} FROM documentary WHERE id = ?1"), [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { record: "Documentary", id })
}

/// The Script of a Documentary: a document of kind Script, written and saved
/// like an Essay (Markdown, Passages, save reports) through the document
/// module. Each Documentary has exactly one. The first call creates it, empty
/// and titled like the Documentary; later calls open the same one.
pub fn documentary_script(lib: &Library, id: DocumentaryId) -> CoreResult<Document> {
    let doc = get_documentary(lib, id)?;
    let script_id: Option<i64> =
        lib.db().query_row("SELECT script_id FROM documentary WHERE id = ?1", [id], |r| r.get(0))?;
    if let Some(script_id) = script_id {
        return document::open_document(lib, script_id);
    }
    let script = document::create_document(lib, DocumentKind::Script, &doc.title)?;
    lib.writer()?.execute("UPDATE documentary SET script_id = ?1 WHERE id = ?2", params![script.id, id])?;
    Ok(script)
}
