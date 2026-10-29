//! Production Order: a committed order to one Vendor for a Drop's Styles
//! (GLOSSARY: Production Order). A Drop has several, one Vendor each.
//!
//! An order holds quantities per size per Style, the Vendor's delivery date,
//! and dated notes in the shared Notes record ([`crate::note`], owner
//! `NoteOwner::ProductionOrder`). It moves Draft → Placed → In Production →
//! Shipped → Received, or to Cancelled ([`ProductionOrderStatus`]).
//!
//! Costs: on Placed, the core copies each Style's unit cost from its Cost
//! Sheet ([`cost_sheet::unit_cost_for_quantity`]) and freezes it, in integer
//! cents of the home currency of that moment. A Draft shows an estimate from
//! the current Cost Sheets instead.
//!
//! Domain command functions for this area: [`create_production_order`],
//! [`get_production_order`], [`drop_production_orders`],
//! [`set_order_quantity`], [`set_delivery_date`], [`move_production_order`].

mod status;

pub use status::ProductionOrderStatus;

use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::cost_sheet::{self, HomeAmount, TierUnitCost};
use crate::drop::{self, DropId, DropStage};
use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::style::{self, StyleId, StyleStatus};
use crate::vendor::{self, VendorId};

pub type ProductionOrderId = i64;

/// A Production Order as the founder sees it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionOrder {
    pub id: ProductionOrderId,
    pub drop_id: DropId,
    pub vendor_id: VendorId,
    /// The Vendor's name, for display.
    pub vendor_name: String,
    pub status: ProductionOrderStatus,
    /// The statuses the lifecycle lets this order move to now, so the UI
    /// never repeats the rules. A move can still be refused by a guard.
    pub moves: Vec<ProductionOrderStatus>,
    /// The Vendor's delivery date, `YYYY-MM-DD`.
    pub delivery_date: Option<String>,
    /// The Styles with a quantity on this order, in the Drop's Style order.
    pub styles: Vec<OrderStyle>,
    /// The pieces on the whole order.
    pub total_quantity: u32,
    /// True once the order was Placed: the unit costs are frozen and no
    /// longer follow the Cost Sheets. While false, the costs are an
    /// estimate from the current Cost Sheets.
    pub costs_frozen: bool,
    /// The currency of every cost on the order: the home currency frozen at
    /// placing, or the current home currency for an estimate.
    pub currency: Option<String>,
    /// The sum of the Style costs. `None` when the order has no Styles or a
    /// Style has no cost (no applicable tier, a missing rate, or an order
    /// Cancelled before it was Placed).
    pub total_cost: Option<HomeAmount>,
}

/// One Style on a Production Order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderStyle {
    pub style_id: StyleId,
    pub name: String,
    pub color: String,
    /// The sizes with a quantity, in the Style's size order.
    pub quantities: Vec<SizeQuantity>,
    /// The pieces of this Style, summed across its sizes. Tier selection
    /// uses this number.
    pub quantity: u32,
    /// The Quantity Tier the unit cost comes from.
    pub tier_quantity: Option<u32>,
    /// The unit cost, in [`ProductionOrder::currency`].
    pub unit_cost: Option<HomeAmount>,
    /// `unit_cost × quantity`.
    pub cost: Option<HomeAmount>,
}

/// The ordered quantity of one size.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeQuantity {
    pub size: String,
    pub quantity: u32,
}

const SELECT: &str = "SELECT o.id, o.drop_id, o.vendor_id, v.name, o.status, o.delivery_date, o.currency
     FROM production_order o JOIN vendor v ON v.id = o.vendor_id";

fn from_row(row: &Row) -> rusqlite::Result<ProductionOrder> {
    let raw: String = row.get(4)?;
    let status = ProductionOrderStatus::from_db(&raw).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, format!("unknown status {raw}").into())
    })?;
    Ok(ProductionOrder {
        id: row.get(0)?,
        drop_id: row.get(1)?,
        vendor_id: row.get(2)?,
        vendor_name: row.get(3)?,
        status,
        moves: status.moves(),
        delivery_date: row.get(5)?,
        styles: Vec::new(),
        total_quantity: 0,
        currency: row.get(6)?,
        costs_frozen: false,
        total_cost: None,
    })
}

