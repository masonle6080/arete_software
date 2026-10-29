//! Library core: Production Orders on a Drop, one Vendor each, with
//! quantities per size per Style, a delivery date, Notes, and a lifecycle.
//! Placing an order freezes each Style's unit cost from its Cost Sheet at
//! the highest Quantity Tier that does not exceed the ordered quantity
//! (spec #1, ticket #32).
//!
//! These tests call the domain API directly against a fresh temporary
//! library, and check results only through the same API.

mod common;

use arete_lib::blank::{self, BlankInput};
use arete_lib::cost_sheet::{self, CostCategory, CostLineInput, HomeAmount};
use arete_lib::drop::{self, DropId};
use arete_lib::error::CoreError;
use arete_lib::library::Library;
use arete_lib::note::{self, NoteInput, NoteOwner};
use arete_lib::production_order::{self as po, ProductionOrderStatus as Status};
use arete_lib::style::{self, StyleBase, StyleId};
use arete_lib::thesis;
use arete_lib::vendor::{self, VendorId, VendorInput, VendorRole};
use common::TestEnv;

struct Fixture {
    drop_id: DropId,
    vendor_id: VendorId,
    blank_id: i64,
}

fn a_drop(lib: &Library) -> Fixture {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(lib, t.id, "Amor fati SS27").unwrap();
    let vendor_id = a_vendor(lib, "Atlas Knit");
    let blank_id = blank::create_blank(
        lib,
        BlankInput {
            brand: "Atlas".into(),
            model: "Heavy 300".into(),
            vendor_id,
            colors: vec![],
            sizes: vec!["S".into(), "M".into(), "L".into()],
            cost_cents: None,
            currency: String::new(),
            notes: String::new(),
        },
    )
    .unwrap()
    .id;
    Fixture { drop_id: d.id, vendor_id, blank_id }
}

fn a_vendor(lib: &Library, name: &str) -> VendorId {
    vendor::create_vendor(
        lib,
        VendorInput { name: name.into(), roles: vec![VendorRole::Manufacturer], ..Default::default() },
    )
    .unwrap()
    .id
}

/// A Style on the Drop, built on the Blank, in sizes S, M, and L.
fn a_style(lib: &Library, f: &Fixture, name: &str) -> StyleId {
    let id = style::create_style(lib, f.drop_id, name, "Bone").unwrap().id;
    style::set_style_base(lib, id, StyleBase::Blank { blank_id: f.blank_id }).unwrap();
    style::set_style_sizes(lib, id, &["S".into(), "M".into(), "L".into()]).unwrap();
    id
}

#[test]
fn a_drop_holds_several_production_orders_one_vendor_each() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = a_drop(&lib);
    let printer = a_vendor(&lib, "Ink Lab");

    let first = po::create_production_order(&lib, f.drop_id, f.vendor_id).unwrap();
    let second = po::create_production_order(&lib, f.drop_id, printer).unwrap();

    assert_eq!(first.status, Status::Draft);
    assert_eq!(first.vendor_name, "Atlas Knit");
    assert_eq!(first.delivery_date, None);
    let orders = po::drop_production_orders(&lib, f.drop_id).unwrap();
    assert_eq!(orders.iter().map(|o| (o.id, o.vendor_id)).collect::<Vec<_>>(), [(first.id, f.vendor_id), (second.id, printer)]);
}

/// The quantities of an order as (style, size, quantity), in order.
fn quantities(order: &po::ProductionOrder) -> Vec<(StyleId, String, u32)> {
    order
        .styles
        .iter()
        .flat_map(|s| s.quantities.iter().map(move |q| (s.style_id, q.size.clone(), q.quantity)))
        .collect()
}

