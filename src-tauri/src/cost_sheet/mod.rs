//! Cost Sheet: the itemized unit cost of a Style at each Quantity Tier
//! (GLOSSARY: Cost Sheet, Quantity Tier).
//!
//! Each Style has one Cost Sheet. It has Quantity Tiers (columns, in
//! quantity order) and lines (category, Vendor, description, quote
//! currency, and a unit cost per tier). Each line keeps its quote currency.
//! The sheet totals each tier in the home currency with the rates in
//! [`currency`]. Money is exact; [`money`] holds the types and the rounding
//! rule.
//!
//! A blank cell (no unit cost at a tier) counts as zero in the total. A tier
//! has no total while the home currency is not set or a line's currency has
//! no exchange rate; [`CostSheet::missing_rates`] names those currencies.
//!
//! Domain command functions: [`cost_sheet`], [`add_quantity_tier`],
//! [`set_quantity_tier`], [`remove_quantity_tier`], [`add_cost_line`],
//! [`update_cost_line`], [`remove_cost_line`], [`set_unit_cost`], and the
//! currency settings in [`currency`]. Production Orders (ticket #32) read a
//! unit cost with [`unit_cost_at_tier`] or [`unit_cost_for_quantity`].

pub mod currency;
pub mod money;

use std::collections::{BTreeSet, HashMap};

use rusqlite::{params, Connection, OptionalExtension};

pub use currency::{currency_settings, remove_exchange_rate, set_exchange_rate, set_home_currency, CurrencyRate, CurrencySettings};
pub use money::{ExchangeRate, HomeAmount, UnitCost};

use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::style::{self, StyleId};
use crate::vendor::{self, VendorId};

pub type QuantityTierId = i64;
pub type CostLineId = i64;

/// What a Cost Sheet line pays for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CostCategory {
    Base,
    Decoration,
    Trims,
    Labels,
    Packaging,
    Shipping,
    Duties,
    Other,
}

impl CostCategory {
    pub const ALL: [CostCategory; 8] = [
        CostCategory::Base,
        CostCategory::Decoration,
        CostCategory::Trims,
        CostCategory::Labels,
        CostCategory::Packaging,
        CostCategory::Shipping,
        CostCategory::Duties,
        CostCategory::Other,
    ];

    fn as_db(self) -> &'static str {
        match self {
            CostCategory::Base => "base",
            CostCategory::Decoration => "decoration",
            CostCategory::Trims => "trims",
            CostCategory::Labels => "labels",
            CostCategory::Packaging => "packaging",
            CostCategory::Shipping => "shipping",
            CostCategory::Duties => "duties",
            CostCategory::Other => "other",
        }
    }

    fn from_db(s: &str) -> CostCategory {
        CostCategory::ALL.into_iter().find(|c| c.as_db() == s).unwrap_or(CostCategory::Other)
    }
}

/// One Quantity Tier: an order quantity, in pieces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantityTier {
    pub id: QuantityTierId,
    pub quantity: u32,
}

/// One line of a Cost Sheet.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CostLine {
    pub id: CostLineId,
    pub category: CostCategory,
    pub vendor_id: Option<VendorId>,
    /// The Vendor's name, for display.
    pub vendor_name: Option<String>,
    pub description: String,
    /// The quote currency, an ISO 4217 code such as "EUR".
    pub currency: String,
    /// The unit cost at each tier, in the order of [`CostSheet::tiers`], in
    /// `currency`. `None` is a blank cell.
    pub unit_costs: Vec<Option<UnitCost>>,
}

/// The total unit cost of one tier, in the home currency.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TierTotal {
    pub tier_id: QuantityTierId,
    pub quantity: u32,
    /// `None` while the home currency is not set or a rate is missing.
    pub total: Option<HomeAmount>,
}

/// The Cost Sheet of one Style.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CostSheet {
    pub style_id: StyleId,
    /// `None` until the founder sets it in the library settings.
    pub home_currency: Option<String>,
    /// In quantity order.
    pub tiers: Vec<QuantityTier>,
    /// In the order the founder added them.
    pub lines: Vec<CostLine>,
    /// One per tier, in the order of `tiers`.
    pub totals: Vec<TierTotal>,
    /// The line currencies with no exchange rate, in code order.
    pub missing_rates: Vec<String>,
}

/// The fields of a Cost Sheet line. An edit replaces every field; unit costs
/// are set one cell at a time with [`set_unit_cost`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostLineInput {
    pub category: CostCategory,
    #[serde(default)]
    pub vendor_id: Option<VendorId>,
    #[serde(default)]
    pub description: String,
    pub currency: String,
}

