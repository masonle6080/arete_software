//! A Style's base and sizes (spec #1, "Style base").
//!
//! A Style base is exactly one of a Blank or a Tech Pack. A Style has no
//! base until the founder sets one. A Style's sizes must be sizes its base
//! offers: a Blank-based Style picks from the Blank's sizes; a Tech Pack
//! Style takes every size of its Point of Measure table.
//!
//! To add a base kind: add a [`StyleBase`] variant, its column in
//! `style_base`, and an arm in `write_base`, `read_base`, `offered_sizes`,
//! and `style_sizes`.

use rusqlite::{params, Connection, OptionalExtension};

use super::{get_style, StyleId};
use crate::blank::{self, BlankId};
use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::tech_pack;

/// What a Style is built on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StyleBase {
    /// A ready-made garment model from the Blank library.
    Blank {
        #[serde(rename = "blankId")]
        blank_id: BlankId,
    },
    /// The Style's own Tech Pack (see [`crate::tech_pack`]).
    TechPack,
}

/// The base of a Style, or `None` when the founder has not set one.
pub fn style_base(lib: &Library, id: StyleId) -> CoreResult<Option<StyleBase>> {
    get_style(lib, id)?;
    read_base(lib.db(), id)
}

/// Sets the base of a Style, replacing any earlier base. A Blank-based
/// Style keeps only the sizes the new Blank also offers. Moving to a Tech
/// Pack starts an empty one, or brings back the Style's earlier Tech Pack.
pub fn set_style_base(lib: &Library, id: StyleId, base: StyleBase) -> CoreResult<StyleBase> {
    get_style(lib, id)?;
    let kept: Vec<String> = match base {
        StyleBase::Blank { blank_id } => {
            let offered = blank::blank_sizes(lib.db(), blank_id)?;
            style_sizes(lib, id)?.into_iter().filter(|s| offered.contains(s)).collect()
        }
        // The table gives the sizes; the Style does not pick any.
        StyleBase::TechPack => Vec::new(),
    };
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    write_base(&tx, id, base)?;
    write_sizes(&tx, id, &kept)?;
    tx.commit()?;
    Ok(base)
}

/// The sizes of a Style, in the order its base lists them. Empty when the
/// Style has no base.
pub fn style_sizes(lib: &Library, id: StyleId) -> CoreResult<Vec<String>> {
    get_style(lib, id)?;
    let conn = lib.db();
    let Some(base) = read_base(conn, id)? else { return Ok(Vec::new()) };
    let offered = offered_sizes(conn, id, base)?;
    match base {
        StyleBase::Blank { .. } => {
            let chosen = chosen_sizes(conn, id)?;
            Ok(offered.into_iter().filter(|s| chosen.contains(s)).collect())
        }
        StyleBase::TechPack => Ok(offered),
    }
}

/// Replaces the sizes of a Blank-based Style and returns them in base
/// order. Each size must be one the Blank offers, so the Style needs a base
/// first. A Tech Pack Style refuses: its sizes are its table's sizes.
pub fn set_style_sizes(lib: &Library, id: StyleId, sizes: &[String]) -> CoreResult<Vec<String>> {
    get_style(lib, id)?;
    let base = read_base(lib.db(), id)?
        .ok_or_else(|| CoreError::InvalidInput { reason: "set the Style's base before its sizes".into() })?;
    if base == StyleBase::TechPack {
        return Err(CoreError::InvalidInput {
            reason: "a Tech Pack Style takes its sizes from its Point of Measure table; edit the table".into(),
        });
    }
    let offered = offered_sizes(lib.db(), id, base)?;
    let sizes: Vec<String> = sizes.iter().map(|s| s.trim().to_string()).collect();
    let missing: Vec<&str> = sizes.iter().filter(|s| !offered.contains(s)).map(String::as_str).collect();
    if !missing.is_empty() {
        return Err(CoreError::InvalidInput {
            reason: format!("the Blank does not come in {}; choose from {}", missing.join(", "), offered.join(", ")),
        });
    }
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    write_sizes(&tx, id, &sizes)?;
    tx.commit()?;
    style_sizes(lib, id)
}

