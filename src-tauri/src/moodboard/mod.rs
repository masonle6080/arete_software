//! Moodboard: a freeform canvas of visual references, text, and color
//! swatches, attached to a Thesis or (later) a single Style.
//!
//! The owner is generic ([`MoodboardOwner`]), so other owner kinds reuse
//! this module unchanged except for one enum variant and its existence check.
//! The canvas scene is opaque JSON that the UI canvas (Excalidraw, ADR-0003)
//! writes; the core only checks it is JSON. Images in a scene reference media
//! by `MediaRef` (see `crate::media`), never inline bytes.
//!
//! Domain command functions for this area: [`create_moodboard`],
//! [`list_moodboards`], [`get_moodboard`], [`load_moodboard_scene`],
//! [`save_moodboard_scene`].

use rusqlite::{params, OptionalExtension, Row};

use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::thesis::{self, ThesisId};

pub type MoodboardId = i64;

/// The record a Moodboard belongs to. Crosses IPC as
/// `{ "kind": "thesis", "id": 1 }`. To add an owner kind, add a variant, its
/// `kind` text in [`MoodboardOwner::parts`] and [`MoodboardOwner::from_parts`],
/// and its existence check in [`MoodboardOwner::ensure_exists`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum MoodboardOwner {
    Thesis(ThesisId),
}

impl MoodboardOwner {
    fn parts(self) -> (&'static str, i64) {
        match self {
            MoodboardOwner::Thesis(id) => ("thesis", id),
        }
    }

    fn from_parts(kind: &str, id: i64) -> Option<Self> {
        match kind {
            "thesis" => Some(MoodboardOwner::Thesis(id)),
            _ => None,
        }
    }

    fn ensure_exists(self, lib: &Library) -> CoreResult<()> {
        match self {
            MoodboardOwner::Thesis(id) => thesis::get_thesis(lib, id).map(drop),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Moodboard {
    pub id: MoodboardId,
    pub owner: MoodboardOwner,
    pub title: String,
}

const COLUMNS: &str = "id, owner_kind, owner_id, title";

fn from_row(row: &Row) -> rusqlite::Result<Moodboard> {
    let kind: String = row.get(1)?;
    let owner = MoodboardOwner::from_parts(&kind, row.get(2)?).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, format!("unknown owner kind {kind}").into())
    })?;
    Ok(Moodboard { id: row.get(0)?, owner, title: row.get(3)? })
}

/// Creates an empty Moodboard on `owner`. The title must not be blank. Fails
/// with `NotFound` when the owner does not exist.
pub fn create_moodboard(lib: &Library, owner: MoodboardOwner, title: &str) -> CoreResult<Moodboard> {
    let title = title.trim();
    if title.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Moodboard needs a title".into() });
    }
    owner.ensure_exists(lib)?;
    let (kind, owner_id) = owner.parts();
    let db = lib.writer()?;
    db.execute(
        "INSERT INTO moodboard (owner_kind, owner_id, title) VALUES (?1, ?2, ?3)",
        params![kind, owner_id, title],
    )?;
    get_moodboard(lib, db.last_insert_rowid())
}

/// The Moodboards of `owner`, oldest first.
pub fn list_moodboards(lib: &Library, owner: MoodboardOwner) -> CoreResult<Vec<Moodboard>> {
    let (kind, owner_id) = owner.parts();
    let mut stmt = lib
        .db()
        .prepare(&format!("SELECT {COLUMNS} FROM moodboard WHERE owner_kind = ?1 AND owner_id = ?2 ORDER BY id"))?;
    let boards = stmt.query_map(params![kind, owner_id], from_row)?.collect::<Result<_, _>>()?;
    Ok(boards)
}

/// Reads one Moodboard.
pub fn get_moodboard(lib: &Library, id: MoodboardId) -> CoreResult<Moodboard> {
    lib.db()
        .query_row(&format!("SELECT {COLUMNS} FROM moodboard WHERE id = ?1"), [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { record: "Moodboard", id })
}

/// The saved canvas scene JSON, or `None` for a Moodboard never saved.
pub fn load_moodboard_scene(lib: &Library, id: MoodboardId) -> CoreResult<Option<String>> {
    lib.db()
        .query_row("SELECT scene FROM moodboard WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .ok_or(CoreError::NotFound { record: "Moodboard", id })
}

/// Replaces the canvas scene. Refuses with `InvalidInput` text that is not
/// JSON, so a bad save cannot make the Moodboard unloadable.
pub fn save_moodboard_scene(lib: &Library, id: MoodboardId, scene: &str) -> CoreResult<()> {
    if serde_json::from_str::<serde_json::Value>(scene).is_err() {
        return Err(CoreError::InvalidInput { reason: "a Moodboard scene must be JSON".into() });
    }
    let changed = lib.writer()?.execute(
        "UPDATE moodboard SET scene = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
        params![scene, id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound { record: "Moodboard", id });
    }
    Ok(())
}
