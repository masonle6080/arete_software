//! Currency settings of the library: the home currency and the exchange
//! rates that the founder enters by hand. There are no live rates.
//!
//! The home currency is the library setting `home_currency`. Each rate says
//! how many home-currency units one unit of another currency is worth. The
//! home currency always has rate 1; a stored rate for it is ignored.

use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};

use super::money::ExchangeRate;
use crate::error::{CoreError, CoreResult};
use crate::library::Library;

const HOME_KEY: &str = "home_currency";

/// One exchange rate the founder entered.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRate {
    /// An ISO 4217 code such as "EUR".
    pub currency: String,
    /// Home-currency units per one unit of `currency`.
    pub rate: ExchangeRate,
}

/// The home currency and every exchange rate, by currency code.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencySettings {
    /// `None` until the founder sets it.
    pub home_currency: Option<String>,
    pub rates: Vec<CurrencyRate>,
}

/// Reads the home currency and every exchange rate.
pub fn currency_settings(lib: &Library) -> CoreResult<CurrencySettings> {
    let conn = lib.db();
    let mut stmt = conn.prepare("SELECT currency, rate FROM exchange_rate ORDER BY currency")?;
    let rates = stmt
        .query_map([], |r| Ok(CurrencyRate { currency: r.get(0)?, rate: ExchangeRate::from_millionths(r.get(1)?) }))?
        .collect::<Result<_, _>>()?;
    Ok(CurrencySettings { home_currency: home_currency(conn)?, rates })
}

/// Sets the home currency, which every Cost Sheet totals in.
pub fn set_home_currency(lib: &Library, currency: &str) -> CoreResult<CurrencySettings> {
    let currency = currency_code(currency)?;
    lib.writer()?.execute(
        "INSERT INTO library_setting (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        params![HOME_KEY, currency],
    )?;
    currency_settings(lib)
}

/// Sets how many home-currency units one unit of `currency` is worth, for
/// example "1.0825". It replaces any earlier rate for that currency.
pub fn set_exchange_rate(lib: &Library, currency: &str, rate: &str) -> CoreResult<CurrencySettings> {
    let currency = currency_code(currency)?;
    let rate = ExchangeRate::parse(rate)?;
    lib.writer()?.execute(
        "INSERT INTO exchange_rate (currency, rate) VALUES (?1, ?2)
         ON CONFLICT (currency) DO UPDATE SET rate = excluded.rate",
        params![currency, rate.millionths()],
    )?;
    currency_settings(lib)
}

/// Removes the exchange rate of `currency`. Cost Sheets with lines in that
/// currency then show no totals until a rate is set again.
pub fn remove_exchange_rate(lib: &Library, currency: &str) -> CoreResult<CurrencySettings> {
    let currency = currency_code(currency)?;
    lib.writer()?.execute("DELETE FROM exchange_rate WHERE currency = ?1", [currency])?;
    currency_settings(lib)
}

/// Trims and upper-cases a currency code, and checks that it is three
/// letters.
pub(crate) fn currency_code(text: &str) -> CoreResult<String> {
    let code = text.trim().to_ascii_uppercase();
    if code.len() != 3 || !code.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(CoreError::InvalidInput {
            reason: format!("a currency is a three-letter code such as USD or EUR, not \"{}\"", text.trim()),
        });
    }
    Ok(code)
}

pub(crate) fn home_currency(conn: &Connection) -> CoreResult<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM library_setting WHERE key = ?1", [HOME_KEY], |r| r.get(0))
        .optional()?)
}

/// The rates to convert into the home currency, by currency code. The home
/// currency maps to 1.
pub(crate) fn rates_to_home(conn: &Connection, home: &str) -> CoreResult<HashMap<String, ExchangeRate>> {
    let mut stmt = conn.prepare("SELECT currency, rate FROM exchange_rate")?;
    let mut rates: HashMap<String, ExchangeRate> = stmt
        .query_map([], |r| Ok((r.get(0)?, ExchangeRate::from_millionths(r.get(1)?))))?
        .collect::<Result<_, _>>()?;
    rates.insert(home.to_string(), ExchangeRate::ONE);
    Ok(rates)
}
