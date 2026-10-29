//! Library core: the Cost Sheet of a Style, with Quantity Tiers, lines in
//! their quote currencies, and totals per tier in the home currency (spec #1,
//! ticket #31).
//!
//! These tests call the domain API directly against a fresh temporary
//! library, and check results only through the same API.

mod common;

use arete_lib::cost_sheet::{self, CostCategory, CostLineInput};
use arete_lib::drop;
use arete_lib::library::Library;
use arete_lib::style::{self, StyleId};
use arete_lib::thesis;
use arete_lib::vendor::{self, VendorInput, VendorRole};
use common::TestEnv;

fn a_style(lib: &Library) -> StyleId {
    let t = thesis::create_thesis(lib, "Amor fati", "").unwrap();
    let d = drop::create_drop(lib, t.id, "Amor fati SS27").unwrap();
    style::create_style(lib, d.id, "Heavy tee", "Bone").unwrap().id
}

fn line(category: CostCategory, description: &str, currency: &str) -> CostLineInput {
    CostLineInput { category, vendor_id: None, description: description.into(), currency: currency.into() }
}

/// The tier totals of a sheet as text, in tier order.
fn totals(sheet: &cost_sheet::CostSheet) -> Vec<Option<String>> {
    sheet.totals.iter().map(|t| t.total.map(|a| a.to_string())).collect()
}

/// The unit costs of one line as text, in tier order.
fn costs(sheet: &cost_sheet::CostSheet, index: usize) -> Vec<Option<String>> {
    sheet.lines[index].unit_costs.iter().map(|c| c.map(|c| c.to_string())).collect()
}

fn s(text: &str) -> Option<String> {
    Some(text.to_string())
}

#[test]
fn a_cost_sheet_totals_each_tier_in_the_home_currency() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    cost_sheet::set_home_currency(&lib, "usd").unwrap();

    cost_sheet::add_quantity_tier(&lib, style_id, 100).unwrap();
    let sheet = cost_sheet::add_quantity_tier(&lib, style_id, 50).unwrap();
    assert_eq!(sheet.tiers.iter().map(|t| t.quantity).collect::<Vec<_>>(), [50, 100]);
    let (t50, t100) = (sheet.tiers[0].id, sheet.tiers[1].id);

    let sheet = cost_sheet::add_cost_line(&lib, style_id, line(CostCategory::Base, "Blank tee", "USD")).unwrap();
    let blank = sheet.lines[0].id;
    let sheet = cost_sheet::add_cost_line(&lib, style_id, line(CostCategory::Labels, "Woven label", "USD")).unwrap();
    let label = sheet.lines[1].id;
    cost_sheet::set_unit_cost(&lib, blank, t50, Some("12.50")).unwrap();
    cost_sheet::set_unit_cost(&lib, blank, t100, Some("11")).unwrap();
    cost_sheet::set_unit_cost(&lib, label, t50, Some("0.355")).unwrap();
    let sheet = cost_sheet::set_unit_cost(&lib, label, t100, Some("0.30")).unwrap();

    assert_eq!(sheet.home_currency.as_deref(), Some("USD"));
    assert_eq!(costs(&sheet, 0), [s("12.50"), s("11.00")]);
    assert_eq!(costs(&sheet, 1), [s("0.355"), s("0.30")]);
    // 12.50 + 0.355 = 12.855, rounded half away from zero to cents.
    assert_eq!(totals(&sheet), [s("12.86"), s("11.30")]);
    assert_eq!(cost_sheet::cost_sheet(&lib, style_id).unwrap(), sheet);
}