/// Creates a Draft Production Order on a Drop for one Vendor. Refuses a
/// Cancelled Drop.
pub fn create_production_order(lib: &Library, drop_id: DropId, vendor_id: VendorId) -> CoreResult<ProductionOrder> {
    let d = drop::get_drop(lib, drop_id)?;
    if d.stage == DropStage::Cancelled {
        return Err(CoreError::InvalidInput { reason: "a Cancelled Drop takes no Production Orders".into() });
    }
    vendor::get_vendor(lib, vendor_id)?;
    let conn = lib.writer()?;
    conn.execute(
        "INSERT INTO production_order (drop_id, vendor_id, status) VALUES (?1, ?2, ?3)",
        params![drop_id, vendor_id, ProductionOrderStatus::Draft.as_db()],
    )?;
    get_production_order(lib, conn.last_insert_rowid())
}

/// Reads one Production Order.
pub fn get_production_order(lib: &Library, id: ProductionOrderId) -> CoreResult<ProductionOrder> {
    let order = lib
        .db()
        .query_row(&format!("{SELECT} WHERE o.id = ?1"), [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { record: "Production Order", id })?;
    complete(lib, order)
}

/// Every Production Order of a Drop, oldest first: the Drop's production
/// overview.
pub fn drop_production_orders(lib: &Library, drop_id: DropId) -> CoreResult<Vec<ProductionOrder>> {
    drop::get_drop(lib, drop_id)?;
    let mut stmt = lib.db().prepare(&format!("{SELECT} WHERE o.drop_id = ?1 ORDER BY o.id"))?;
    let orders: Vec<ProductionOrder> = stmt.query_map([drop_id], from_row)?.collect::<Result<_, _>>()?;
    orders.into_iter().map(|o| complete(lib, o)).collect()
}

/// Sets the quantity of one size of one Style on a Draft order. Zero
/// removes the size. The Style must be on the order's Drop and Approved (zero
/// removes a size of any Style), and the size one of the Style's sizes.
pub fn set_order_quantity(
    lib: &Library,
    id: ProductionOrderId,
    style_id: StyleId,
    size: &str,
    quantity: u32,
) -> CoreResult<ProductionOrder> {
    let order = get_production_order(lib, id)?;
    if order.status != ProductionOrderStatus::Draft {
        return Err(CoreError::InvalidInput {
            reason: format!("quantities are fixed once an order is {}", order.status.label()),
        });
    }
    let style = style::get_style(lib, style_id)?;
    if style.drop_id != order.drop_id {
        return Err(CoreError::InvalidInput { reason: "the Style is on another Drop".into() });
    }
    if quantity > 0 && style.status != StyleStatus::Approved {
        return Err(CoreError::InvalidInput {
            reason: format!("{} ({}) is {}; order only Approved Styles", style.name, style.color, style.status.label()),
        });
    }
    let size = size.trim();
    if quantity > 0 && !style::style_sizes(lib, style_id)?.iter().any(|s| s == size) {
        return Err(CoreError::InvalidInput {
            reason: format!("{} ({}) does not come in size {size}", style.name, style.color),
        });
    }
    let conn = lib.writer()?;
    if quantity == 0 {
        conn.execute(
            "DELETE FROM production_order_quantity WHERE order_id = ?1 AND style_id = ?2 AND size = ?3",
            params![id, style_id, size],
        )?;
    } else {
        conn.execute(
            "INSERT INTO production_order_quantity (order_id, style_id, size, quantity) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (order_id, style_id, size) DO UPDATE SET quantity = excluded.quantity",
            params![id, style_id, size, quantity],
        )?;
    }
    get_production_order(lib, id)
}

/// Sets the Vendor's delivery date, `YYYY-MM-DD`. `None` or blank text
/// clears it. The date can change in any status, since Vendors move dates.
pub fn set_delivery_date(lib: &Library, id: ProductionOrderId, date: Option<&str>) -> CoreResult<ProductionOrder> {
    get_production_order(lib, id)?;
    let date = date.map(str::trim).filter(|d| !d.is_empty());
    if let Some(d) = date {
        check_date(d)?;
    }
    lib.writer()?.execute("UPDATE production_order SET delivery_date = ?1 WHERE id = ?2", params![date, id])?;
    get_production_order(lib, id)
}

/// True when a Production Order with this ID exists.
pub(crate) fn production_order_exists(conn: &Connection, id: ProductionOrderId) -> CoreResult<bool> {
    Ok(conn.query_row("SELECT 1 FROM production_order WHERE id = ?1", [id], |_| Ok(())).optional()?.is_some())
}

/// Refuses anything that is not a real calendar date as `YYYY-MM-DD`.
fn check_date(date: &str) -> CoreResult<()> {
    let refuse = || CoreError::InvalidInput { reason: format!("{date} is not a date like 2027-03-15") };
    let parts: Vec<&str> = date.split('-').collect();
    let [y, m, d] = parts.as_slice() else { return Err(refuse()) };
    let digits = |s: &str, len: usize| s.len() == len && s.chars().all(|c| c.is_ascii_digit());
    if !digits(y, 4) || !digits(m, 2) || !digits(d, 2) {
        return Err(refuse());
    }
    let (y, m, d): (u32, u32, u32) = (y.parse().unwrap_or(0), m.parse().unwrap_or(0), d.parse().unwrap_or(0));
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return Err(refuse()),
    };
    if d == 0 || d > days {
        return Err(refuse());
    }
    Ok(())
}

