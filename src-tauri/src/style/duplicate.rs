//! Duplicating a Style: the same design in another color (spec #1, ticket #22).
//!
//! The copy is a new Draft Style on the same Drop, with the same name and
//! the founder's new color. It gets a copy of the base and sizes, the Tech
//! Pack (new Point of Measure and size IDs), the design files, the
//! Decorations, the Design Decisions with their Passage links, the
//! Moodboard, and the Cost Sheet. It does not get Samples: they record what
//! came back for the original color.
//!
//! Each area copies its own rows with a `copy_*` function in its module,
//! and every step runs on one transaction, so a failed step leaves no
//! partial copy. To copy another area, add its `copy_*` function and one
//! call in [`copy_areas`].

use rusqlite::{params, Connection};

use super::{base, get_style, Style, StyleId, StyleStatus};
use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::{cost_sheet, decoration, design_decision, moodboard, tech_pack};

/// Copies Style `id` into a new Draft Style in `color`, on the same Drop,
/// and returns the copy. The color must not be blank. Samples stay with the
/// original.
pub fn duplicate_style(lib: &Library, id: StyleId, color: &str) -> CoreResult<Style> {
    let color = color.trim();
    if color.is_empty() {
        return Err(CoreError::InvalidInput { reason: "the copy needs a color".into() });
    }
    let original = get_style(lib, id)?;
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO style (drop_id, name, color, status) VALUES (?1, ?2, ?3, ?4)",
        params![original.drop_id, original.name, color, StyleStatus::Draft.as_db()],
    )?;
    let copy = tx.last_insert_rowid();
    copy_areas(&tx, id, copy)?;
    tx.commit()?;
    get_style(lib, copy)
}

/// Copies every area of Style `from` onto Style `to`, in an order that
/// keeps references valid (the Tech Pack before the base that points to it,
/// Decorations before the Design Decisions that link to them).
fn copy_areas(conn: &Connection, from: StyleId, to: StyleId) -> CoreResult<()> {
    tech_pack::copy_tech_pack(conn, from, to)?;
    base::copy_base(conn, from, to)?;
    copy_files(conn, from, to)?;
    let decorations = decoration::copy_decorations(conn, from, to)?;
    design_decision::copy_design_decisions(conn, from, to, &decorations)?;
    moodboard::copy_style_moodboard(conn, from, to)?;
    cost_sheet::copy_cost_sheet(conn, from, to)?;
    Ok(())
}

/// Copies the design file records. The media files are shared: stored
/// media is content-addressed and never changes.
fn copy_files(conn: &Connection, from: StyleId, to: StyleId) -> CoreResult<()> {
    conn.execute(
        "INSERT INTO style_file (style_id, media, file_name, added_at)
         SELECT ?2, media, file_name, added_at FROM style_file WHERE style_id = ?1 ORDER BY id",
        params![from, to],
    )?;
    Ok(())
}