#[test]
fn totals_convert_mixed_quote_currencies_with_the_rates_the_founder_enters() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    cost_sheet::set_home_currency(&lib, "USD").unwrap();
    cost_sheet::set_exchange_rate(&lib, "eur", "1.0825").unwrap();
    let settings = cost_sheet::set_exchange_rate(&lib, "JPY", "0.0067").unwrap();
    assert_eq!(settings.home_currency.as_deref(), Some("USD"));
    assert_eq!(
        settings.rates.iter().map(|r| (r.currency.as_str(), r.rate.to_string())).collect::<Vec<_>>(),
        [("EUR", "1.0825".to_string()), ("JPY", "0.0067".to_string())]
    );
    assert_eq!(cost_sheet::currency_settings(&lib).unwrap(), settings);

    let t100 = cost_sheet::add_quantity_tier(&lib, style_id, 100).unwrap().tiers[0].id;
    let t300 = cost_sheet::add_quantity_tier(&lib, style_id, 300).unwrap().tiers[1].id;
    let blank = cost_sheet::add_cost_line(&lib, style_id, line(CostCategory::Base, "Blank tee", "USD")).unwrap().lines[0].id;
    let print =
        cost_sheet::add_cost_line(&lib, style_id, line(CostCategory::Decoration, "Screen print", "EUR")).unwrap().lines[1].id;
    let label = cost_sheet::add_cost_line(&lib, style_id, line(CostCategory::Labels, "Woven label", "JPY")).unwrap().lines[2].id;
    cost_sheet::set_unit_cost(&lib, blank, t100, Some("11.00")).unwrap();
    cost_sheet::set_unit_cost(&lib, blank, t300, Some("10.00")).unwrap();
    cost_sheet::set_unit_cost(&lib, print, t100, Some("4.20")).unwrap();
    cost_sheet::set_unit_cost(&lib, print, t300, Some("3.10")).unwrap();
    cost_sheet::set_unit_cost(&lib, label, t100, Some("45")).unwrap();
    let sheet = cost_sheet::set_unit_cost(&lib, label, t300, Some("38")).unwrap();

    // Each line keeps its quote currency and its amounts as entered.
    assert_eq!(sheet.lines.iter().map(|l| l.currency.as_str()).collect::<Vec<_>>(), ["USD", "EUR", "JPY"]);
    assert_eq!(costs(&sheet, 1), [s("4.20"), s("3.10")]);
    assert_eq!(costs(&sheet, 2), [s("45.00"), s("38.00")]);
    // 100: 11.00 + 4.20 × 1.0825 (4.5465) + 45 × 0.0067 (0.3015) = 15.848 → 15.85
    // 300: 10.00 + 3.10 × 1.0825 (3.35575) + 38 × 0.0067 (0.2546) = 13.61035 → 13.61
    assert_eq!(totals(&sheet), [s("15.85"), s("13.61")]);
    assert!(sheet.missing_rates.is_empty());

    // A new rate changes the totals; the quotes stay.
    cost_sheet::set_exchange_rate(&lib, "EUR", "1.10").unwrap();
    let sheet = cost_sheet::cost_sheet(&lib, style_id).unwrap();
    // 100: 11.00 + 4.62 + 0.3015 = 15.9215 → 15.92
    assert_eq!(totals(&sheet)[0], s("15.92"));
    assert_eq!(costs(&sheet, 1), [s("4.20"), s("3.10")]);
}

#[test]
fn a_tier_has_no_total_until_the_home_currency_and_every_rate_are_set() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let tier = cost_sheet::add_quantity_tier(&lib, style_id, 50).unwrap().tiers[0].id;
    let print = cost_sheet::add_cost_line(&lib, style_id, line(CostCategory::Decoration, "Print", "eur")).unwrap().lines[0].id;
    cost_sheet::set_unit_cost(&lib, print, tier, Some("3")).unwrap();

    let sheet = cost_sheet::cost_sheet(&lib, style_id).unwrap();
    assert_eq!(sheet.home_currency, None);
    assert_eq!(totals(&sheet), [None]);
    assert!(cost_sheet::unit_cost_at_tier(&lib, style_id, 50).is_err());

    cost_sheet::set_home_currency(&lib, "USD").unwrap();
    let sheet = cost_sheet::cost_sheet(&lib, style_id).unwrap();
    assert_eq!(totals(&sheet), [None]);
    assert_eq!(sheet.missing_rates, ["EUR"]);
    let refused = cost_sheet::unit_cost_at_tier(&lib, style_id, 50).unwrap_err();
    assert!(refused.to_string().contains("EUR"), "{refused}");

    cost_sheet::set_exchange_rate(&lib, "EUR", "1.5").unwrap();
    assert_eq!(totals(&cost_sheet::cost_sheet(&lib, style_id).unwrap()), [s("4.50")]);

    cost_sheet::remove_exchange_rate(&lib, "EUR").unwrap();
    assert_eq!(totals(&cost_sheet::cost_sheet(&lib, style_id).unwrap()), [None]);
}