/// The unit cost of a Style at one tier, in the home currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TierUnitCost {
    pub tier_quantity: u32,
    pub unit_cost: HomeAmount,
}

/// Reads the Cost Sheet of a Style. A Style with no tiers or lines has an
/// empty sheet.
pub fn cost_sheet(lib: &Library, style_id: StyleId) -> CoreResult<CostSheet> {
    style::get_style(lib, style_id)?;
    read_sheet(lib.db(), style_id)
}

/// Adds a Quantity Tier of `quantity` pieces. Each tier of a sheet has a
/// different quantity.
pub fn add_quantity_tier(lib: &Library, style_id: StyleId, quantity: u32) -> CoreResult<CostSheet> {
    style::get_style(lib, style_id)?;
    check_tier_quantity(lib.db(), style_id, quantity, None)?;
    lib.writer()?
        .execute("INSERT INTO cost_tier (style_id, quantity) VALUES (?1, ?2)", params![style_id, quantity])?;
    read_sheet(lib.db(), style_id)
}

/// Changes the quantity of a tier. Its unit costs stay.
pub fn set_quantity_tier(lib: &Library, tier_id: QuantityTierId, quantity: u32) -> CoreResult<CostSheet> {
    let style_id = tier_style(lib.db(), tier_id)?;
    check_tier_quantity(lib.db(), style_id, quantity, Some(tier_id))?;
    lib.writer()?.execute("UPDATE cost_tier SET quantity = ?1 WHERE id = ?2", params![quantity, tier_id])?;
    read_sheet(lib.db(), style_id)
}

/// Removes a tier and every unit cost at it.
pub fn remove_quantity_tier(lib: &Library, tier_id: QuantityTierId) -> CoreResult<CostSheet> {
    let style_id = tier_style(lib.db(), tier_id)?;
    lib.writer()?.execute("DELETE FROM cost_tier WHERE id = ?1", [tier_id])?;
    read_sheet(lib.db(), style_id)
}

/// Adds a line at the end of the sheet, with blank unit costs.
pub fn add_cost_line(lib: &Library, style_id: StyleId, input: CostLineInput) -> CoreResult<CostSheet> {
    style::get_style(lib, style_id)?;
    let input = validate_line(lib, input)?;
    lib.writer()?.execute(
        "INSERT INTO cost_line (style_id, category, vendor_id, description, currency) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![style_id, input.category.as_db(), input.vendor_id, input.description, input.currency],
    )?;
    read_sheet(lib.db(), style_id)
}

/// Replaces the category, Vendor, description, and currency of a line. Its
/// unit costs stay as entered, now read in the new currency.
pub fn update_cost_line(lib: &Library, line_id: CostLineId, input: CostLineInput) -> CoreResult<CostSheet> {
    let style_id = line_style(lib.db(), line_id)?;
    let input = validate_line(lib, input)?;
    lib.writer()?.execute(
        "UPDATE cost_line SET category = ?1, vendor_id = ?2, description = ?3, currency = ?4 WHERE id = ?5",
        params![input.category.as_db(), input.vendor_id, input.description, input.currency, line_id],
    )?;
    read_sheet(lib.db(), style_id)
}

/// Removes a line and its unit costs.
pub fn remove_cost_line(lib: &Library, line_id: CostLineId) -> CoreResult<CostSheet> {
    let style_id = line_style(lib.db(), line_id)?;
    lib.writer()?.execute("DELETE FROM cost_line WHERE id = ?1", [line_id])?;
    read_sheet(lib.db(), style_id)
}

/// Sets the unit cost of a line at a tier, in the line's currency, for
/// example "12.50". `None` or blank text clears the cell.
pub fn set_unit_cost(
    lib: &Library,
    line_id: CostLineId,
    tier_id: QuantityTierId,
    unit_cost: Option<&str>,
) -> CoreResult<CostSheet> {
    let style_id = line_style(lib.db(), line_id)?;
    if tier_style(lib.db(), tier_id)? != style_id {
        return Err(CoreError::InvalidInput { reason: "the Quantity Tier is on another Style's Cost Sheet".into() });
    }
    let unit_cost = unit_cost.map(str::trim).filter(|t| !t.is_empty()).map(UnitCost::parse).transpose()?;
    let conn = lib.writer()?;
    match unit_cost {
        Some(cost) => conn.execute(
            "INSERT INTO cost_line_price (line_id, tier_id, unit_cost) VALUES (?1, ?2, ?3)
             ON CONFLICT (line_id, tier_id) DO UPDATE SET unit_cost = excluded.unit_cost",
            params![line_id, tier_id, cost.ten_thousandths()],
        )?,
        None => conn.execute("DELETE FROM cost_line_price WHERE line_id = ?1 AND tier_id = ?2", params![line_id, tier_id])?,
    };
    read_sheet(lib.db(), style_id)
}