#[test]
fn a_draft_order_takes_quantities_per_size_per_style_and_a_delivery_date() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = a_drop(&lib);
    let tee = a_style(&lib, &f, "Heavy tee");
    let hoodie = a_style(&lib, &f, "Hoodie");
    let order = po::create_production_order(&lib, f.drop_id, f.vendor_id).unwrap();

    po::set_order_quantity(&lib, order.id, hoodie, "M", 20).unwrap();
    po::set_order_quantity(&lib, order.id, tee, "L", 30).unwrap();
    po::set_order_quantity(&lib, order.id, tee, "S", 10).unwrap();
    po::set_order_quantity(&lib, order.id, tee, "M", 99).unwrap();
    let order = po::set_order_quantity(&lib, order.id, tee, "M", 25).unwrap();
    // Sizes follow the Style's size order; Styles follow the Drop's order.
    assert_eq!(
        quantities(&order),
        [(tee, "S".into(), 10), (tee, "M".into(), 25), (tee, "L".into(), 30), (hoodie, "M".into(), 20)]
    );
    assert_eq!(order.styles[0].name, "Heavy tee");
    assert_eq!(order.styles[0].quantity, 65);
    assert_eq!(order.total_quantity, 85);

    // Zero removes a size; a Style with no sizes left leaves the order.
    let order = po::set_order_quantity(&lib, order.id, hoodie, "M", 0).unwrap();
    assert_eq!(order.styles.len(), 1);

    let order = po::set_delivery_date(&lib, order.id, Some("2027-03-15")).unwrap();
    assert_eq!(order.delivery_date.as_deref(), Some("2027-03-15"));
    assert_eq!(po::get_production_order(&lib, order.id).unwrap(), order);
    // Blank text clears the date, like None.
    let order = po::set_delivery_date(&lib, order.id, Some(" ")).unwrap();
    assert_eq!(order.delivery_date, None);
}

#[test]
fn an_order_refuses_bad_sizes_styles_and_dates() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = a_drop(&lib);
    let tee = a_style(&lib, &f, "Heavy tee");
    let order = po::create_production_order(&lib, f.drop_id, f.vendor_id).unwrap();

    assert!(matches!(po::set_order_quantity(&lib, order.id, tee, "XXL", 5), Err(CoreError::InvalidInput { .. })));

    let t2 = thesis::create_thesis(&lib, "Memento mori", "").unwrap();
    let d2 = drop::create_drop(&lib, t2.id, "Memento mori FW27").unwrap();
    let other = style::create_style(&lib, d2.id, "Coat", "Black").unwrap().id;
    assert!(matches!(po::set_order_quantity(&lib, order.id, other, "S", 5), Err(CoreError::InvalidInput { .. })));

    for bad in ["2027-02-30", "2027-13-01", "15/03/2027", "2027-3-15"] {
        assert!(matches!(po::set_delivery_date(&lib, order.id, Some(bad)), Err(CoreError::InvalidInput { .. })), "{bad}");
    }
    po::set_delivery_date(&lib, order.id, Some("2028-02-29")).unwrap();
}

/// Gives a Style a one-line Cost Sheet in `currency` with these
/// (tier quantity, unit cost) pairs. Returns the line ID and tier IDs.
fn price(lib: &Library, style_id: StyleId, currency: &str, tiers: &[(u32, &str)]) -> (i64, Vec<i64>) {
    for (quantity, _) in tiers {
        cost_sheet::add_quantity_tier(lib, style_id, *quantity).unwrap();
    }
    let sheet = cost_sheet::add_cost_line(
        lib,
        style_id,
        CostLineInput { category: CostCategory::Base, vendor_id: None, description: "Make".into(), currency: currency.into() },
    )
    .unwrap();
    let line = sheet.lines.last().unwrap().id;
    let tier_ids: Vec<i64> = tiers
        .iter()
        .map(|(quantity, _)| sheet.tiers.iter().find(|t| t.quantity == *quantity).unwrap().id)
        .collect();
    for ((_, cost), tier) in tiers.iter().zip(&tier_ids) {
        cost_sheet::set_unit_cost(lib, line, *tier, Some(cost)).unwrap();
    }
    (line, tier_ids)
}

fn text(amount: Option<HomeAmount>) -> Option<String> {
    amount.map(|a| a.to_string())
}

fn s(t: &str) -> Option<String> {
    Some(t.to_string())
}

