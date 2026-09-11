//! Feature-gated ICU4C number operations. Lua wiring and packaging are separate.
//! Native data/defaults may vary by installed ICU version; see `version()`.
mod ffi;

use std::ffi::CString;

#[repr(i32)]
#[derive(Clone, Copy, Debug)]
pub enum NumberStyle {
    Decimal = 0,
    Integer = 1,
    Percent = 2,
    Currency = 3,
}

/// WoW date/time selectors; the C shim maps them to ICU's style constants.
#[repr(i32)]
#[derive(Clone, Copy, Debug)]
pub enum DateTimeStyle {
    None = 0,
    Short = 1,
    Medium = 2,
    Long = 3,
    Full = 4,
}

/// WoW selector values; the C shim explicitly maps these to ICU's different order.
#[repr(i32)]
#[derive(Clone, Copy, Debug)]
pub enum CurrencyNameStyle {
    Symbol = 0,
    NarrowSymbol = 1,
    Long = 2,
    FormalSymbol = 3,
    VariantSymbol = 4,
}

#[derive(Debug, PartialEq)]
pub struct ParsedCurrency {
    pub amount: f64,
    pub currency_code: String,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(String);

/// ICU defaults for the selected locale/style. Integer uses zero fraction digits,
/// half-even rounding, and integer-only parsing; Currency uses the locale currency.
pub fn format_number(locale: &str, value: f64, style: NumberStyle) -> Result<String, Error> {
    validate_finite(value)?;
    ffi::format(&locale_string(locale)?, value, style, None)
}

/// Formats a finite amount with a three-ASCII-letter currency code, uppercased.
/// ICU's currency data supplies symbols, digits, and patterns.
pub fn format_currency(locale: &str, value: f64, currency: &str) -> Result<String, Error> {
    validate_finite(value)?;
    let currency = currency_string(currency)?;
    ffi::format(
        &locale_string(locale)?,
        value,
        NumberStyle::Currency,
        Some(&currency),
    )
}

/// Returns None for invalid, nonfinite, empty, or partially consumed number text.
/// Invalid locale/configuration and native operational failures return Err.
pub fn parse_number(locale: &str, text: &str, style: NumberStyle) -> Result<Option<f64>, Error> {
    let result = ffi::parse(&locale_string(locale)?, text, style, false)?;
    Ok(result.map(|parsed| parsed.amount))
}

/// Requires complete input consumption; returns ICU's parsed ISO currency code.
pub fn parse_currency(locale: &str, text: &str) -> Result<Option<ParsedCurrency>, Error> {
    ffi::parse(&locale_string(locale)?, text, NumberStyle::Currency, true)
}

/// Returns ICU's localized name only for a currency in its all-date catalogue.
pub fn currency_name(
    locale: &str,
    currency: &str,
    style: CurrencyNameStyle,
) -> Result<Option<String>, Error> {
    let code = currency_string(currency)?;
    ffi::currency_name(&locale_string(locale)?, &code, style)
}

/// Unknown currencies return None; a known zero-digit currency returns Some(0).
pub fn currency_fraction_digits(currency: &str) -> Result<Option<i32>, Error> {
    ffi::currency_fraction_digits(&currency_string(currency)?)
}

/// Formats Unix seconds in an explicit validated zone. Empty zone means UTC;
/// both styles None produce an empty string after validating the inputs.
pub fn format_date_time(
    locale: &str,
    seconds: f64,
    date_style: DateTimeStyle,
    time_style: DateTimeStyle,
    zone: &str,
) -> Result<String, Error> {
    validate_finite(seconds)?;
    let milliseconds = seconds * 1000.0;
    if !milliseconds.is_finite() {
        return Err(Error("ICU4C Unix seconds overflow milliseconds".into()));
    }
    if zone.contains('\0') {
        return Err(Error("ICU4C time zone must not contain NUL".into()));
    }
    let zone = if zone.is_empty() { "UTC" } else { zone };
    ffi::format_date_time(
        &locale_string(locale)?,
        milliseconds,
        date_style,
        time_style,
        zone,
    )
}

/// Runtime ICU library version, as four dot-separated numeric components.
pub fn version() -> String {
    ffi::version()
}

fn validate_finite(value: f64) -> Result<(), Error> {
    if !value.is_finite() {
        return Err(Error("ICU4C formatting requires a finite number".into()));
    }
    Ok(())
}

fn locale_string(locale: &str) -> Result<CString, Error> {
    checked_length(locale.len(), "locale")?;
    if locale.is_empty() {
        return Err(Error("ICU4C locale must not be empty".into()));
    }
    CString::new(locale).map_err(|_| Error("ICU4C locale must not contain NUL".into()))
}

fn currency_string(currency: &str) -> Result<CString, Error> {
    let valid = currency.len() == 3 && currency.bytes().all(|byte| byte.is_ascii_alphabetic());
    if !valid {
        return Err(Error(
            "ICU4C currency code must contain three ASCII letters".into(),
        ));
    }
    CString::new(currency.to_ascii_uppercase())
        .map_err(|_| Error("ICU4C currency code must not contain NUL".into()))
}

fn checked_length(length: usize, field: &str) -> Result<i32, Error> {
    // Leave one i32-representable unit for C/ICU's trailing terminator.
    if length >= i32::MAX as usize {
        return Err(Error(format!(
            "ICU4C {field} exceeds the supported i32 length"
        )));
    }
    Ok(length as i32)
}

#[cfg(test)]
mod currency_metadata_tests;
#[cfg(test)]
mod date_tests;
#[cfg(test)]
mod tests;
