//! Moodboard items linked to Passages.
//!
//! The scene JSON owns the link: the canvas sets `customData.passageId` on an
//! element to link it and removes it to unlink. On every scene save,
//! [`reindex`] copies the scene's links into the `moodboard_item_passage`
//! table, so the core can answer "which Moodboard items link to Passage X"
//! ([`moodboard_items_of_passage`]) without reading every scene. One source
//! of truth (the scene) means a deleted canvas item cannot leave a stale link.
//!
//! An item links to at most one Passage. A link to a Passage the library does
//! not have (its document was deleted) is left out of the table, and the
//! save still succeeds, so the canvas never becomes unsaveable.

use rusqlite::{params, Connection, Row};
use serde_json::Value;

use super::{from_row as moodboard_from_row, Moodboard, MoodboardId};
use crate::document;
use crate::error::CoreResult;
use crate::library::Library;

/// A canvas item linked to a Passage, with enough to show it in a list.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedMoodboardItem {
    pub moodboard: Moodboard,
    /// The Excalidraw element ID.
    pub element_id: String,
    /// The Excalidraw element type: `image`, `text`, `rectangle`, ...
    pub element_type: String,
    /// The text of a text element.
    pub text: Option<String>,
    /// The `MediaRef` of an image element.
    pub media: Option<String>,
    /// The fill color, when the element has one (a swatch).
    pub color: Option<String>,
}

/// The links in one scene: (element ID, Passage ID, type, text, media, color).
fn links_in(scene: &Value) -> Vec<(String, String, String, Option<String>, Option<String>, Option<String>)> {
    let str_of = |el: &Value, key: &str| el.get(key).and_then(Value::as_str).map(str::to_owned);
    let Some(elements) = scene.get("elements").and_then(Value::as_array) else {
        return Vec::new();
    };
    elements
        .iter()
        .filter(|el| el.get("isDeleted").and_then(Value::as_bool) != Some(true))
        .filter_map(|el| {
            let passage = el.pointer("/customData/passageId").and_then(Value::as_str)?;
            let id = str_of(el, "id")?;
            let color = str_of(el, "backgroundColor").filter(|c| c != "transparent");
            Some((
                id,
                passage.to_owned(),
                str_of(el, "type").unwrap_or_default(),
                str_of(el, "text"),
                str_of(el, "fileId"),
                color,
            ))
        })
        .collect()
}

/// Replaces the stored links of Moodboard `id` with the links in `scene`.
/// Call it inside the transaction that saves the scene.
pub(super) fn reindex(db: &Connection, id: MoodboardId, scene: &Value) -> CoreResult<()> {
    db.execute("DELETE FROM moodboard_item_passage WHERE moodboard_id = ?1", [id])?;
    let mut insert = db.prepare(
        "INSERT OR REPLACE INTO moodboard_item_passage
             (moodboard_id, element_id, passage_id, element_type, text, media, color)
         SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7 WHERE EXISTS (SELECT 1 FROM passage WHERE id = ?3)",
    )?;
    for (element, passage, kind, text, media, color) in links_in(scene) {
        insert.execute(params![id, element, passage, kind, text, media, color])?;
    }
    Ok(())
}

fn item(row: &Row) -> rusqlite::Result<LinkedMoodboardItem> {
    Ok(LinkedMoodboardItem {
        moodboard: moodboard_from_row(row)?,
        element_id: row.get(4)?,
        element_type: row.get(5)?,
        text: row.get(6)?,
        media: row.get(7)?,
        color: row.get(8)?,
    })
}

/// The canvas items linked to Passage `passage_id`, orphaned or not, by
/// Moodboard and then by element. Fails with `PassageNotFound` when the
/// Passage does not exist.
pub fn moodboard_items_of_passage(lib: &Library, passage_id: &str) -> CoreResult<Vec<LinkedMoodboardItem>> {
    document::get_passage(lib, passage_id)?;
    let mut stmt = lib.db().prepare(
        "SELECT m.id, m.owner_kind, m.owner_id, m.title,
                l.element_id, l.element_type, l.text, l.media, l.color
         FROM moodboard_item_passage l JOIN moodboard m ON m.id = l.moodboard_id
         WHERE l.passage_id = ?1
         ORDER BY m.id, l.rowid",
    )?;
    let items = stmt.query_map([passage_id], item)?.collect::<Result<_, _>>()?;
    Ok(items)
}
