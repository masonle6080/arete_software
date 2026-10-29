//! Tech Pack: the full specification for a Style that is not built on a
//! Blank (GLOSSARY: Tech Pack, Point of Measure).
//!
//! A Style's base is a Tech Pack after
//! `style::set_style_base(.., StyleBase::TechPack)` (see `style/base.rs`,
//! which enforces that a base is exactly one of a Blank or a Tech Pack).
//! While the base is something else, [`get_tech_pack`] returns `None` and
//! every edit here is refused; the stored Tech Pack comes back if the base
//! returns to it.
//!
//! The Tech Pack holds a table of Points of Measure (rows) by sizes
//! (columns), each cell a value, each row a tolerance. The Style takes its
//! sizes from those columns (`style::style_sizes`).
//!
//! Domain command functions: [`get_tech_pack`], [`add_size`],
//! [`rename_size`], [`remove_size`], [`add_point_of_measure`],
//! [`update_point_of_measure`], [`set_point_of_measure_value`],
//! [`remove_point_of_measure`], [`update_tech_pack_details`],
//! [`attach_flats`].

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::media::{self, MediaRef};
use crate::style::{self, StyleBase, StyleId};
use crate::vendor::{self, VendorId};

/// A Tech Pack as the founder sees it. Sizes and Points of Measure are in
/// the order the founder added them.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TechPack {
    pub style_id: StyleId,
    /// The sizes: the columns of the Point of Measure table.
    pub sizes: Vec<TechPackSize>,
    /// The rows of the Point of Measure table.
    pub points_of_measure: Vec<PointOfMeasure>,
    pub details: TechPackDetails,
    /// Flat drawings, in the order attached.
    pub flats: Vec<TechPackFlat>,
}

/// The written parts of a Tech Pack. An edit replaces every field. Text
/// fields are empty when not yet written.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TechPackDetails {
    /// The unit of every Point of Measure value and tolerance, such as "cm".
    pub unit: String,
    pub fabric: Fabric,
    pub trims: String,
    pub construction_notes: String,
    pub label_placement: String,
}

/// The main fabric of a Tech Pack.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fabric {
    /// For example "100% cotton loopback".
    pub composition: String,
    /// Free text, for example "450 gsm".
    pub weight: String,
    pub color: String,
    /// The Vendor the fabric comes from, if known.
    pub vendor_id: Option<VendorId>,
}

/// One flat drawing attached to a Tech Pack.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TechPackFlat {
    pub id: i64,
    pub media: MediaRef,
    /// The name the file had when the founder attached it.
    pub file_name: String,
    /// Where the stored copy is now, inside the library's media folder.
    pub path: PathBuf,
}

pub type SizeId = i64;
pub type PointOfMeasureId = i64;

/// One size of a Tech Pack. The ID is stable across renames.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TechPackSize {
    pub id: SizeId,
    pub name: String,
}

/// One named measurement, such as chest width. The ID is stable across
/// edits, so exports and fit notes can refer to it.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointOfMeasure {
    pub id: PointOfMeasureId,
    pub name: String,
    /// The allowed deviation either way (plus or minus), in the Tech Pack's
    /// unit. Zero or more.
    pub tolerance: f64,
    /// One entry per size, in the order of [`TechPack::sizes`]. `None` is an
    /// empty cell.
    pub values: Vec<Option<f64>>,
}

/// Starts an empty Tech Pack for a Style, unless it already has one. Only
/// `style::set_style_base` calls this, inside its write.
pub(crate) fn ensure_row(conn: &Connection, style_id: StyleId) -> CoreResult<()> {
    conn.execute("INSERT OR IGNORE INTO tech_pack (style_id) VALUES (?1)", [style_id])?;
    Ok(())
}

/// The size names of a Style's Tech Pack table, in table order. Empty when
/// it has no Tech Pack. Callers use `style::style_sizes`.
pub(crate) fn table_sizes(conn: &Connection, style_id: StyleId) -> CoreResult<Vec<String>> {
    Ok(sizes(conn, style_id)?.into_iter().map(|z| z.name).collect())
}

