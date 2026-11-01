//! Style: a single garment design in a single color, on one Drop.
//!
//! Domain command functions for this area: [`create_style`], [`move_style`],
//! [`get_style`], [`drop_styles`], [`attach_style_files`], [`style_files`],
//! [`remove_style_file`], the Cover ([`set_style_cover`],
//! [`clear_style_cover`]), and, for the base and sizes (`base.rs`),
//! [`style_base`], [`set_style_base`], [`style_sizes`], [`set_style_sizes`],
//! and [`duplicate_style`] (`duplicate.rs`).
//! The status rules live on [`StyleStatus`]. The Drop's Production guard
//! reads Style statuses (see `guard_move` in `drop/mod.rs`).
//!
//! Later areas hang off a Style (Design Decisions, base, Decorations,
//! Moodboard, Samples, Cost Sheet). Give each its own module and table with a
//! `style_id`, not new columns here. The Cover (`style.cover_file_id`) is
//! the one exception: it points at one of the Style's own design files.

mod base;
mod duplicate;
mod status;

use std::path::{Path, PathBuf};

use rusqlite::{params, OptionalExtension, Row};

use crate::drop::{self, DropId};
use crate::error::{CoreError, CoreResult};
use crate::library::{require_row, Library};
use crate::lifecycle;
use crate::media::{self, MediaRef};

pub use base::{set_style_base, set_style_sizes, style_base, style_sizes, StyleBase};
pub(crate) use base::{blank_sizes_in_use, read_base};
pub use duplicate::duplicate_style;
pub use status::StyleStatus;

pub type StyleId = i64;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Style {
    pub id: StyleId,
    pub drop_id: DropId,
    pub name: String,
    /// The one color of this Style. The same design in another color is a
    /// separate Style.
    pub color: String,
    pub status: StyleStatus,
    /// The statuses the lifecycle lets this Style move to now, so the UI
    /// never repeats the rules.
    pub moves: Vec<StyleStatus>,
    /// The one image design file picked to represent this Style, if any.
    pub cover: Option<StyleFile>,
}

/// The Style columns, then its Cover file's columns (NULL with no Cover).
/// Select them `FROM style s LEFT JOIN style_file f ON f.id = s.cover_file_id`.
const COLUMNS: &str = "s.id, s.drop_id, s.name, s.color, s.status, f.id, f.media, f.file_name";
const FROM: &str = "style s LEFT JOIN style_file f ON f.id = s.cover_file_id";

fn from_row(lib: &Library, row: &Row) -> rusqlite::Result<Style> {
    let raw: String = row.get(4)?;
    let status = StyleStatus::from_db(&raw).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, format!("unknown status {raw}").into())
    })?;
    Ok(Style {
        id: row.get(0)?,
        drop_id: row.get(1)?,
        name: row.get(2)?,
        color: row.get(3)?,
        status,
        moves: status.moves(),
        cover: match row.get::<_, Option<i64>>(5)? {
            Some(file_id) => Some(style_file(lib, file_id, row.get(6)?, row.get(7)?)),
            None => None,
        },
    })
}

/// Adds a Style in one color to a Drop. A new Style starts as Draft. The
/// name and the color must not be blank. The Drop must be in Design or
/// Sampling ([`drop::DropStage::takes_new_styles`]).
pub fn create_style(lib: &Library, drop_id: DropId, name: &str, color: &str) -> CoreResult<Style> {
    let (name, color) = (name.trim(), color.trim());
    if name.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Style needs a name".into() });
    }
    if color.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Style needs a color".into() });
    }
    drop::check_takes_new_styles(&drop::get_drop(lib, drop_id)?)?;
    let conn = lib.writer()?;
    conn.execute(
        "INSERT INTO style (drop_id, name, color, status) VALUES (?1, ?2, ?3, ?4)",
        params![drop_id, name, color, StyleStatus::Draft.as_db()],
    )?;
    get_style(lib, conn.last_insert_rowid())
}

/// Moves a Style to `to`. The lifecycle ([`StyleStatus::move_allowed`])
/// refuses skipped, reversed, and post-Cut moves.
pub fn move_style(lib: &Library, id: StyleId, to: StyleStatus) -> CoreResult<Style> {
    let from = get_style(lib, id)?.status;
    lifecycle::move_status(lib, "style", id, from, to)?;
    get_style(lib, id)
}

/// One design file attached to a Style.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleFile {
    pub id: i64,
    pub media: MediaRef,
    /// The name the file had when the founder attached it.
    pub file_name: String,
    /// Where the stored copy is now, inside the library's media folder.
    pub path: PathBuf,
}

