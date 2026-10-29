//! Exact money for the Cost Sheet. No floats.
//!
//! - [`UnitCost`]: a quote amount in a line's currency, kept in
//!   ten-thousandths (4 decimal places), so a quote such as 0.0350 per label
//!   stays exact.
//! - [`ExchangeRate`]: home-currency units per one unit of another currency,
//!   kept in millionths (6 decimal places).
//! - [`HomeAmount`]: a total in the home currency, in cents (2 decimal
//!   places).
//!
//! Rounding rule: a tier total is the exact sum of `unit cost × rate` over
//! its lines, rounded once, at the end, to cents, half away from zero. Lines
//! are never rounded one by one. All amounts are zero or more, so half away
//! from zero is the same as half up here.
//!
//! All three cross IPC as decimal strings (for example "12.50").

use std::fmt;

use crate::error::{CoreError, CoreResult};

/// A unit cost in a quote currency, in ten-thousandths. Zero or more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitCost(i64);

/// Home-currency units per one unit of a quote currency, in millionths.
/// More than zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExchangeRate(i64);

/// An amount in the home currency, in cents. Zero or more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct HomeAmount(i64);

const COST_PLACES: u32 = 4;
const RATE_PLACES: u32 = 6;
const HOME_PLACES: u32 = 2;

impl UnitCost {
    /// Reads a decimal such as "12.5" or "0.035". Refuses negatives, more
    /// than 4 decimal places, and anything that is not a plain decimal.
    pub fn parse(text: &str) -> CoreResult<UnitCost> {
        parse_decimal(text, COST_PLACES, "a unit cost").map(UnitCost)
    }

    pub fn from_ten_thousandths(value: i64) -> UnitCost {
        UnitCost(value)
    }

    pub fn ten_thousandths(self) -> i64 {
        self.0
    }
}

impl ExchangeRate {
    /// The rate of the home currency to itself.
    pub const ONE: ExchangeRate = ExchangeRate(1_000_000);

    /// Reads a decimal such as "1.0825". Refuses zero, negatives, more than
    /// 6 decimal places, and anything that is not a plain decimal.
    pub fn parse(text: &str) -> CoreResult<ExchangeRate> {
        let value = parse_decimal(text, RATE_PLACES, "an exchange rate")?;
        if value == 0 {
            return Err(CoreError::InvalidInput { reason: "an exchange rate must be more than zero".into() });
        }
        Ok(ExchangeRate(value))
    }

    pub fn from_millionths(value: i64) -> ExchangeRate {
        ExchangeRate(value)
    }

    pub fn millionths(self) -> i64 {
        self.0
    }
}

impl HomeAmount {
    pub fn from_cents(cents: i64) -> HomeAmount {
        HomeAmount(cents)
    }

    pub fn cents(self) -> i64 {
        self.0
    }

    /// The total of `unit cost × rate` over `items`, summed exactly and
    /// rounded once to cents, half away from zero.
    pub fn total(items: impl IntoIterator<Item = (UnitCost, ExchangeRate)>) -> HomeAmount {
        // Each product is in 10^-(4 + 6) home units.
        let exact: i128 = items.into_iter().map(|(cost, rate)| cost.0 as i128 * rate.0 as i128).sum();
        let divisor = 10i128.pow(COST_PLACES + RATE_PLACES - HOME_PLACES);
        let cents = (exact + divisor / 2) / divisor;
        HomeAmount(i64::try_from(cents).unwrap_or(i64::MAX))
    }
}

/// Parses a non-negative decimal into an integer with `places` decimal
/// places.
fn parse_decimal(text: &str, places: u32, what: &str) -> CoreResult<i64> {
    let refuse = || CoreError::InvalidInput {
        reason: format!("{what} must be a number of zero or more with at most {places} decimal places, like 12.50"),
    };
    let text = text.trim();
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    let digits = |s: &str| s.chars().all(|c| c.is_ascii_digit());
    if (whole.is_empty() && fraction.is_empty()) || !digits(whole) || !digits(fraction) {
        return Err(refuse());
    }
    let fraction = fraction.trim_end_matches('0');
    if fraction.len() > places as usize {
        return Err(refuse());
    }
    let whole: i64 = if whole.is_empty() { 0 } else { whole.parse().map_err(|_| refuse())? };
    let fraction_value: i64 = if fraction.is_empty() { 0 } else { fraction.parse().map_err(|_| refuse())? };
    let scale = 10i64.pow(places);
    whole
        .checked_mul(scale)
        .and_then(|w| w.checked_add(fraction_value * 10i64.pow(places - fraction.len() as u32)))
        .ok_or_else(refuse)
}

/// Writes `value` (with `places` decimal places) with at least `min_places`
/// decimals and no extra trailing zeros.
fn write_decimal(f: &mut fmt::Formatter, value: i64, places: u32, min_places: usize) -> fmt::Result {
    let scale = 10i64.pow(places);
    let fraction = format!("{:0width$}", value % scale, width = places as usize);
    let trimmed = fraction.trim_end_matches('0');
    let shown = if trimmed.len() < min_places { &fraction[..min_places] } else { trimmed };
    if shown.is_empty() {
        write!(f, "{}", value / scale)
    } else {
        write!(f, "{}.{}", value / scale, shown)
    }
}

impl fmt::Display for UnitCost {
    /// At least 2 decimals: "12.50", "0.355".
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_decimal(f, self.0, COST_PLACES, 2)
    }
}

impl fmt::Display for ExchangeRate {
    /// No extra zeros: "1", "1.0825".
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_decimal(f, self.0, RATE_PLACES, 0)
    }
}

impl fmt::Display for HomeAmount {
    /// Always 2 decimals: "12.86".
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_decimal(f, self.0, HOME_PLACES, 2)
    }
}

macro_rules! serialize_as_text {
    ($($t:ty),*) => {$(
        impl serde::Serialize for $t {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.collect_str(self)
            }
        }
    )*};
}

serialize_as_text!(UnitCost, ExchangeRate, HomeAmount);