/// The total unit cost of a Style, in the home currency, at the tier of
/// exactly `quantity` pieces. Refuses when the sheet has no such tier, when
/// the home currency is not set, or when a line's currency has no rate.
pub fn unit_cost_at_tier(lib: &Library, style_id: StyleId, quantity: u32) -> CoreResult<HomeAmount> {
    let sheet = cost_sheet(lib, style_id)?;
    let tier = sheet.totals.iter().find(|t| t.quantity == quantity).ok_or_else(|| CoreError::InvalidInput {
        reason: format!("the Cost Sheet has no Quantity Tier of {quantity}"),
    })?;
    tier_total(&sheet, tier)
}

/// The total unit cost of a Style, in the home currency, at the highest
/// tier that does not exceed `quantity`. `None` when every tier is above
/// `quantity` or the sheet has no tiers. Refuses like [`unit_cost_at_tier`]
/// when the total cannot be computed.
pub fn unit_cost_for_quantity(lib: &Library, style_id: StyleId, quantity: u32) -> CoreResult<Option<TierUnitCost>> {
    let sheet = cost_sheet(lib, style_id)?;
    let Some(tier) = sheet.totals.iter().filter(|t| t.quantity <= quantity).max_by_key(|t| t.quantity) else {
        return Ok(None);
    };
    Ok(Some(TierUnitCost { tier_quantity: tier.quantity, unit_cost: tier_total(&sheet, tier)? }))
}

fn tier_total(sheet: &CostSheet, tier: &TierTotal) -> CoreResult<HomeAmount> {
    if sheet.home_currency.is_none() {
        return Err(CoreError::InvalidInput { reason: "set the home currency in the library settings".into() });
    }
    tier.total.ok_or_else(|| CoreError::InvalidInput {
        reason: format!("set an exchange rate for {} in the library settings", sheet.missing_rates.join(", ")),
    })
}

