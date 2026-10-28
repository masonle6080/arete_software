//! Vendor: a library-wide company that supplies Blanks, materials,
//! Decoration, or manufacturing (GLOSSARY: Vendor). One Vendor can fill
//! several roles.
//!
//! Domain command functions for this area: [`create_vendor`],
//! [`update_vendor`], [`get_vendor`], [`list_vendors`].

use rusqlite::{Connection, OptionalExtension};

use crate::error::{CoreError, CoreResult};
use crate::library::Library;

pub type VendorId = i64;

/// A role a Vendor fills. Listed in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VendorRole {
    BlankSupplier,
    Decorator,
    Manufacturer,
    Mill,
}

impl VendorRole {
    pub const ALL: [VendorRole; 4] =
        [VendorRole::BlankSupplier, VendorRole::Decorator, VendorRole::Manufacturer, VendorRole::Mill];

    fn db_name(self) -> &'static str {
        match self {
            VendorRole::BlankSupplier => "blank_supplier",
            VendorRole::Decorator => "decorator",
            VendorRole::Manufacturer => "manufacturer",
            VendorRole::Mill => "mill",
        }
    }

    fn from_db_name(name: &str) -> Option<VendorRole> {
        VendorRole::ALL.into_iter().find(|r| r.db_name() == name)
    }
}

/// A Vendor as the founder sees it. Roles are unique and in display order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vendor {
    pub id: VendorId,
    pub name: String,
    pub roles: Vec<VendorRole>,
}

/// The fields the founder sets when creating or editing a Vendor.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorInput {
    pub name: String,
    pub roles: Vec<VendorRole>,
}

/// Creates a Vendor.
pub fn create_vendor(lib: &Library, input: VendorInput) -> CoreResult<Vendor> {
    let input = validate(input)?;
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    tx.execute("INSERT INTO vendor (name) VALUES (?1)", [&input.name])?;
    let id = tx.last_insert_rowid();
    write_roles(&tx, id, &input.roles)?;
    tx.commit()?;
    get_vendor(lib, id)
}

/// Replaces a Vendor's name and roles.
pub fn update_vendor(lib: &Library, id: VendorId, input: VendorInput) -> CoreResult<Vendor> {
    let input = validate(input)?;
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    let changed = tx.execute("UPDATE vendor SET name = ?1 WHERE id = ?2", (&input.name, id))?;
    if changed == 0 {
        return Err(CoreError::NotFound { record: "Vendor", id });
    }
    tx.execute("DELETE FROM vendor_role WHERE vendor_id = ?1", [id])?;
    write_roles(&tx, id, &input.roles)?;
    tx.commit()?;
    get_vendor(lib, id)
}

/// One Vendor by ID.
pub fn get_vendor(lib: &Library, id: VendorId) -> CoreResult<Vendor> {
    let conn = lib.db();
    let name: Option<String> =
        conn.query_row("SELECT name FROM vendor WHERE id = ?1", [id], |r| r.get(0)).optional()?;
    let name = name.ok_or(CoreError::NotFound { record: "Vendor", id })?;
    Ok(Vendor { id, name, roles: read_roles(conn, id)? })
}

/// All Vendors, sorted by name.
pub fn list_vendors(lib: &Library) -> CoreResult<Vec<Vendor>> {
    let conn = lib.db();
    let mut stmt = conn.prepare("SELECT id, name FROM vendor ORDER BY name COLLATE NOCASE, id")?;
    let rows: Vec<(VendorId, String)> =
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
    rows.into_iter()
        .map(|(id, name)| Ok(Vendor { id, name, roles: read_roles(conn, id)? }))
        .collect()
}

/// Trims the name and refuses a Vendor with no name or no role.
fn validate(input: VendorInput) -> CoreResult<VendorInput> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Vendor needs a name".into() });
    }
    if input.roles.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Vendor needs at least one role".into() });
    }
    Ok(VendorInput { name, roles: input.roles })
}

fn write_roles(conn: &Connection, id: VendorId, roles: &[VendorRole]) -> CoreResult<()> {
    for role in roles {
        conn.execute(
            "INSERT OR IGNORE INTO vendor_role (vendor_id, role) VALUES (?1, ?2)",
            (id, role.db_name()),
        )?;
    }
    Ok(())
}

fn read_roles(conn: &Connection, id: VendorId) -> CoreResult<Vec<VendorRole>> {
    let mut stmt = conn.prepare("SELECT role FROM vendor_role WHERE vendor_id = ?1")?;
    let names: Vec<String> = stmt.query_map([id], |r| r.get(0))?.collect::<Result<_, _>>()?;
    let mut roles: Vec<VendorRole> = names.iter().filter_map(|n| VendorRole::from_db_name(n)).collect();
    roles.sort();
    Ok(roles)
}