/// Fills in the Styles and quantities of an order.
fn complete(lib: &Library, mut order: ProductionOrder) -> CoreResult<ProductionOrder> {
    let mut stmt = lib.db().prepare(
        "SELECT s.id, s.name, s.color, q.size, q.quantity FROM production_order_quantity q
         JOIN style s ON s.id = q.style_id WHERE q.order_id = ?1 ORDER BY s.id",
    )?;
    let rows: Vec<(StyleId, String, String, String, u32)> = stmt
        .query_map([order.id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))?
        .collect::<Result<_, _>>()?;
    let mut styles: Vec<OrderStyle> = Vec::new();
    for (style_id, name, color, size, quantity) in rows {
        if styles.last().map(|s| s.style_id) != Some(style_id) {
            styles.push(OrderStyle {
                style_id,
                name,
                color,
                quantities: Vec::new(),
                quantity: 0,
                tier_quantity: None,
                unit_cost: None,
                cost: None,
            });
        }
        let style = styles.last_mut().expect("a Style was just pushed");
        style.quantity += quantity;
        style.quantities.push(SizeQuantity { size, quantity });
    }
    for style in &mut styles {
        let size_order = style::style_sizes(lib, style.style_id)?;
        let rank = |size: &str| size_order.iter().position(|s| s == size).unwrap_or(usize::MAX);
        style.quantities.sort_by(|a, b| rank(&a.size).cmp(&rank(&b.size)).then_with(|| a.size.cmp(&b.size)));
    }
    order.costs_frozen = order.currency.is_some();
    if order.costs_frozen {
        let mut stmt = lib
            .db()
            .prepare("SELECT style_id, tier_quantity, unit_cost_cents FROM production_order_cost WHERE order_id = ?1")?;
        let frozen: HashMap<StyleId, (u32, i64)> = stmt
            .query_map([order.id], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))?
            .collect::<Result<_, _>>()?;
        for style in &mut styles {
            if let Some((tier, cents)) = frozen.get(&style.style_id) {
                style.tier_quantity = Some(*tier);
                style.unit_cost = Some(HomeAmount::from_cents(*cents));
            }
        }
    } else if order.status == ProductionOrderStatus::Draft {
        order.currency = cost_sheet::currency_settings(lib)?.home_currency;
        for style in &mut styles {
            // An estimate: a Style whose cost cannot be computed yet has none.
            if let Ok(Some(tier)) = cost_sheet::unit_cost_for_quantity(lib, style.style_id, style.quantity) {
                style.tier_quantity = Some(tier.tier_quantity);
                style.unit_cost = Some(tier.unit_cost);
            }
        }
    }
    for style in &mut styles {
        style.cost = style.unit_cost.map(|u| HomeAmount::from_cents(u.cents() * i64::from(style.quantity)));
    }
    order.total_cost = if styles.is_empty() {
        None
    } else {
        styles.iter().map(|s| s.cost.map(HomeAmount::cents)).sum::<Option<i64>>().map(HomeAmount::from_cents)
    };
    order.total_quantity = styles.iter().map(|s| s.quantity).sum();
    order.styles = styles;
    Ok(order)
}

