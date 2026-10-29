//! Decoration: a treatment applied to a Style's base, such as a print,
//! embroidery, dye, or label (GLOSSARY: Decoration). It records technique,
//! placement, size, colors, an artwork file (imported through
//! [`crate::media`]), and the Vendor that applies it. A Design Decision can
//! link to one Decoration on the same Style (see [`crate::design_decision`]).
//!
//! Domain command functions for this area: [`add_decoration`],
//! [`update_decoration`], [`set_decoration_artwork`],
//! [`clear_decoration_artwork`], [`remove_decoration`], [`get_decoration`],
//! [`style_decorations`].
//!
//! Style duplication (ticket #22) copies each Decoration to a new ID. A
//! Design Decision stores its link as `design_decision.decoration_id`, so the
//! copy must map each old Decoration ID to its new one and set the copied
//! Design Decision's `decoration_id` from that map.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::media::{self, MediaRef};
use crate::style::{self, StyleId};
use crate::vendor::{self, VendorId};

pub type DecorationId = i64;

/// The artwork file of a Decoration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecorationArtwork {
    pub media: MediaRef,
    /// The name the file had when the founder imported it.
    pub file_name: String,
    /// Where the stored copy is now, inside the library's media folder.
    pub path: PathBuf,
}

/// A Decoration as the founder sees it. Colors are in the order listed.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Decoration {
    pub id: DecorationId,
    pub style_id: StyleId,
    pub technique: String,
    /// Free text such as "Center chest"; empty when unknown.
    pub placement: String,
    /// Free text such as "10 x 12 cm"; empty when unknown.
    pub size: String,
    pub colors: Vec<String>,
    pub artwork: Option<DecorationArtwork>,
    /// The Vendor that applies it; `None` when not chosen yet.
    pub vendor_id: Option<VendorId>,
    /// The Vendor's name, for display.
    pub vendor_name: Option<String>,
}

/// The fields the founder sets when adding or editing a Decoration. An edit
/// replaces every field. The artwork file is set on its own.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecorationInput {
    pub technique: String,
    #[serde(default)]
    pub placement: String,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub colors: Vec<String>,
    #[serde(default)]
    pub vendor_id: Option<VendorId>,
}

/// Adds a Decoration to a Style, with no artwork. The technique must not be
/// blank.
pub fn add_decoration(lib: &Library, style_id: StyleId, input: DecorationInput) -> CoreResult<Decoration> {
    style::get_style(lib, style_id)?;
    let input = validate(lib, input)?;
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO decoration (style_id, technique, placement, size, vendor_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![style_id, input.technique, input.placement, input.size, input.vendor_id],
    )?;
    let id = tx.last_insert_rowid();
    write_colors(&tx, id, &input.colors)?;
    tx.commit()?;
    get_decoration(lib, id)
}

/// Replaces every field of a Decoration except its artwork.
pub fn update_decoration(lib: &Library, id: DecorationId, input: DecorationInput) -> CoreResult<Decoration> {
    get_decoration(lib, id)?;
    let input = validate(lib, input)?;
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE decoration SET technique = ?1, placement = ?2, size = ?3, vendor_id = ?4 WHERE id = ?5",
        params![input.technique, input.placement, input.size, input.vendor_id, id],
    )?;
    tx.execute("DELETE FROM decoration_color WHERE decoration_id = ?1", [id])?;
    write_colors(&tx, id, &input.colors)?;
    tx.commit()?;
    get_decoration(lib, id)
}

/// Imports the file at `source` into the library and makes it the
/// Decoration's artwork, in place of any earlier one.
pub fn set_decoration_artwork(lib: &Library, id: DecorationId, source: &Path) -> CoreResult<Decoration> {
    get_decoration(lib, id)?;
    let conn = lib.writer()?;
    let media = media::import_media(lib, source)?;
    let file_name = source
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| source.display().to_string());
    conn.execute(
        "UPDATE decoration SET artwork_media = ?1, artwork_file_name = ?2 WHERE id = ?3",
        params![media.file_name(), file_name, id],
    )?;
    get_decoration(lib, id)
}

/// Takes the artwork off a Decoration. The stored media file stays.
pub fn clear_decoration_artwork(lib: &Library, id: DecorationId) -> CoreResult<Decoration> {
    get_decoration(lib, id)?;
    lib.writer()?
        .execute("UPDATE decoration SET artwork_media = NULL, artwork_file_name = NULL WHERE id = ?1", [id])?;
    get_decoration(lib, id)
}

/// Removes a Decoration from its Style. A Design Decision linked to it
/// stays, with no Decoration.
pub fn remove_decoration(lib: &Library, id: DecorationId) -> CoreResult<()> {
    get_decoration(lib, id)?;
    lib.writer()?.execute("DELETE FROM decoration WHERE id = ?1", [id])?;
    Ok(())
}