/// The Tech Pack of a Style, or `None` when its base is not a Tech Pack.
pub fn get_tech_pack(lib: &Library, style_id: StyleId) -> CoreResult<Option<TechPack>> {
    style::get_style(lib, style_id)?;
    if style::read_base(lib.db(), style_id)? != Some(StyleBase::TechPack) {
        return Ok(None);
    }
    let details = lib
        .db()
        .query_row(
            "SELECT unit, fabric_composition, fabric_weight, fabric_color, fabric_vendor_id,
                    trims, construction_notes, label_placement
             FROM tech_pack WHERE style_id = ?1",
            [style_id],
            |r| {
                Ok(TechPackDetails {
                    unit: r.get(0)?,
                    fabric: Fabric { composition: r.get(1)?, weight: r.get(2)?, color: r.get(3)?, vendor_id: r.get(4)? },
                    trims: r.get(5)?,
                    construction_notes: r.get(6)?,
                    label_placement: r.get(7)?,
                })
            },
        )
        .optional()?;
    let Some(details) = details else { return Ok(None) };
    let sizes = sizes(lib.db(), style_id)?;
    let points_of_measure = points_of_measure(lib, style_id, &sizes)?;
    let flats = flats(lib, style_id)?;
    Ok(Some(TechPack { style_id, sizes, points_of_measure, details, flats }))
}

/// Replaces the written parts of a Tech Pack: unit, fabric, trims,
/// construction notes, and label placement. Text is trimmed. The unit must
/// not be blank; the fabric Vendor, when given, must exist.
pub fn update_tech_pack_details(lib: &Library, style_id: StyleId, details: TechPackDetails) -> CoreResult<TechPack> {
    require(lib, style_id)?;
    let unit = details.unit.trim();
    if unit.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Tech Pack needs a unit of measure, such as cm".into() });
    }
    if let Some(vendor_id) = details.fabric.vendor_id {
        vendor::get_vendor(lib, vendor_id)?;
    }
    let f = &details.fabric;
    lib.writer()?.execute(
        "UPDATE tech_pack SET unit = ?1, fabric_composition = ?2, fabric_weight = ?3, fabric_color = ?4,
                fabric_vendor_id = ?5, trims = ?6, construction_notes = ?7, label_placement = ?8
         WHERE style_id = ?9",
        params![
            unit,
            f.composition.trim(),
            f.weight.trim(),
            f.color.trim(),
            f.vendor_id,
            details.trims.trim(),
            details.construction_notes.trim(),
            details.label_placement.trim(),
            style_id
        ],
    )?;
    require(lib, style_id)
}

/// Attaches flat drawings to a Tech Pack through the media import path.
/// `sources` are files outside the library; the core stores a copy of each.
pub fn attach_flats(lib: &Library, style_id: StyleId, sources: &[PathBuf]) -> CoreResult<TechPack> {
    require(lib, style_id)?;
    let conn = lib.writer()?;
    let imported: Vec<(MediaRef, String)> = sources
        .iter()
        .map(|source| Ok((media::import_media(lib, source)?, display_name(source))))
        .collect::<CoreResult<_>>()?;
    let tx = conn.unchecked_transaction()?;
    for (media, file_name) in &imported {
        tx.execute(
            "INSERT INTO tech_pack_flat (style_id, media, file_name) VALUES (?1, ?2, ?3)",
            params![style_id, media.file_name(), file_name],
        )?;
    }
    tx.commit()?;
    require(lib, style_id)
}

fn display_name(source: &Path) -> String {
    source.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| source.display().to_string())
}