/// Copies the Cost Sheet of Style `from` onto the new Style `to`: tiers,
/// lines, and unit costs, with new IDs. Only `style::duplicate_style` calls
/// this, inside its transaction.
pub(crate) fn copy_cost_sheet(conn: &Connection, from: StyleId, to: StyleId) -> CoreResult<()> {
    let mut tiers: HashMap<QuantityTierId, QuantityTierId> = HashMap::new();
    for old in ids(conn, "SELECT id FROM cost_tier WHERE style_id = ?1 ORDER BY quantity", from)? {
        conn.execute(
            "INSERT INTO cost_tier (style_id, quantity) SELECT ?1, quantity FROM cost_tier WHERE id = ?2",
            params![to, old],
        )?;
        tiers.insert(old, conn.last_insert_rowid());
    }
    for old in ids(conn, "SELECT id FROM cost_line WHERE style_id = ?1 ORDER BY id", from)? {
        conn.execute(
            "INSERT INTO cost_line (style_id, category, vendor_id, description, currency)
             SELECT ?1, category, vendor_id, description, currency FROM cost_line WHERE id = ?2",
            params![to, old],
        )?;
        let new_line = conn.last_insert_rowid();
        let mut stmt = conn.prepare("SELECT tier_id, unit_cost FROM cost_line_price WHERE line_id = ?1")?;
        let prices: Vec<(QuantityTierId, i64)> =
            stmt.query_map([old], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
        for (old_tier, cost) in prices {
            conn.execute(
                "INSERT INTO cost_line_price (line_id, tier_id, unit_cost) VALUES (?1, ?2, ?3)",
                params![new_line, tiers[&old_tier], cost],
            )?;
        }
    }
    Ok(())
}

fn ids(conn: &Connection, sql: &str, style_id: StyleId) -> CoreResult<Vec<i64>> {
    let mut stmt = conn.prepare(sql)?;
    let ids = stmt.query_map([style_id], |r| r.get(0))?.collect::<Result<_, _>>()?;
    Ok(ids)
}

fn read_sheet(conn: &Connection, style_id: StyleId) -> CoreResult<CostSheet> {
    let mut stmt = conn.prepare("SELECT id, quantity FROM cost_tier WHERE style_id = ?1 ORDER BY quantity")?;
    let tiers: Vec<QuantityTier> = stmt
        .query_map([style_id], |r| Ok(QuantityTier { id: r.get(0)?, quantity: r.get(1)? }))?
        .collect::<Result<_, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT l.id, l.category, l.vendor_id, v.name, l.description, l.currency
         FROM cost_line l LEFT JOIN vendor v ON v.id = l.vendor_id
         WHERE l.style_id = ?1 ORDER BY l.id",
    )?;
    let mut lines: Vec<CostLine> = stmt
        .query_map([style_id], |r| {
            Ok(CostLine {
                id: r.get(0)?,
                category: CostCategory::from_db(&r.get::<_, String>(1)?),
                vendor_id: r.get(2)?,
                vendor_name: r.get(3)?,
                description: r.get(4)?,
                currency: r.get(5)?,
                unit_costs: Vec::new(),
            })
        })?
        .collect::<Result<_, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT p.line_id, p.tier_id, p.unit_cost FROM cost_line_price p
         JOIN cost_line l ON l.id = p.line_id WHERE l.style_id = ?1",
    )?;
    let prices: HashMap<(CostLineId, QuantityTierId), UnitCost> = stmt
        .query_map([style_id], |r| Ok(((r.get(0)?, r.get(1)?), UnitCost::from_ten_thousandths(r.get(2)?))))?
        .collect::<Result<_, _>>()?;
    for line in &mut lines {
        line.unit_costs = tiers.iter().map(|t| prices.get(&(line.id, t.id)).copied()).collect();
    }

    let home_currency = currency::home_currency(conn)?;
    let rates = match &home_currency {
        Some(home) => currency::rates_to_home(conn, home)?,
        None => HashMap::new(),
    };
    let missing_rates: Vec<String> = match home_currency {
        Some(_) => lines
            .iter()
            .filter(|l| !rates.contains_key(&l.currency))
            .map(|l| l.currency.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        None => Vec::new(),
    };
    let can_total = home_currency.is_some() && missing_rates.is_empty();
    let totals = tiers
        .iter()
        .enumerate()
        .map(|(i, tier)| TierTotal {
            tier_id: tier.id,
            quantity: tier.quantity,
            total: can_total.then(|| {
                HomeAmount::total(lines.iter().filter_map(|l| l.unit_costs[i].map(|c| (c, rates[&l.currency]))))
            }),
        })
        .collect();

    Ok(CostSheet { style_id, home_currency, tiers, lines, totals, missing_rates })
}

fn check_tier_quantity(
    conn: &Connection,
    style_id: StyleId,
    quantity: u32,
    except: Option<QuantityTierId>,
) -> CoreResult<()> {
    if quantity == 0 {
        return Err(CoreError::InvalidInput { reason: "a Quantity Tier needs a quantity of at least 1".into() });
    }
    let taken: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM cost_tier WHERE style_id = ?1 AND quantity = ?2 AND id IS NOT ?3)",
        params![style_id, quantity, except],
        |r| r.get(0),
    )?;
    if taken {
        return Err(CoreError::InvalidInput { reason: format!("the Cost Sheet already has a Quantity Tier of {quantity}") });
    }
    Ok(())
}

fn tier_style(conn: &Connection, tier_id: QuantityTierId) -> CoreResult<StyleId> {
    conn.query_row("SELECT style_id FROM cost_tier WHERE id = ?1", [tier_id], |r| r.get(0))
        .optional()?
        .ok_or(CoreError::NotFound { record: "Quantity Tier", id: tier_id })
}

fn line_style(conn: &Connection, line_id: CostLineId) -> CoreResult<StyleId> {
    conn.query_row("SELECT style_id FROM cost_line WHERE id = ?1", [line_id], |r| r.get(0))
        .optional()?
        .ok_or(CoreError::NotFound { record: "Cost Sheet line", id: line_id })
}

/// Trims the description, normalizes the currency, and checks the Vendor.
fn validate_line(lib: &Library, input: CostLineInput) -> CoreResult<CostLineInput> {
    if let Some(vendor_id) = input.vendor_id {
        vendor::get_vendor(lib, vendor_id)?;
    }
    Ok(CostLineInput {
        category: input.category,
        vendor_id: input.vendor_id,
        description: input.description.trim().to_string(),
        currency: currency::currency_code(&input.currency)?,
    })
}