/// Moves an order to `to`. The lifecycle
/// ([`ProductionOrderStatus::move_allowed`]) refuses with an error. Moving
/// to Placed copies each Style's unit cost from its Cost Sheet, at the
/// highest Quantity Tier that does not exceed the Style's quantity on the
/// order (summed across sizes), and freezes it with the home currency.
/// Placing is refused when the order has no quantities, when a Style on it is
/// no longer Approved, when a Style has no such tier, or when a cost cannot
/// be computed (home currency or exchange rate missing).
pub fn move_production_order(
    lib: &Library,
    id: ProductionOrderId,
    to: ProductionOrderStatus,
) -> CoreResult<ProductionOrder> {
    let order = get_production_order(lib, id)?;
    if !order.status.move_allowed(to) {
        return Err(CoreError::StatusMoveRefused { what: "Production Order", from: order.status.label(), to: to.label() });
    }
    let frozen = if to == ProductionOrderStatus::Placed { Some(costs_to_freeze(lib, &order)?) } else { None };
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    tx.execute("UPDATE production_order SET status = ?1 WHERE id = ?2", params![to.as_db(), id])?;
    if let Some((currency, costs)) = frozen {
        tx.execute("UPDATE production_order SET currency = ?1 WHERE id = ?2", params![currency, id])?;
        for (style_id, tier) in costs {
            tx.execute(
                "INSERT INTO production_order_cost (order_id, style_id, tier_quantity, unit_cost_cents)
                 VALUES (?1, ?2, ?3, ?4)",
                params![id, style_id, tier.tier_quantity, tier.unit_cost.cents()],
            )?;
        }
    }
    tx.commit()?;
    get_production_order(lib, id)
}

/// The home currency and the unit cost of each Style on the order, as
/// placing freezes them. Refuses as [`move_production_order`] describes.
fn costs_to_freeze(lib: &Library, order: &ProductionOrder) -> CoreResult<(String, Vec<(StyleId, TierUnitCost)>)> {
    if order.styles.is_empty() {
        return Err(CoreError::InvalidInput { reason: "enter quantities before you place the order".into() });
    }
    let mut not_approved = Vec::new();
    for s in &order.styles {
        let current = style::get_style(lib, s.style_id)?;
        if current.status != StyleStatus::Approved {
            not_approved.push(format!("{} ({}) is {}", s.name, s.color, current.status.label()));
        }
    }
    if !not_approved.is_empty() {
        return Err(CoreError::InvalidInput {
            reason: format!("order only Approved Styles; remove: {}", not_approved.join(", ")),
        });
    }
    let mut costs = Vec::new();
    let mut no_tier = Vec::new();
    for style in &order.styles {
        match cost_sheet::unit_cost_for_quantity(lib, style.style_id, style.quantity)? {
            Some(tier) => costs.push((style.style_id, tier)),
            None => no_tier.push(format!("{} ({}) at {} pieces", style.name, style.color, style.quantity)),
        }
    }
    if !no_tier.is_empty() {
        return Err(CoreError::InvalidInput {
            reason: format!(
                "no Quantity Tier on the Cost Sheet is at or below the ordered quantity for: {}",
                no_tier.join(", ")
            ),
        });
    }
    let currency = cost_sheet::currency_settings(lib)?
        .home_currency
        .ok_or_else(|| CoreError::InvalidInput { reason: "set the home currency in the library settings".into() })?;
    Ok((currency, costs))
}