const SELECT: &str = "SELECT d.id, d.style_id, d.technique, d.placement, d.size, d.artwork_media,
            d.artwork_file_name, d.vendor_id, v.name
     FROM decoration d LEFT JOIN vendor v ON v.id = d.vendor_id";

/// Reads one Decoration.
pub fn get_decoration(lib: &Library, id: DecorationId) -> CoreResult<Decoration> {
    let decoration = lib
        .db()
        .query_row(&format!("{SELECT} WHERE d.id = ?1"), [id], |r| from_row(lib, r))
        .optional()?
        .ok_or(CoreError::NotFound { record: "Decoration", id })?;
    complete(lib.db(), decoration)
}

/// Every Decoration of a Style, oldest first.
pub fn style_decorations(lib: &Library, style_id: StyleId) -> CoreResult<Vec<Decoration>> {
    style::get_style(lib, style_id)?;
    let mut stmt = lib.db().prepare(&format!("{SELECT} WHERE d.style_id = ?1 ORDER BY d.id"))?;
    let rows: Vec<Decoration> = stmt.query_map([style_id], |r| from_row(lib, r))?.collect::<Result<_, _>>()?;
    rows.into_iter().map(|d| complete(lib.db(), d)).collect()
}

fn from_row(lib: &Library, r: &Row) -> rusqlite::Result<Decoration> {
    let artwork_media: Option<String> = r.get(5)?;
    let artwork_file_name: Option<String> = r.get(6)?;
    let artwork = artwork_media.zip(artwork_file_name).map(|(stored, file_name)| {
        let media = MediaRef::from_stored(stored);
        let path = media::media_path(lib, &media);
        DecorationArtwork { media, file_name, path }
    });
    Ok(Decoration {
        id: r.get(0)?,
        style_id: r.get(1)?,
        technique: r.get(2)?,
        placement: r.get(3)?,
        size: r.get(4)?,
        colors: Vec::new(),
        artwork,
        vendor_id: r.get(7)?,
        vendor_name: r.get(8)?,
    })
}

fn complete(conn: &Connection, mut decoration: Decoration) -> CoreResult<Decoration> {
    let mut stmt = conn.prepare("SELECT color FROM decoration_color WHERE decoration_id = ?1 ORDER BY position")?;
    decoration.colors = stmt.query_map([decoration.id], |r| r.get(0))?.collect::<Result<_, _>>()?;
    Ok(decoration)
}

/// Trims every field, drops blank colors, and checks the technique and the
/// Vendor.
fn validate(lib: &Library, input: DecorationInput) -> CoreResult<DecorationInput> {
    let technique = input.technique.trim().to_string();
    if technique.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Decoration needs a technique".into() });
    }
    if let Some(vendor_id) = input.vendor_id {
        vendor::get_vendor(lib, vendor_id)?;
    }
    Ok(DecorationInput {
        technique,
        placement: input.placement.trim().to_string(),
        size: input.size.trim().to_string(),
        colors: input.colors.iter().map(|c| c.trim()).filter(|c| !c.is_empty()).map(String::from).collect(),
        vendor_id: input.vendor_id,
    })
}

fn write_colors(conn: &Connection, id: DecorationId, colors: &[String]) -> CoreResult<()> {
    for (position, color) in colors.iter().enumerate() {
        conn.execute(
            "INSERT INTO decoration_color (decoration_id, position, color) VALUES (?1, ?2, ?3)",
            params![id, position as i64, color],
        )?;
    }
    Ok(())
}

/// Copies every Decoration of Style `from` onto the new Style `to`, with
/// its colors and artwork. Returns each old Decoration ID with its copy's
/// ID. Only `style::duplicate_style` calls this, inside its transaction.
pub(crate) fn copy_decorations(
    conn: &Connection,
    from: StyleId,
    to: StyleId,
) -> CoreResult<HashMap<DecorationId, DecorationId>> {
    let mut stmt = conn.prepare("SELECT id FROM decoration WHERE style_id = ?1 ORDER BY id")?;
    let old_ids: Vec<DecorationId> = stmt.query_map([from], |r| r.get(0))?.collect::<Result<_, _>>()?;
    let mut copies = HashMap::new();
    for old_id in old_ids {
        // The artwork media is shared: stored media never changes.
        conn.execute(
            "INSERT INTO decoration (style_id, technique, placement, size, artwork_media, artwork_file_name, vendor_id)
             SELECT ?1, technique, placement, size, artwork_media, artwork_file_name, vendor_id
             FROM decoration WHERE id = ?2",
            params![to, old_id],
        )?;
        let new_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO decoration_color (decoration_id, position, color)
             SELECT ?1, position, color FROM decoration_color WHERE decoration_id = ?2",
            params![new_id, old_id],
        )?;
        copies.insert(old_id, new_id);
    }
    Ok(copies)
}
