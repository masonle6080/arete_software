//! What the founder adds to a Clip to find and connect it later: tags,
//! notes, and links to Passages. Also the Clip pool filters.

use rusqlite::params;

use super::{get_clip, Clip};
use crate::document::{self, Passage};
use crate::error::{CoreError, CoreResult};
use crate::library::Library;

/// The tags of one Clip, alphabetical, case ignored.
pub(super) fn tags_of(lib: &Library, clip_id: i64) -> rusqlite::Result<Vec<String>> {
    let mut stmt = lib.db().prepare_cached("SELECT tag FROM clip_tag WHERE clip_id = ?1 ORDER BY tag")?;
    let tags = stmt.query_map([clip_id], |r| r.get(0))?.collect();
    tags
}

/// A tag as stored: trimmed and non-empty.
fn clean_tag(tag: &str) -> CoreResult<&str> {
    let tag = tag.trim();
    if tag.is_empty() {
        return Err(CoreError::InvalidInput { reason: "A tag cannot be empty".into() });
    }
    Ok(tag)
}

/// Adds a tag to a Clip. A tag the Clip has already (case ignored) is kept
/// as it is.
pub fn add_tag(lib: &Library, clip_id: i64, tag: &str) -> CoreResult<Clip> {
    let tag = clean_tag(tag)?;
    get_clip(lib, clip_id)?;
    lib.writer()?
        .execute("INSERT OR IGNORE INTO clip_tag (clip_id, tag) VALUES (?1, ?2)", params![clip_id, tag])?;
    get_clip(lib, clip_id)
}

/// Replaces a Clip's notes. An empty text clears them.
pub fn set_notes(lib: &Library, clip_id: i64, notes: &str) -> CoreResult<Clip> {
    let changed = lib.writer()?.execute("UPDATE clip SET notes = ?1 WHERE id = ?2", params![notes, clip_id])?;
    if changed == 0 {
        return Err(CoreError::NotFound { record: "Clip", id: clip_id });
    }
    get_clip(lib, clip_id)
}

/// Links a Clip to a Passage. Linking it again changes nothing.
pub fn link_passage(lib: &Library, clip_id: i64, passage_id: &str) -> CoreResult<()> {
    get_clip(lib, clip_id)?;
    document::get_passage(lib, passage_id)?;
    lib.writer()?.execute(
        "INSERT OR IGNORE INTO clip_passage (clip_id, passage_id) VALUES (?1, ?2)",
        params![clip_id, passage_id],
    )?;
    Ok(())
}

/// Removes the link between a Clip and a Passage, if there is one.
pub fn unlink_passage(lib: &Library, clip_id: i64, passage_id: &str) -> CoreResult<()> {
    lib.writer()?.execute(
        "DELETE FROM clip_passage WHERE clip_id = ?1 AND passage_id = ?2",
        params![clip_id, passage_id],
    )?;
    Ok(())
}

/// The Passages a Clip is linked to, orphaned ones included, in link order.
pub fn clip_passages(lib: &Library, clip_id: i64) -> CoreResult<Vec<Passage>> {
    get_clip(lib, clip_id)?;
    let mut stmt = lib.db().prepare("SELECT passage_id FROM clip_passage WHERE clip_id = ?1 ORDER BY rowid")?;
    let ids: Vec<String> = stmt.query_map([clip_id], |r| r.get(0))?.collect::<Result<_, _>>()?;
    ids.iter().map(|id| document::get_passage(lib, id)).collect()
}

/// The Clips linked to a Passage, in pool order (newest date shot first).
pub fn passage_clips(lib: &Library, passage_id: &str) -> CoreResult<Vec<Clip>> {
    document::get_passage(lib, passage_id)?;
    filter_clips(lib, &ClipFilter { passage_id: Some(passage_id.into()), ..Default::default() })
}

/// What the Clip pool is filtered by. Every set field must match.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipFilter {
    /// Clips with this tag (case ignored).
    pub tag: Option<String>,
    /// Clips linked to this Passage.
    pub passage_id: Option<String>,
}

/// The Clips that match `filter`, in pool order. An empty filter gives every
/// Clip, as [`list_clips`](super::list_clips) does.
pub fn filter_clips(lib: &Library, filter: &ClipFilter) -> CoreResult<Vec<Clip>> {
    let mut stmt = lib.db().prepare(
        "SELECT id FROM clip c
         WHERE (?1 IS NULL OR EXISTS (SELECT 1 FROM clip_tag t WHERE t.clip_id = c.id AND t.tag = trim(?1)))
           AND (?2 IS NULL OR EXISTS (SELECT 1 FROM clip_passage p WHERE p.clip_id = c.id AND p.passage_id = ?2))
         ORDER BY date_shot IS NULL, date_shot DESC, id DESC",
    )?;
    let ids: Vec<i64> = stmt
        .query_map(params![filter.tag, filter.passage_id], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    ids.into_iter().map(|id| get_clip(lib, id)).collect()
}

/// Every tag in use on any Clip, alphabetical, case ignored.
pub fn all_tags(lib: &Library) -> CoreResult<Vec<String>> {
    let mut stmt = lib.db().prepare("SELECT tag FROM clip_tag GROUP BY tag ORDER BY tag")?;
    let tags = stmt.query_map([], |r| r.get(0))?.collect::<Result<_, _>>()?;
    Ok(tags)
}

/// Removes a tag (case ignored) from a Clip. Removing a tag the Clip does
/// not have changes nothing.
pub fn remove_tag(lib: &Library, clip_id: i64, tag: &str) -> CoreResult<Clip> {
    get_clip(lib, clip_id)?;
    lib.writer()?
        .execute("DELETE FROM clip_tag WHERE clip_id = ?1 AND tag = ?2", params![clip_id, tag.trim()])?;
    get_clip(lib, clip_id)
}