fn flats(lib: &Library, style_id: StyleId) -> CoreResult<Vec<TechPackFlat>> {
    let mut stmt =
        lib.db().prepare("SELECT id, media, file_name FROM tech_pack_flat WHERE style_id = ?1 ORDER BY id")?;
    let rows: Vec<(i64, String, String)> =
        stmt.query_map([style_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?.collect::<Result<_, _>>()?;
    Ok(rows
        .into_iter()
        .map(|(id, stored, file_name)| {
            let media = MediaRef::from_stored(stored);
            let path = media::media_path(lib, &media);
            TechPackFlat { id, media, file_name, path }
        })
        .collect())
}

/// Adds a Point of Measure (a new row of the table) at the end, with every
/// value empty. The name must not be blank; the tolerance must be zero or
/// more.
pub fn add_point_of_measure(lib: &Library, style_id: StyleId, name: &str, tolerance: f64) -> CoreResult<TechPack> {
    require(lib, style_id)?;
    let (name, tolerance) = valid_point_of_measure(name, tolerance)?;
    lib.writer()?.execute(
        "INSERT INTO tech_pack_pom (style_id, name, tolerance) VALUES (?1, ?2, ?3)",
        params![style_id, name, tolerance],
    )?;
    require(lib, style_id)
}

/// Changes the name and tolerance of a Point of Measure. Its ID, its
/// place in the table, and its values stay.
pub fn update_point_of_measure(lib: &Library, id: PointOfMeasureId, name: &str, tolerance: f64) -> CoreResult<TechPack> {
    let style_id = point_of_measure_owner(lib, id)?;
    let (name, tolerance) = valid_point_of_measure(name, tolerance)?;
    lib.writer()?
        .execute("UPDATE tech_pack_pom SET name = ?1, tolerance = ?2 WHERE id = ?3", params![name, tolerance, id])?;
    require(lib, style_id)
}

/// Sets or clears (`None`) the value of a Point of Measure in one size of
/// the same Tech Pack.
pub fn set_point_of_measure_value(
    lib: &Library,
    id: PointOfMeasureId,
    size_id: SizeId,
    value: Option<f64>,
) -> CoreResult<TechPack> {
    let style_id = point_of_measure_owner(lib, id)?;
    let size_style = size_owner(lib, size_id).ok();
    if size_style != Some(style_id) {
        return Err(CoreError::InvalidInput { reason: "that size is not in this Tech Pack".into() });
    }
    if value.is_some_and(|v| !v.is_finite()) {
        return Err(CoreError::InvalidInput { reason: "a measurement must be a number".into() });
    }
    let conn = lib.writer()?;
    match value {
        Some(v) => conn.execute(
            "INSERT INTO tech_pack_pom_value (pom_id, size_id, value) VALUES (?1, ?2, ?3)
             ON CONFLICT (pom_id, size_id) DO UPDATE SET value = excluded.value",
            params![id, size_id, v],
        )?,
        None => conn.execute(
            "DELETE FROM tech_pack_pom_value WHERE pom_id = ?1 AND size_id = ?2",
            params![id, size_id],
        )?,
    };
    require(lib, style_id)
}

/// Removes a Point of Measure and its values from the table.
pub fn remove_point_of_measure(lib: &Library, id: PointOfMeasureId) -> CoreResult<TechPack> {
    let style_id = point_of_measure_owner(lib, id)?;
    lib.writer()?.execute("DELETE FROM tech_pack_pom WHERE id = ?1", [id])?;
    require(lib, style_id)
}

fn valid_point_of_measure(name: &str, tolerance: f64) -> CoreResult<(&str, f64)> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Point of Measure needs a name".into() });
    }
    if !(tolerance.is_finite() && tolerance >= 0.0) {
        return Err(CoreError::InvalidInput { reason: "a tolerance must be a number of zero or more".into() });
    }
    Ok((name, tolerance))
}

/// The Style of a Point of Measure. Refuses when that Style's base is no
/// longer its Tech Pack.
fn point_of_measure_owner(lib: &Library, id: PointOfMeasureId) -> CoreResult<StyleId> {
    let style_id = lib
        .db()
        .query_row("SELECT style_id FROM tech_pack_pom WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .ok_or(CoreError::NotFound { record: "Point of Measure", id })?;
    require(lib, style_id)?;
    Ok(style_id)
}

fn points_of_measure(lib: &Library, style_id: StyleId, sizes: &[TechPackSize]) -> CoreResult<Vec<PointOfMeasure>> {
    let mut stmt = lib.db().prepare("SELECT id, name, tolerance FROM tech_pack_pom WHERE style_id = ?1 ORDER BY id")?;
    let mut poms: Vec<PointOfMeasure> = stmt
        .query_map([style_id], |r| {
            Ok(PointOfMeasure { id: r.get(0)?, name: r.get(1)?, tolerance: r.get(2)?, values: vec![None; sizes.len()] })
        })?
        .collect::<Result<_, _>>()?;
    let mut stmt = lib.db().prepare(
        "SELECT v.pom_id, v.size_id, v.value FROM tech_pack_pom_value v
         JOIN tech_pack_pom p ON p.id = v.pom_id WHERE p.style_id = ?1",
    )?;
    let cells: Vec<(PointOfMeasureId, SizeId, f64)> =
        stmt.query_map([style_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?.collect::<Result<_, _>>()?;
    for (pom_id, size_id, value) in cells {
        let row = poms.iter_mut().find(|p| p.id == pom_id);
        let col = sizes.iter().position(|z| z.id == size_id);
        if let (Some(row), Some(col)) = (row, col) {
            row.values[col] = Some(value);
        }
    }
    Ok(poms)
}

/// Adds a size (a new column of the Point of Measure table) at the end.
/// The name must not be blank and must differ from the other sizes.
pub fn add_size(lib: &Library, style_id: StyleId, name: &str) -> CoreResult<TechPack> {
    let tp = require(lib, style_id)?;
    let name = valid_size_name(&tp, name, None)?;
    lib.writer()?.execute("INSERT INTO tech_pack_size (style_id, name) VALUES (?1, ?2)", params![style_id, name])?;
    require(lib, style_id)
}

/// Renames a size. Its values and its ID stay.
pub fn rename_size(lib: &Library, size_id: SizeId, name: &str) -> CoreResult<TechPack> {
    let tp = require(lib, size_owner(lib, size_id)?)?;
    let name = valid_size_name(&tp, name, Some(size_id))?;
    lib.writer()?.execute("UPDATE tech_pack_size SET name = ?1 WHERE id = ?2", params![name, size_id])?;
    require(lib, tp.style_id)
}

/// Removes a size and its values from the table.
pub fn remove_size(lib: &Library, size_id: SizeId) -> CoreResult<TechPack> {
    let style_id = size_owner(lib, size_id)?;
    require(lib, style_id)?;
    lib.writer()?.execute("DELETE FROM tech_pack_size WHERE id = ?1", [size_id])?;
    require(lib, style_id)
}

fn valid_size_name<'a>(tp: &TechPack, name: &'a str, renaming: Option<SizeId>) -> CoreResult<&'a str> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a size needs a name".into() });
    }
    if tp.sizes.iter().any(|z| z.name == name && Some(z.id) != renaming) {
        return Err(CoreError::InvalidInput { reason: format!("this Tech Pack already has a size {name}") });
    }
    Ok(name)
}