#[test]
fn placing_freezes_each_styles_unit_cost_at_the_highest_tier_not_above_its_quantity() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = a_drop(&lib);
    cost_sheet::set_home_currency(&lib, "USD").unwrap();
    let tee = a_style(&lib, &f, "Heavy tee");
    let hoodie = a_style(&lib, &f, "Hoodie");
    let (tee_line, tee_tiers) = price(&lib, tee, "USD", &[(50, "12"), (100, "11"), (300, "9.50")]);
    price(&lib, hoodie, "USD", &[(100, "30"), (200, "27.25")]);

    let order = po::create_production_order(&lib, f.drop_id, f.vendor_id).unwrap();
    // 10 + 25 + 30 = 65 tees: the 50 tier, since 100 is above 65.
    po::set_order_quantity(&lib, order.id, tee, "S", 10).unwrap();
    po::set_order_quantity(&lib, order.id, tee, "M", 25).unwrap();
    po::set_order_quantity(&lib, order.id, tee, "L", 30).unwrap();
    // 100 + 100 = 200 hoodies: exactly the 200 tier.
    po::set_order_quantity(&lib, order.id, hoodie, "M", 100).unwrap();
    let draft = po::set_order_quantity(&lib, order.id, hoodie, "L", 100).unwrap();
    assert!(!draft.costs_frozen);

    let placed = po::move_production_order(&lib, order.id, Status::Placed).unwrap();
    assert_eq!(placed.status, Status::Placed);
    assert!(placed.costs_frozen);
    assert_eq!(placed.currency.as_deref(), Some("USD"));
    let (t, h) = (&placed.styles[0], &placed.styles[1]);
    assert_eq!((t.tier_quantity, text(t.unit_cost), text(t.cost)), (Some(50), s("12.00"), s("780.00")));
    assert_eq!((h.tier_quantity, text(h.unit_cost), text(h.cost)), (Some(200), s("27.25"), s("5450.00")));
    assert_eq!(text(placed.total_cost), s("6230.00"));

    // Later Cost Sheet and currency edits do not change the order.
    cost_sheet::set_unit_cost(&lib, tee_line, tee_tiers[0], Some("15")).unwrap();
    cost_sheet::set_home_currency(&lib, "EUR").unwrap();
    let after = po::get_production_order(&lib, order.id).unwrap();
    assert_eq!(after, placed);
    assert_eq!(po::drop_production_orders(&lib, f.drop_id).unwrap(), [placed]);
}

#[test]
fn a_draft_shows_an_estimate_from_the_current_cost_sheets() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = a_drop(&lib);
    cost_sheet::set_home_currency(&lib, "USD").unwrap();
    let tee = a_style(&lib, &f, "Heavy tee");
    let (line, tiers) = price(&lib, tee, "USD", &[(50, "12")]);
    let order = po::create_production_order(&lib, f.drop_id, f.vendor_id).unwrap();
    let draft = po::set_order_quantity(&lib, order.id, tee, "M", 60).unwrap();
    assert_eq!(text(draft.total_cost), s("720.00"));
    cost_sheet::set_unit_cost(&lib, line, tiers[0], Some("10")).unwrap();
    assert_eq!(text(po::get_production_order(&lib, order.id).unwrap().total_cost), s("600.00"));
}

#[test]
fn placing_is_refused_when_a_style_has_no_tier_at_or_below_its_quantity() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = a_drop(&lib);
    cost_sheet::set_home_currency(&lib, "USD").unwrap();
    let tee = a_style(&lib, &f, "Heavy tee");
    let hoodie = a_style(&lib, &f, "Hoodie");
    price(&lib, tee, "USD", &[(50, "12")]);
    price(&lib, hoodie, "USD", &[(100, "30")]);
    let order = po::create_production_order(&lib, f.drop_id, f.vendor_id).unwrap();
    po::set_order_quantity(&lib, order.id, tee, "M", 60).unwrap();
    // 99 hoodies: the lowest tier is 100.
    po::set_order_quantity(&lib, order.id, hoodie, "M", 99).unwrap();

    let err = po::move_production_order(&lib, order.id, Status::Placed).unwrap_err();
    assert!(matches!(&err, CoreError::InvalidInput { reason } if reason.contains("Hoodie")), "{err}");
    let order = po::get_production_order(&lib, order.id).unwrap();
    assert_eq!(order.status, Status::Draft);
    assert!(!order.costs_frozen);

    // A Style with no Cost Sheet tiers at all is refused the same way.
    let cap = a_style(&lib, &f, "Cap");
    po::set_order_quantity(&lib, order.id, hoodie, "M", 100).unwrap();
    po::set_order_quantity(&lib, order.id, cap, "S", 10).unwrap();
    let err = po::move_production_order(&lib, order.id, Status::Placed).unwrap_err();
    assert!(matches!(&err, CoreError::InvalidInput { reason } if reason.contains("Cap")), "{err}");
}