#[test]
fn production_orders_read_the_unit_cost_at_a_tier_or_the_highest_tier_within_a_quantity() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    cost_sheet::set_home_currency(&lib, "GBP").unwrap();
    cost_sheet::set_exchange_rate(&lib, "USD", "0.8").unwrap();
    let mut tiers = Vec::new();
    for quantity in [50, 100, 300] {
        tiers = cost_sheet::add_quantity_tier(&lib, style_id, quantity).unwrap().tiers;
    }
    let tee = cost_sheet::add_cost_line(&lib, style_id, line(CostCategory::Base, "Tee", "USD")).unwrap().lines[0].id;
    for (tier, cost) in tiers.iter().zip(["15", "12.5", "10"]) {
        cost_sheet::set_unit_cost(&lib, tee, tier.id, Some(cost)).unwrap();
    }

    assert_eq!(cost_sheet::unit_cost_at_tier(&lib, style_id, 100).unwrap().to_string(), "10.00");
    assert_eq!(cost_sheet::unit_cost_at_tier(&lib, style_id, 100).unwrap().cents(), 1000);
    assert!(cost_sheet::unit_cost_at_tier(&lib, style_id, 120).is_err());

    let at = |q| {
        cost_sheet::unit_cost_for_quantity(&lib, style_id, q).unwrap().map(|c| (c.tier_quantity, c.unit_cost.cents()))
    };
    assert_eq!(at(49), None);
    assert_eq!(at(50), Some((50, 1200)));
    assert_eq!(at(299), Some((100, 1000)));
    assert_eq!(at(1000), Some((300, 800)));
}

#[test]
fn the_founder_edits_tiers_lines_and_cells() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    let vendor_id = vendor::create_vendor(
        &lib,
        VendorInput { name: "Ink House".into(), roles: vec![VendorRole::Decorator], ..Default::default() },
    )
    .unwrap()
    .id;
    cost_sheet::set_home_currency(&lib, "USD").unwrap();
    let t50 = cost_sheet::add_quantity_tier(&lib, style_id, 50).unwrap().tiers[0].id;
    let t100 = cost_sheet::add_quantity_tier(&lib, style_id, 100).unwrap().tiers[1].id;
    let id = cost_sheet::add_cost_line(&lib, style_id, line(CostCategory::Trims, "Thread", "USD")).unwrap().lines[0].id;
    cost_sheet::set_unit_cost(&lib, id, t50, Some("2")).unwrap();
    cost_sheet::set_unit_cost(&lib, id, t100, Some("1")).unwrap();

    // A tier can change quantity and keeps its costs; it moves into order.
    let sheet = cost_sheet::set_quantity_tier(&lib, t50, 200).unwrap();
    assert_eq!(sheet.tiers.iter().map(|t| t.quantity).collect::<Vec<_>>(), [100, 200]);
    assert_eq!(costs(&sheet, 0), [s("1.00"), s("2.00")]);
    assert!(cost_sheet::set_quantity_tier(&lib, t50, 100).is_err(), "two tiers of 100");
    assert!(cost_sheet::add_quantity_tier(&lib, style_id, 0).is_err());

    // A line edit keeps the costs.
    let input = CostLineInput {
        category: CostCategory::Decoration,
        vendor_id: Some(vendor_id),
        description: " Print ".into(),
        currency: "USD".into(),
    };
    let sheet = cost_sheet::update_cost_line(&lib, id, input).unwrap();
    let edited = &sheet.lines[0];
    assert_eq!(edited.category, CostCategory::Decoration);
    assert_eq!(edited.vendor_name.as_deref(), Some("Ink House"));
    assert_eq!(edited.description, "Print");
    assert_eq!(costs(&sheet, 0), [s("1.00"), s("2.00")]);

    // A blank cell clears and counts as zero.
    let sheet = cost_sheet::set_unit_cost(&lib, id, t100, Some(" ")).unwrap();
    assert_eq!(costs(&sheet, 0), [None, s("2.00")]);
    assert_eq!(totals(&sheet), [s("0.00"), s("2.00")]);

    // Bad input is refused.
    assert!(cost_sheet::set_unit_cost(&lib, id, t50, Some("-1")).is_err());
    assert!(cost_sheet::set_unit_cost(&lib, id, t50, Some("1.00001")).is_err());
    assert!(cost_sheet::set_unit_cost(&lib, id, t50, Some("1,50")).is_err());
    assert!(cost_sheet::add_cost_line(&lib, style_id, line(CostCategory::Other, "x", "dollars")).is_err());
    assert!(cost_sheet::set_exchange_rate(&lib, "EUR", "0").is_err());
    let drop_id = style::get_style(&lib, style_id).unwrap().drop_id;
    let other_style = style::create_style(&lib, drop_id, "Hoodie", "Black").unwrap().id;
    let other_tier = cost_sheet::add_quantity_tier(&lib, other_style, 50).unwrap().tiers[0].id;
    assert!(cost_sheet::set_unit_cost(&lib, id, other_tier, Some("1")).is_err());

    // Removing a tier and a line.
    let sheet = cost_sheet::remove_quantity_tier(&lib, t100).unwrap();
    assert_eq!(sheet.tiers.len(), 1);
    assert_eq!(costs(&sheet, 0), [s("2.00")]);
    let sheet = cost_sheet::remove_cost_line(&lib, id).unwrap();
    assert!(sheet.lines.is_empty());
    assert_eq!(totals(&sheet), [s("0.00")]);
}