/// Attaches design files (flat sketches, Illustrator or Procreate work) to a
/// Style through the media import path, and returns all its files. `sources`
/// are files outside the library; the core stores a copy of each.
pub fn attach_style_files(lib: &Library, id: StyleId, sources: &[PathBuf]) -> CoreResult<Vec<StyleFile>> {
    get_style(lib, id)?;
    let conn = lib.writer()?;
    let imported: Vec<(MediaRef, String)> = sources
        .iter()
        .map(|source| Ok((media::import_media(lib, source)?, display_name(source))))
        .collect::<CoreResult<_>>()?;
    let tx = conn.unchecked_transaction()?;
    for (media, file_name) in &imported {
        tx.execute(
            "INSERT INTO style_file (style_id, media, file_name) VALUES (?1, ?2, ?3)",
            params![id, media.file_name(), file_name],
        )?;
    }
    tx.commit()?;
    style_files(lib, id)
}

/// The design files of a Style, in the order attached.
pub fn style_files(lib: &Library, id: StyleId) -> CoreResult<Vec<StyleFile>> {
    get_style(lib, id)?;
    let mut stmt = lib.db().prepare("SELECT id, media, file_name FROM style_file WHERE style_id = ?1 ORDER BY id")?;
    let rows: Vec<(i64, String, String)> =
        stmt.query_map([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?.collect::<Result<_, _>>()?;
    Ok(rows.into_iter().map(|(file_id, stored, file_name)| style_file(lib, file_id, stored, file_name)).collect())
}

/// Removes design file `file_id` from Style `id`, and returns the files
/// left. If the file was the Cover, the Style has no Cover after: the
/// schema's `ON DELETE SET NULL` on `style.cover_file_id` clears it. The
/// stored media stays: other records (a duplicated Style) can share it.
pub fn remove_style_file(lib: &Library, id: StyleId, file_id: i64) -> CoreResult<Vec<StyleFile>> {
    get_style(lib, id)?;
    if lib.writer()?.execute("DELETE FROM style_file WHERE id = ?1 AND style_id = ?2", params![file_id, id])? == 0 {
        return Err(CoreError::NotFound { record: "Style design file", id: file_id });
    }
    style_files(lib, id)
}

fn style_file(lib: &Library, id: i64, stored: String, file_name: String) -> StyleFile {
    let media = MediaRef::from_stored(stored);
    let path = media::media_path(lib, &media);
    StyleFile { id, media, file_name, path }
}

/// The file names a Cover can have: PNG, JPEG, WebP, and HEIC images.
const COVER_EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "webp", "heic"];

fn is_cover_image(file_name: &str) -> bool {
    Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| COVER_EXTENSIONS.iter().any(|ok| e.eq_ignore_ascii_case(ok)))
}

/// Makes design file `file_id` the Cover of Style `id`, in place of any
/// earlier Cover. The file must be a PNG, JPEG, WebP, or HEIC image
/// attached to this Style.
pub fn set_style_cover(lib: &Library, id: StyleId, file_id: i64) -> CoreResult<Style> {
    let conn = lib.writer()?;
    // Check and write in one transaction, so the file cannot go between them.
    let tx = conn.unchecked_transaction()?;
    require_row(&tx, "style", "Style", id)?;
    let file_name: String = tx
        .query_row("SELECT file_name FROM style_file WHERE id = ?1 AND style_id = ?2", params![file_id, id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| CoreError::InvalidInput { reason: "the Cover must be one of this Style's design files".into() })?;
    if !is_cover_image(&file_name) {
        return Err(CoreError::InvalidInput { reason: "the Cover must be a PNG, JPEG, WebP, or HEIC image".into() });
    }
    tx.execute("UPDATE style SET cover_file_id = ?2 WHERE id = ?1", params![id, file_id])?;
    tx.commit()?;
    get_style(lib, id)
}

/// Leaves Style `id` with no Cover. The design file stays.
pub fn clear_style_cover(lib: &Library, id: StyleId) -> CoreResult<Style> {
    get_style(lib, id)?;
    lib.writer()?.execute("UPDATE style SET cover_file_id = NULL WHERE id = ?1", [id])?;
    get_style(lib, id)
}

fn display_name(source: &Path) -> String {
    source.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| source.display().to_string())
}

/// Every Style of a Drop, Cut ones included, oldest first.
pub fn drop_styles(lib: &Library, drop_id: DropId) -> CoreResult<Vec<Style>> {
    drop::get_drop(lib, drop_id)?;
    let mut stmt = lib.db().prepare(&format!("SELECT {COLUMNS} FROM {FROM} WHERE s.drop_id = ?1 ORDER BY s.id"))?;
    let styles = stmt.query_map([drop_id], |row| from_row(lib, row))?.collect::<Result<_, _>>()?;
    Ok(styles)
}

/// Reads one Style.
pub fn get_style(lib: &Library, id: StyleId) -> CoreResult<Style> {
    lib.db()
        .query_row(&format!("SELECT {COLUMNS} FROM {FROM} WHERE s.id = ?1"), [id], |row| from_row(lib, row))
        .optional()?
        .ok_or(CoreError::NotFound { record: "Style", id })
}