#[test]
fn placing_is_refused_without_quantities_or_with_a_missing_exchange_rate() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = a_drop(&lib);
    cost_sheet::set_home_currency(&lib, "USD").unwrap();
    let tee = a_style(&lib, &f, "Heavy tee");
    price(&lib, tee, "EUR", &[(50, "12")]);
    let order = po::create_production_order(&lib, f.drop_id, f.vendor_id).unwrap();
    assert!(matches!(po::move_production_order(&lib, order.id, Status::Placed), Err(CoreError::InvalidInput { .. })));

    po::set_order_quantity(&lib, order.id, tee, "M", 60).unwrap();
    assert!(matches!(po::move_production_order(&lib, order.id, Status::Placed), Err(CoreError::InvalidInput { .. })));
    assert_eq!(po::get_production_order(&lib, order.id).unwrap().status, Status::Draft);

    cost_sheet::set_exchange_rate(&lib, "EUR", "1.10").unwrap();
    let placed = po::move_production_order(&lib, order.id, Status::Placed).unwrap();
    // 12 EUR × 1.10 = 13.20 USD, × 60 = 792.00.
    assert_eq!(text(placed.total_cost), s("792.00"));
}

#[test]
fn an_order_moves_through_its_lifecycle_or_is_cancelled() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = a_drop(&lib);
    cost_sheet::set_home_currency(&lib, "USD").unwrap();
    let tee = a_style(&lib, &f, "Heavy tee");
    price(&lib, tee, "USD", &[(50, "12")]);
    let order = po::create_production_order(&lib, f.drop_id, f.vendor_id).unwrap();
    assert_eq!(order.moves, [Status::Placed, Status::Cancelled]);
    po::set_order_quantity(&lib, order.id, tee, "M", 60).unwrap();

    assert!(matches!(
        po::move_production_order(&lib, order.id, Status::Shipped),
        Err(CoreError::StatusMoveRefused { .. })
    ));
    for to in [Status::Placed, Status::InProduction, Status::Shipped, Status::Received] {
        assert_eq!(po::move_production_order(&lib, order.id, to).unwrap().status, to);
    }
    let received = po::get_production_order(&lib, order.id).unwrap();
    assert!(received.moves.is_empty());
    assert!(matches!(
        po::move_production_order(&lib, order.id, Status::Cancelled),
        Err(CoreError::StatusMoveRefused { .. })
    ));
    // Quantities are fixed once the order is placed.
    assert!(matches!(po::set_order_quantity(&lib, order.id, tee, "M", 70), Err(CoreError::InvalidInput { .. })));

    let draft = po::create_production_order(&lib, f.drop_id, f.vendor_id).unwrap();
    let cancelled = po::move_production_order(&lib, draft.id, Status::Cancelled).unwrap();
    assert_eq!(cancelled.status, Status::Cancelled);
    assert!(cancelled.moves.is_empty());

    // A placed order keeps its frozen costs when cancelled.
    let placed = po::create_production_order(&lib, f.drop_id, f.vendor_id).unwrap();
    po::set_order_quantity(&lib, placed.id, tee, "S", 50).unwrap();
    po::move_production_order(&lib, placed.id, Status::Placed).unwrap();
    let cancelled = po::move_production_order(&lib, placed.id, Status::Cancelled).unwrap();
    assert_eq!(text(cancelled.total_cost), s("600.00"));
}

#[test]
fn a_production_order_takes_notes() {
    let env = TestEnv::new();
    let lib = env.library();
    let f = a_drop(&lib);
    let order = po::create_production_order(&lib, f.drop_id, f.vendor_id).unwrap();
    let owner = NoteOwner::ProductionOrder(order.id);

    note::add_note(&lib, owner, NoteInput { body: "Deposit paid".into(), attachments: vec![] }).unwrap();
    let notes = note::list_notes(&lib, owner).unwrap();
    assert_eq!(notes.iter().map(|n| n.body.as_str()).collect::<Vec<_>>(), ["Deposit paid"]);

    assert!(matches!(
        note::add_note(&lib, NoteOwner::ProductionOrder(order.id + 99), NoteInput { body: "x".into(), attachments: vec![] }),
        Err(CoreError::NotFound { .. })
    ));
}