#[test]
fn duplicating_a_style_copies_its_cost_sheet() {
    let env = TestEnv::new();
    let lib = env.library();
    let style_id = a_style(&lib);
    cost_sheet::set_home_currency(&lib, "USD").unwrap();
    cost_sheet::set_exchange_rate(&lib, "EUR", "1.25").unwrap();
    let t50 = cost_sheet::add_quantity_tier(&lib, style_id, 50).unwrap().tiers[0].id;
    cost_sheet::add_quantity_tier(&lib, style_id, 100).unwrap();
    let a = cost_sheet::add_cost_line(&lib, style_id, line(CostCategory::Base, "Tee", "USD")).unwrap().lines[0].id;
    let b = cost_sheet::add_cost_line(&lib, style_id, line(CostCategory::Packaging, "Bag", "EUR")).unwrap().lines[1].id;
    cost_sheet::set_unit_cost(&lib, a, t50, Some("9.99")).unwrap();
    let original = cost_sheet::set_unit_cost(&lib, b, t50, Some("0.40")).unwrap();

    let copy = style::duplicate_style(&lib, style_id, "Black").unwrap();
    let copied = cost_sheet::cost_sheet(&lib, copy.id).unwrap();

    assert_eq!(copied.style_id, copy.id);
    assert_eq!(copied.tiers.iter().map(|t| t.quantity).collect::<Vec<_>>(), [50, 100]);
    assert!(copied.tiers.iter().all(|t| original.tiers.iter().all(|o| o.id != t.id)), "new tier IDs");
    assert_eq!(
        copied.lines.iter().map(|l| (l.category, l.description.as_str(), l.currency.as_str())).collect::<Vec<_>>(),
        [(CostCategory::Base, "Tee", "USD"), (CostCategory::Packaging, "Bag", "EUR")]
    );
    assert_eq!(costs(&copied, 0), [s("9.99"), None]);
    assert_eq!(costs(&copied, 1), [s("0.40"), None]);
    // 9.99 + 0.40 × 1.25 = 10.49
    assert_eq!(totals(&copied), [s("10.49"), s("0.00")]);

    // The copy is independent of the original.
    cost_sheet::set_unit_cost(&lib, copied.lines[0].id, copied.tiers[0].id, Some("5")).unwrap();
    assert_eq!(cost_sheet::cost_sheet(&lib, style_id).unwrap(), original);
}
