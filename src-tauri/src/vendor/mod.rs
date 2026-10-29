//! Vendor: a library-wide company that supplies Blanks, materials,
//! Decoration, or manufacturing (GLOSSARY: Vendor). One Vendor can fill
//! several roles. A Vendor also holds contact people, a location, a minimum
//! order quantity, and a typical lead time. Its dated notes and attachments
//! live in the shared Notes record ([`crate::note`], owner `NoteOwner::Vendor`).
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

/// A contact person at a Vendor. Only the name is required; the other
/// fields are empty strings when unknown.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VendorContact {
    pub name: String,
    /// The person's job, or what the founder talks to them about.
    pub role: String,
    pub email: String,
    pub phone: String,
}

/// A Vendor as the founder sees it. Roles are unique and in display order.
/// Contacts are in the order the founder listed them.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vendor {
    pub id: VendorId,
    pub name: String,
    pub roles: Vec<VendorRole>,
    pub contacts: Vec<VendorContact>,
    /// Free text, empty when unknown.
    pub location: String,
    /// In pieces; `None` when unknown.
    pub minimum_order_quantity: Option<u32>,
    /// Free text such as "4-6 weeks", empty when unknown.
    pub lead_time: String,
}

/// The fields the founder sets when creating or editing a Vendor. An edit
/// replaces every field.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorInput {
    pub name: String,
    pub roles: Vec<VendorRole>,
    #[serde(default)]
    pub contacts: Vec<VendorContact>,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub minimum_order_quantity: Option<u32>,
    #[serde(default)]
    pub lead_time: String,
}

/// Creates a Vendor.
pub fn create_vendor(lib: &Library, input: VendorInput) -> CoreResult<Vendor> {
    let input = validate(input)?;
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO vendor (name, location, minimum_order_quantity, lead_time) VALUES (?1, ?2, ?3, ?4)",
        (&input.name, &input.location, input.minimum_order_quantity, &input.lead_time),
    )?;
    let id = tx.last_insert_rowid();
    write_roles(&tx, id, &input.roles)?;
    write_contacts(&tx, id, &input.contacts)?;
    tx.commit()?;
    get_vendor(lib, id)
}

/// Replaces a Vendor's name, roles, contacts, and details.
pub fn update_vendor(lib: &Library, id: VendorId, input: VendorInput) -> CoreResult<Vendor> {
    let input = validate(input)?;
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    let changed = tx.execute(
        "UPDATE vendor SET name = ?1, location = ?2, minimum_order_quantity = ?3, lead_time = ?4 WHERE id = ?5",
        (&input.name, &input.location, input.minimum_order_quantity, &input.lead_time, id),
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound { record: "Vendor", id });
    }
    tx.execute("DELETE FROM vendor_role WHERE vendor_id = ?1", [id])?;
    write_roles(&tx, id, &input.roles)?;
    tx.execute("DELETE FROM vendor_contact WHERE vendor_id = ?1", [id])?;
    write_contacts(&tx, id, &input.contacts)?;
    tx.commit()?;
    get_vendor(lib, id)
}

const VENDOR_COLUMNS: &str = "id, name, location, minimum_order_quantity, lead_time";

/// One Vendor by ID.
pub fn get_vendor(lib: &Library, id: VendorId) -> CoreResult<Vendor> {
    let conn = lib.db();
    let vendor = conn
        .query_row(&format!("SELECT {VENDOR_COLUMNS} FROM vendor WHERE id = ?1"), [id], read_vendor_row)
        .optional()?;
    let vendor = vendor.ok_or(CoreError::NotFound { record: "Vendor", id })?;
    complete(conn, vendor)
}

/// All Vendors, sorted by name.
pub fn list_vendors(lib: &Library) -> CoreResult<Vec<Vendor>> {
    let conn = lib.db();
    let mut stmt = conn.prepare(&format!("SELECT {VENDOR_COLUMNS} FROM vendor ORDER BY name COLLATE NOCASE, id"))?;
    let rows: Vec<Vendor> = stmt.query_map([], read_vendor_row)?.collect::<Result<_, _>>()?;
    rows.into_iter().map(|v| complete(conn, v)).collect()
}

/// The Vendor row, without roles and contacts.
fn read_vendor_row(r: &rusqlite::Row) -> rusqlite::Result<Vendor> {
    Ok(Vendor {
        id: r.get(0)?,
        name: r.get(1)?,
        location: r.get(2)?,
        minimum_order_quantity: r.get(3)?,
        lead_time: r.get(4)?,
        roles: Vec::new(),
        contacts: Vec::new(),
    })
}

fn complete(conn: &Connection, mut vendor: Vendor) -> CoreResult<Vendor> {
    vendor.roles = read_roles(conn, vendor.id)?;
    vendor.contacts = read_contacts(conn, vendor.id)?;
    Ok(vendor)
}

/// Trims text fields. Refuses a Vendor with no name or no role, a contact
/// with no name, and a minimum order quantity of zero.
fn validate(input: VendorInput) -> CoreResult<VendorInput> {
    let invalid = |reason: &str| Err(CoreError::InvalidInput { reason: reason.into() });
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return invalid("a Vendor needs a name");
    }
    if input.roles.is_empty() {
        return invalid("a Vendor needs at least one role");
    }
    if input.minimum_order_quantity == Some(0) {
        return invalid("a minimum order quantity must be at least 1");
    }
    let contacts: Vec<VendorContact> = input
        .contacts
        .iter()
        .map(|c| VendorContact {
            name: c.name.trim().to_string(),
            role: c.role.trim().to_string(),
            email: c.email.trim().to_string(),
            phone: c.phone.trim().to_string(),
        })
        .collect();
    if contacts.iter().any(|c| c.name.is_empty()) {
        return invalid("a contact needs a name");
    }
    Ok(VendorInput {
        name,
        roles: input.roles,
        contacts,
        location: input.location.trim().to_string(),
        minimum_order_quantity: input.minimum_order_quantity,
        lead_time: input.lead_time.trim().to_string(),
    })
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

fn write_contacts(conn: &Connection, id: VendorId, contacts: &[VendorContact]) -> CoreResult<()> {
    for (position, c) in contacts.iter().enumerate() {
        conn.execute(
            "INSERT INTO vendor_contact (vendor_id, position, name, role, email, phone)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (id, position as i64, &c.name, &c.role, &c.email, &c.phone),
        )?;
    }
    Ok(())
}

fn read_contacts(conn: &Connection, id: VendorId) -> CoreResult<Vec<VendorContact>> {
    let mut stmt =
        conn.prepare("SELECT name, role, email, phone FROM vendor_contact WHERE vendor_id = ?1 ORDER BY position")?;
    let contacts = stmt
        .query_map([id], |r| Ok(VendorContact { name: r.get(0)?, role: r.get(1)?, email: r.get(2)?, phone: r.get(3)? }))?
        .collect::<Result<_, _>>()?;
    Ok(contacts)
}
