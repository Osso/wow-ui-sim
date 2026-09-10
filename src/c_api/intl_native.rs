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
mod tests;
