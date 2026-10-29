//! Blank: a library-wide, ready-made garment model bought from a Vendor
//! (GLOSSARY: Blank). A Style can use a Blank as its base (see
//! `style/base.rs`), and a Blank can be reused across Drops.
//!
//! Domain command functions for this area: [`create_blank`],
//! [`update_blank`], [`get_blank`], [`list_blanks`].

use rusqlite::{Connection, OptionalExtension};

use crate::error::{CoreError, CoreResult};
use crate::library::{self, Library};
use crate::style;
use crate::vendor::VendorId;

pub type BlankId = i64;

/// A Blank as the founder sees it. Colors and sizes are unique and in the
/// order the founder listed them.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Blank {
    pub id: BlankId,
    pub brand: String,
    pub model: String,
    pub vendor_id: VendorId,
    /// The Vendor's name, for display.
    pub vendor_name: String,
    pub colors: Vec<String>,
    pub sizes: Vec<String>,
    /// Unit cost in minor units of `currency` (cents); `None` when unknown.
    pub cost_cents: Option<i64>,
    /// Free text such as "USD"; empty when unknown.
    pub currency: String,
    pub notes: String,
}

/// The fields the founder sets when creating or editing a Blank. An edit
/// replaces every field.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlankInput {
    pub brand: String,
    pub model: String,
    pub vendor_id: VendorId,
    #[serde(default)]
    pub colors: Vec<String>,
    pub sizes: Vec<String>,
    #[serde(default)]
    pub cost_cents: Option<i64>,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub notes: String,
}

/// Adds a Blank to the library.
pub fn create_blank(lib: &Library, input: BlankInput) -> CoreResult<Blank> {
    let input = validate(lib.db(), input)?;
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO blank (brand, model, vendor_id, cost_cents, currency, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (&input.brand, &input.model, input.vendor_id, input.cost_cents, &input.currency, &input.notes),
    )?;
    let id = tx.last_insert_rowid();
    write_lists(&tx, id, &input)?;
    tx.commit()?;
    get_blank(lib, id)
}

/// Replaces every field of a Blank. Refuses to remove a size that a Style
/// on this Blank uses; take the size off those Styles first.
pub fn update_blank(lib: &Library, id: BlankId, input: BlankInput) -> CoreResult<Blank> {
    get_blank(lib, id)?;
    let input = validate(lib.db(), input)?;
    let in_use: Vec<String> = style::blank_sizes_in_use(lib.db(), id)?
        .into_iter()
        .filter(|(size, _)| !input.sizes.contains(size))
        .map(|(size, style)| format!("{size} ({style})"))
        .collect();
    if !in_use.is_empty() {
        return Err(CoreError::InvalidInput {
            reason: format!("Styles use sizes you removed; take them off the Styles first: {}", in_use.join(", ")),
        });
    }
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE blank SET brand = ?1, model = ?2, vendor_id = ?3, cost_cents = ?4, currency = ?5, notes = ?6
         WHERE id = ?7",
        (&input.brand, &input.model, input.vendor_id, input.cost_cents, &input.currency, &input.notes, id),
    )?;
    tx.execute("DELETE FROM blank_color WHERE blank_id = ?1", [id])?;
    tx.execute("DELETE FROM blank_size WHERE blank_id = ?1", [id])?;
    write_lists(&tx, id, &input)?;
    tx.commit()?;
    get_blank(lib, id)
}

const BLANK_SELECT: &str = "SELECT b.id, b.brand, b.model, b.vendor_id, v.name, b.cost_cents, b.currency, b.notes
     FROM blank b JOIN vendor v ON v.id = b.vendor_id";

/// One Blank by ID.
pub fn get_blank(lib: &Library, id: BlankId) -> CoreResult<Blank> {
    let conn = lib.db();
    let blank = conn.query_row(&format!("{BLANK_SELECT} WHERE b.id = ?1"), [id], read_row).optional()?;
    complete(conn, blank.ok_or(CoreError::NotFound { record: "Blank", id })?)
}

