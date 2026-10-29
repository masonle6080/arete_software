//! Moodboard: a freeform canvas of visual references, text, and color
//! swatches, attached to a Thesis or a single Style.
//!
//! The owner is generic ([`MoodboardOwner`]), so other owner kinds reuse
//! this module unchanged except for one enum variant and its existence check.
//! The canvas scene is opaque JSON that the UI canvas (Excalidraw, ADR-0003)
//! writes; the core only checks it is JSON. Images in a scene reference media
//! by `MediaRef` (see `crate::media`), never inline bytes.
//!
//! Domain command functions for this area: [`create_moodboard`],
//! [`list_moodboards`], [`get_moodboard`], [`load_moodboard_scene`],
//! [`save_moodboard_scene`], and [`moodboard_items_of_passage`] (canvas items
//! linked to a Passage, see [`passage_link`]).

mod passage_link;

pub use passage_link::{moodboard_items_of_passage, LinkedMoodboardItem};

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{CoreError, CoreResult};
use crate::library::{self, Library};
use crate::style::StyleId;
use crate::thesis::ThesisId;

pub type MoodboardId = i64;

/// The record a Moodboard belongs to. Crosses IPC as
/// `{ "kind": "thesis", "id": 1 }`. To add an owner kind, add a variant, its
/// `kind` text in [`MoodboardOwner::parts`] and [`MoodboardOwner::from_parts`],
/// and its existence check in [`MoodboardOwner::ensure_exists`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum MoodboardOwner {
    Thesis(ThesisId),
    /// A Style has at most one Moodboard.
    Style(StyleId),
}

impl MoodboardOwner {
    fn parts(self) -> (&'static str, i64) {
        match self {
            MoodboardOwner::Thesis(id) => ("thesis", id),
            MoodboardOwner::Style(id) => ("style", id),
        }
    }

    fn from_parts(kind: &str, id: i64) -> Option<Self> {
        match kind {
            "thesis" => Some(MoodboardOwner::Thesis(id)),
            "style" => Some(MoodboardOwner::Style(id)),
            _ => None,
        }
    }

    /// Whether this owner kind has at most one Moodboard.
    fn single_board(self) -> bool {
        matches!(self, MoodboardOwner::Style(_))
    }

    fn ensure_exists(self, lib: &Library) -> CoreResult<()> {
        match self {
            MoodboardOwner::Thesis(id) => library::require_row(lib.db(), "thesis", "Thesis", id),
            MoodboardOwner::Style(id) => library::require_row(lib.db(), "style", "Style", id),
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
/// with `NotFound` when the owner does not exist, and with `InvalidInput`
/// when a Style already has its one Moodboard.
pub fn create_moodboard(lib: &Library, owner: MoodboardOwner, title: &str) -> CoreResult<Moodboard> {
    let title = title.trim();
    if title.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Moodboard needs a title".into() });
    }
    owner.ensure_exists(lib)?;
    if owner.single_board() && !list_moodboards(lib, owner)?.is_empty() {
        return Err(CoreError::InvalidInput { reason: "this Style already has a Moodboard".into() });
    }
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

/// Replaces the canvas scene and, with it, the scene's Passage links (an
/// element's `customData.passageId`). Refuses with `InvalidInput` text that
/// is not JSON, so a bad save cannot make the Moodboard unloadable.
pub fn save_moodboard_scene(lib: &Library, id: MoodboardId, scene: &str) -> CoreResult<()> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(scene) else {
        return Err(CoreError::InvalidInput { reason: "a Moodboard scene must be JSON".into() });
    };
    let db = lib.writer()?;
    let tx = db.unchecked_transaction()?;
    let changed = tx.execute(
        "UPDATE moodboard SET scene = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
        params![scene, id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound { record: "Moodboard", id });
    }
    passage_link::reindex(&tx, id, &value)?;
    tx.commit()?;
    Ok(())
}

/// Copies the Moodboard of Style `from`, if it has one, onto the new Style
/// `to`: its title and scene, and, through the save path's reindex, its
/// Passage links. Only `style::duplicate_style` calls this, inside its
/// transaction.
pub(crate) fn copy_style_moodboard(db: &Connection, from: StyleId, to: StyleId) -> CoreResult<()> {
    let (kind, from_id) = MoodboardOwner::Style(from).parts();
    let board: Option<(String, Option<String>)> = db
        .query_row(
            "SELECT title, scene FROM moodboard WHERE owner_kind = ?1 AND owner_id = ?2 ORDER BY id LIMIT 1",
            params![kind, from_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((title, scene)) = board else { return Ok(()) };
    db.execute(
        "INSERT INTO moodboard (owner_kind, owner_id, title, scene) VALUES (?1, ?2, ?3, ?4)",
        params![kind, to, title, scene],
    )?;
    let id = db.last_insert_rowid();
    if let Some(value) = scene.as_deref().and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok()) {
        passage_link::reindex(db, id, &value)?;
    }
    Ok(())
}
