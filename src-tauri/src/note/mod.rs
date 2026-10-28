//! Notes: a shared, dated note-and-attachment record (spec #1, "Notes"). A
//! note belongs to one owner: a Vendor, a Sample, or a Production Order. It
//! records when it was created; that timestamp is record metadata, not a
//! planning date.
//!
//! To use Notes on a new owner kind: the [`NoteOwner`] variant and the
//! `owner_kind` value already exist. Make [`owner_exists`] check the owner's
//! table, and add a delete trigger for that table (see `0004_note.sql`).
//!
//! Domain command functions for this area: [`add_note`], [`list_notes`].

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension};

use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::media::{self, MediaRef};

pub type NoteId = i64;

/// The record a note belongs to. Crosses IPC as `{ "kind": "vendor", "id": 3 }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum NoteOwner {
    Vendor(i64),
    Sample(i64),
    ProductionOrder(i64),
}

impl NoteOwner {
    fn kind_db_name(self) -> &'static str {
        match self {
            NoteOwner::Vendor(_) => "vendor",
            NoteOwner::Sample(_) => "sample",
            NoteOwner::ProductionOrder(_) => "production_order",
        }
    }

    fn id(self) -> i64 {
        match self {
            NoteOwner::Vendor(id) | NoteOwner::Sample(id) | NoteOwner::ProductionOrder(id) => id,
        }
    }

    fn not_found(self) -> CoreError {
        let record = match self {
            NoteOwner::Vendor(_) => "Vendor",
            NoteOwner::Sample(_) => "Sample",
            NoteOwner::ProductionOrder(_) => "Production Order",
        };
        CoreError::NotFound { record, id: self.id() }
    }
}

/// One file attached to a note.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub media: MediaRef,
    /// The name the file had when the founder attached it.
    pub file_name: String,
    /// Where the stored copy is now, inside the library's media folder.
    pub path: PathBuf,
}

/// A note as the founder sees it. Attachments are in the order added.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: NoteId,
    pub owner: NoteOwner,
    /// May be empty when the note has attachments.
    pub body: String,
    /// When the note was created, as `YYYY-MM-DDTHH:MM:SS.sssZ` (UTC).
    pub created_at: String,
    pub attachments: Vec<Attachment>,
}

/// What the founder writes. `attachments` are files outside the library; the
/// core imports a copy of each (see [`crate::media`]).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteInput {
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub attachments: Vec<PathBuf>,
}

/// Adds a note to `owner`. Refuses a note with no text and no attachment,
/// and an owner that does not exist.
pub fn add_note(lib: &Library, owner: NoteOwner, input: NoteInput) -> CoreResult<Note> {
    let body = input.body.trim().to_string();
    if body.is_empty() && input.attachments.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a note needs text or an attachment".into() });
    }
    let conn = lib.writer()?;
    if !owner_exists(conn, owner)? {
        return Err(owner.not_found());
    }
    let imported: Vec<(MediaRef, String)> = input
        .attachments
        .iter()
        .map(|source| Ok((media::import_media(lib, source)?, display_name(source))))
        .collect::<CoreResult<_>>()?;

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO note (owner_kind, owner_id, body) VALUES (?1, ?2, ?3)",
        (owner.kind_db_name(), owner.id(), &body),
    )?;
    let id = tx.last_insert_rowid();
    for (position, (media, file_name)) in imported.iter().enumerate() {
        tx.execute(
            "INSERT INTO note_attachment (note_id, position, media, file_name) VALUES (?1, ?2, ?3, ?4)",
            (id, position as i64, media.file_name(), file_name),
        )?;
    }
    tx.commit()?;
    get_note(lib, id)
}

/// The notes of `owner`, newest first.
pub fn list_notes(lib: &Library, owner: NoteOwner) -> CoreResult<Vec<Note>> {
    let conn = lib.db();
    let mut stmt = conn.prepare(
        "SELECT id, body, created_at FROM note
         WHERE owner_kind = ?1 AND owner_id = ?2
         ORDER BY created_at DESC, id DESC",
    )?;
    let rows: Vec<(NoteId, String, String)> = stmt
        .query_map((owner.kind_db_name(), owner.id()), |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<Result<_, _>>()?;
    rows.into_iter()
        .map(|(id, body, created_at)| {
            Ok(Note { id, owner, body, created_at, attachments: read_attachments(lib, id)? })
        })
        .collect()
}

fn get_note(lib: &Library, id: NoteId) -> CoreResult<Note> {
    let conn = lib.db();
    let row: Option<(String, i64, String, String)> = conn
        .query_row("SELECT owner_kind, owner_id, body, created_at FROM note WHERE id = ?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .optional()?;
    let (kind, owner_id, body, created_at) = row.ok_or(CoreError::NotFound { record: "Note", id })?;
    let owner = match kind.as_str() {
        "vendor" => NoteOwner::Vendor(owner_id),
        "sample" => NoteOwner::Sample(owner_id),
        _ => NoteOwner::ProductionOrder(owner_id),
    };
    Ok(Note { id, owner, body, created_at, attachments: read_attachments(lib, id)? })
}

fn read_attachments(lib: &Library, note_id: NoteId) -> CoreResult<Vec<Attachment>> {
    let mut stmt =
        lib.db().prepare("SELECT media, file_name FROM note_attachment WHERE note_id = ?1 ORDER BY position")?;
    let rows: Vec<(String, String)> =
        stmt.query_map([note_id], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
    Ok(rows
        .into_iter()
        .map(|(stored, file_name)| {
            let media = MediaRef::from_stored(stored);
            let path = media::media_path(lib, &media);
            Attachment { media, file_name, path }
        })
        .collect())
}

/// True when the owner record exists. Samples and Production Orders do not
/// exist until their areas are built, so for now no such owner exists.
fn owner_exists(conn: &Connection, owner: NoteOwner) -> CoreResult<bool> {
    match owner {
        NoteOwner::Vendor(id) => crate::vendor::vendor_exists(conn, id),
        NoteOwner::Sample(_) | NoteOwner::ProductionOrder(_) => Ok(false),
    }
}

fn display_name(source: &Path) -> String {
    source.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| source.display().to_string())
}