fn size_owner(lib: &Library, size_id: SizeId) -> CoreResult<StyleId> {
    lib.db()
        .query_row("SELECT style_id FROM tech_pack_size WHERE id = ?1", [size_id], |r| r.get(0))
        .optional()?
        .ok_or(CoreError::NotFound { record: "Tech Pack size", id: size_id })
}

fn sizes(conn: &Connection, style_id: StyleId) -> CoreResult<Vec<TechPackSize>> {
    let mut stmt = conn.prepare("SELECT id, name FROM tech_pack_size WHERE style_id = ?1 ORDER BY id")?;
    let sizes = stmt
        .query_map([style_id], |r| Ok(TechPackSize { id: r.get(0)?, name: r.get(1)? }))?
        .collect::<Result<_, _>>()?;
    Ok(sizes)
}

/// The Tech Pack of a Style; refuses when its base is not a Tech Pack.
fn require(lib: &Library, style_id: StyleId) -> CoreResult<TechPack> {
    get_tech_pack(lib, style_id)?.ok_or_else(|| CoreError::InvalidInput {
        reason: "set this Style's base to a Tech Pack first".into(),
    })
}

/// Copies the Tech Pack of Style `from`, if it has one, onto the new Style
/// `to`, with new size and Point of Measure IDs. Only
/// `style::duplicate_style` calls this, inside its transaction.
pub(crate) fn copy_tech_pack(conn: &Connection, from: StyleId, to: StyleId) -> CoreResult<()> {
    let copied = conn.execute(
        "INSERT INTO tech_pack (style_id, unit, fabric_composition, fabric_weight, fabric_color, fabric_vendor_id,
                                trims, construction_notes, label_placement)
         SELECT ?2, unit, fabric_composition, fabric_weight, fabric_color, fabric_vendor_id,
                trims, construction_notes, label_placement
         FROM tech_pack WHERE style_id = ?1",
        params![from, to],
    )?;
    if copied == 0 {
        return Ok(());
    }
    let mut size_ids: HashMap<SizeId, SizeId> = HashMap::new();
    for size in sizes(conn, from)? {
        conn.execute("INSERT INTO tech_pack_size (style_id, name) VALUES (?1, ?2)", params![to, size.name])?;
        size_ids.insert(size.id, conn.last_insert_rowid());
    }
    let mut stmt = conn.prepare("SELECT id, name, tolerance FROM tech_pack_pom WHERE style_id = ?1 ORDER BY id")?;
    let poms: Vec<(PointOfMeasureId, String, f64)> =
        stmt.query_map([from], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?.collect::<Result<_, _>>()?;
    let mut values = conn.prepare("SELECT size_id, value FROM tech_pack_pom_value WHERE pom_id = ?1")?;
    for (old_id, name, tolerance) in poms {
        conn.execute(
            "INSERT INTO tech_pack_pom (style_id, name, tolerance) VALUES (?1, ?2, ?3)",
            params![to, name, tolerance],
        )?;
        let new_id = conn.last_insert_rowid();
        let cells: Vec<(SizeId, f64)> =
            values.query_map([old_id], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
        for (size_id, value) in cells {
            if let Some(new_size) = size_ids.get(&size_id) {
                conn.execute(
                    "INSERT INTO tech_pack_pom_value (pom_id, size_id, value) VALUES (?1, ?2, ?3)",
                    params![new_id, new_size, value],
                )?;
            }
        }
    }
    // Flat drawings share their media: stored media never changes.
    conn.execute(
        "INSERT INTO tech_pack_flat (style_id, media, file_name, added_at)
         SELECT ?2, media, file_name, added_at FROM tech_pack_flat WHERE style_id = ?1 ORDER BY id",
        params![from, to],
    )?;
    Ok(())
}
