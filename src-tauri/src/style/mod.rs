//! Style: a single garment design in a single color, on one Drop.
//!
//! Domain command functions for this area: [`create_style`], [`move_style`],
//! [`get_style`], [`drop_styles`], [`attach_style_files`], [`style_files`].
//! The status rules live on [`StyleStatus`]. The Drop's Production guard
//! reads Style statuses (see `guard_move` in `drop/mod.rs`).
//!
//! Later areas hang off a Style (Design Decisions, base, Decorations,
//! Moodboard, Samples, Cost Sheet). Give each its own module and table with a
//! `style_id`, not new columns here.

mod status;

use std::path::{Path, PathBuf};

use rusqlite::{params, OptionalExtension, Row};

use crate::drop::{self, DropId};
use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::media::{self, MediaRef};

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
}

const COLUMNS: &str = "id, drop_id, name, color, status";

fn from_row(row: &Row) -> rusqlite::Result<Style> {
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
    })
}

/// Adds a Style in one color to a Drop. A new Style starts as Draft. The
/// name and the color must not be blank.
pub fn create_style(lib: &Library, drop_id: DropId, name: &str, color: &str) -> CoreResult<Style> {
    let (name, color) = (name.trim(), color.trim());
    if name.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Style needs a name".into() });
    }
    if color.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Style needs a color".into() });
    }
    drop::get_drop(lib, drop_id)?;
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
    let style = get_style(lib, id)?;
    if !style.status.move_allowed(to) {
        return Err(CoreError::StatusMoveRefused { what: "Style", from: style.status.label(), to: to.label() });
    }
    lib.writer()?.execute("UPDATE style SET status = ?1 WHERE id = ?2", params![to.as_db(), id])?;
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
    Ok(rows
        .into_iter()
        .map(|(file_id, stored, file_name)| {
            let media = MediaRef::from_stored(stored);
            let path = media::media_path(lib, &media);
            StyleFile { id: file_id, media, file_name, path }
        })
        .collect())
}

fn display_name(source: &Path) -> String {
    source.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| source.display().to_string())
}

/// Every Style of a Drop, Cut ones included, oldest first.
pub fn drop_styles(lib: &Library, drop_id: DropId) -> CoreResult<Vec<Style>> {
    drop::get_drop(lib, drop_id)?;
    let mut stmt = lib.db().prepare(&format!("SELECT {COLUMNS} FROM style WHERE drop_id = ?1 ORDER BY id"))?;
    let styles = stmt.query_map([drop_id], from_row)?.collect::<Result<_, _>>()?;
    Ok(styles)
}

/// Reads one Style.
pub fn get_style(lib: &Library, id: StyleId) -> CoreResult<Style> {
    lib.db()
        .query_row(&format!("SELECT {COLUMNS} FROM style WHERE id = ?1"), [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { record: "Style", id })
}