/// All Blanks, sorted by brand, then model.
pub fn list_blanks(lib: &Library) -> CoreResult<Vec<Blank>> {
    let conn = lib.db();
    let mut stmt =
        conn.prepare(&format!("{BLANK_SELECT} ORDER BY b.brand COLLATE NOCASE, b.model COLLATE NOCASE, b.id"))?;
    let rows: Vec<Blank> = stmt.query_map([], read_row)?.collect::<Result<_, _>>()?;
    rows.into_iter().map(|b| complete(conn, b)).collect()
}

/// The sizes a Blank comes in, in order. Fails when the Blank does not exist.
pub(crate) fn blank_sizes(conn: &Connection, id: BlankId) -> CoreResult<Vec<String>> {
    library::require_row(conn, "blank", "Blank", id)?;
    read_list(conn, "SELECT size FROM blank_size WHERE blank_id = ?1 ORDER BY position", id)
}

fn read_row(r: &rusqlite::Row) -> rusqlite::Result<Blank> {
    Ok(Blank {
        id: r.get(0)?,
        brand: r.get(1)?,
        model: r.get(2)?,
        vendor_id: r.get(3)?,
        vendor_name: r.get(4)?,
        cost_cents: r.get(5)?,
        currency: r.get(6)?,
        notes: r.get(7)?,
        colors: Vec::new(),
        sizes: Vec::new(),
    })
}

fn complete(conn: &Connection, mut blank: Blank) -> CoreResult<Blank> {
    blank.colors = read_list(conn, "SELECT color FROM blank_color WHERE blank_id = ?1 ORDER BY position", blank.id)?;
    blank.sizes = blank_sizes(conn, blank.id)?;
    Ok(blank)
}

fn read_list(conn: &Connection, sql: &str, id: BlankId) -> CoreResult<Vec<String>> {
    let mut stmt = conn.prepare(sql)?;
    let items = stmt.query_map([id], |r| r.get(0))?.collect::<Result<_, _>>()?;
    Ok(items)
}

fn write_lists(conn: &Connection, id: BlankId, input: &BlankInput) -> CoreResult<()> {
    for (position, color) in input.colors.iter().enumerate() {
        conn.execute(
            "INSERT INTO blank_color (blank_id, position, color) VALUES (?1, ?2, ?3)",
            (id, position as i64, color),
        )?;
    }
    for (position, size) in input.sizes.iter().enumerate() {
        conn.execute(
            "INSERT INTO blank_size (blank_id, position, size) VALUES (?1, ?2, ?3)",
            (id, position as i64, size),
        )?;
    }
    Ok(())
}

/// Trims text, drops empty and repeated colors and sizes. Refuses a Blank
/// with no brand, no model, an unknown Vendor, no size, or a negative cost.
fn validate(conn: &Connection, input: BlankInput) -> CoreResult<BlankInput> {
    let invalid = |reason: &str| Err(CoreError::InvalidInput { reason: reason.into() });
    let (brand, model) = (input.brand.trim().to_string(), input.model.trim().to_string());
    if brand.is_empty() {
        return invalid("a Blank needs a brand");
    }
    if model.is_empty() {
        return invalid("a Blank needs a model");
    }
    library::require_row(conn, "vendor", "Vendor", input.vendor_id)?;
    let sizes = clean_list(&input.sizes);
    if sizes.is_empty() {
        return invalid("a Blank needs at least one size");
    }
    if input.cost_cents.is_some_and(|c| c < 0) {
        return invalid("a cost cannot be negative");
    }
    Ok(BlankInput {
        brand,
        model,
        vendor_id: input.vendor_id,
        colors: clean_list(&input.colors),
        sizes,
        cost_cents: input.cost_cents,
        currency: input.currency.trim().to_string(),
        notes: input.notes.trim().to_string(),
    })
}

/// Trimmed, non-empty, unique items, in first-seen order.
fn clean_list(items: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in items.iter().map(|i| i.trim()).filter(|i| !i.is_empty()) {
        if !out.iter().any(|o| o == item) {
            out.push(item.to_string());
        }
    }
    out
}