/// For each Style on a Blank, the sizes it uses, as (size, "<name> (<color>)").
pub(crate) fn blank_sizes_in_use(conn: &Connection, blank_id: BlankId) -> CoreResult<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT ss.size, s.name || ' (' || s.color || ')'
         FROM style_size ss
         JOIN style_base sb ON sb.style_id = ss.style_id
         JOIN style s ON s.id = ss.style_id
         WHERE sb.blank_id = ?1
         ORDER BY s.id, ss.size",
    )?;
    let rows = stmt.query_map([blank_id], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
    Ok(rows)
}

pub(crate) fn read_base(conn: &Connection, id: StyleId) -> CoreResult<Option<StyleBase>> {
    let row: Option<(Option<BlankId>, Option<StyleId>)> = conn
        .query_row("SELECT blank_id, tech_pack_id FROM style_base WHERE style_id = ?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .optional()?;
    Ok(match row {
        Some((Some(blank_id), _)) => Some(StyleBase::Blank { blank_id }),
        Some((None, Some(_))) => Some(StyleBase::TechPack),
        _ => None,
    })
}

/// Writes the one base reference and clears the other, so a row never
/// holds both.
fn write_base(conn: &Connection, id: StyleId, base: StyleBase) -> CoreResult<()> {
    let (blank_id, tech_pack_id) = match base {
        StyleBase::Blank { blank_id } => (Some(blank_id), None),
        StyleBase::TechPack => {
            tech_pack::ensure_row(conn, id)?;
            (None, Some(id))
        }
    };
    conn.execute(
        "INSERT INTO style_base (style_id, blank_id, tech_pack_id) VALUES (?1, ?2, ?3)
         ON CONFLICT (style_id) DO UPDATE SET blank_id = excluded.blank_id, tech_pack_id = excluded.tech_pack_id",
        params![id, blank_id, tech_pack_id],
    )?;
    Ok(())
}

/// Copies the base and chosen sizes of Style `from` onto the new Style
/// `to`. A Tech Pack base points to `to`'s own Tech Pack, so copy the Tech
/// Pack first (`tech_pack::copy_tech_pack`). Only `duplicate_style` calls
/// this, inside its transaction.
pub(super) fn copy_base(conn: &Connection, from: StyleId, to: StyleId) -> CoreResult<()> {
    if let Some(base) = read_base(conn, from)? {
        write_base(conn, to, base)?;
    }
    conn.execute(
        "INSERT INTO style_size (style_id, size) SELECT ?2, size FROM style_size WHERE style_id = ?1",
        params![from, to],
    )?;
    Ok(())
}

/// The sizes a base offers, in its order. Fails when the base record does
/// not exist.
fn offered_sizes(conn: &Connection, id: StyleId, base: StyleBase) -> CoreResult<Vec<String>> {
    match base {
        StyleBase::Blank { blank_id } => blank::blank_sizes(conn, blank_id),
        StyleBase::TechPack => tech_pack::table_sizes(conn, id),
    }
}

fn chosen_sizes(conn: &Connection, id: StyleId) -> CoreResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT size FROM style_size WHERE style_id = ?1")?;
    let sizes = stmt.query_map([id], |r| r.get(0))?.collect::<Result<_, _>>()?;
    Ok(sizes)
}

fn write_sizes(conn: &Connection, id: StyleId, sizes: &[String]) -> CoreResult<()> {
    conn.execute("DELETE FROM style_size WHERE style_id = ?1", [id])?;
    for size in sizes {
        conn.execute("INSERT OR IGNORE INTO style_size (style_id, size) VALUES (?1, ?2)", params![id, size])?;
    }
    Ok(())
}
