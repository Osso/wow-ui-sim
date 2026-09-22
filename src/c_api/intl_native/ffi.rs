//! Private C ABI. ICU types and version-renamed symbols never cross this boundary.
use std::ffi::{CStr, c_char};

use super::{CurrencyNameStyle, DateTimeStyle, Error, NumberStyle, ParsedCurrency, checked_length};

use crate::c_api::native_icu::{NativeError, NativeString};

unsafe extern "C" {
    fn wow_icu_format(
        locale: *const c_char,
        locale_length: i32,
        style: i32,
        value: f64,
        currency: *const c_char,
        output: *mut NativeString,
        error: *mut NativeError,
    ) -> i32;
    fn wow_icu_parse(
        locale: *const c_char,
        locale_length: i32,
        style: i32,
        text: *const u8,
        text_length: i32,
        with_currency: i32,
        value: *mut f64,
        currency: *mut u8,
        error: *mut NativeError,
    ) -> i32;
    fn wow_icu_currency_name(
        locale: *const c_char,
        locale_length: i32,
        currency: *const c_char,
        style: i32,
        output: *mut NativeString,
        error: *mut NativeError,
    ) -> i32;
    fn wow_icu_currency_fraction_digits(
        currency: *const c_char,
        digits: *mut i32,
        error: *mut NativeError,
    ) -> i32;
    fn wow_icu_format_date_time(
        locale: *const c_char,
        locale_length: i32,
        milliseconds: f64,
        date_style: i32,
        time_style: i32,
        zone: *const u8,
        zone_length: i32,
        output: *mut NativeString,
        error: *mut NativeError,
    ) -> i32;
    fn wow_icu_display_name(
        target: *const c_char,
        target_length: i32,
        display: *const c_char,
        display_length: i32,
        output: *mut NativeString,
        error: *mut NativeError,
    ) -> i32;
    fn wow_icu_transliterate(
        text: *const u8,
        text_length: i32,
        id: *const u8,
        id_length: i32,
        output: *mut NativeString,
        error: *mut NativeError,
    ) -> i32;
    fn wow_icu_version(version: *mut u8);
}

pub(super) fn format(
    locale: &CStr,
    value: f64,
    style: NumberStyle,
    currency: Option<&CStr>,
) -> Result<String, Error> {
    let locale_length = checked_length(locale.to_bytes().len(), "locale")?;
    let mut output = NativeString::default();
    let mut error = NativeError::default();
    let currency = currency.map_or(std::ptr::null(), CStr::as_ptr);
    // SAFETY: input C strings live through this synchronous call; outputs are writable locals.
    let status = unsafe {
        wow_icu_format(
            locale.as_ptr(),
            locale_length,
            style as i32,
            value,
            currency,
            &mut output,
            &mut error,
        )
    };
    if status != 0 {
        return Err(error.into_error(locale));
    }
    output.copy_string()
}

pub(super) fn parse(
    locale: &CStr,
    text: &str,
    style: NumberStyle,
    with_currency: bool,
) -> Result<Option<ParsedCurrency>, Error> {
    let locale_length = checked_length(locale.to_bytes().len(), "locale")?;
    let text_length = checked_length(text.len(), "number text")?;
    let mut amount = 0.0;
    let mut currency = [0u8; 4];
    let mut error = NativeError::default();
    // SAFETY: text is valid UTF-8 with explicit checked byte length; all outputs have C ABI sizes.
    let status = unsafe {
        wow_icu_parse(
            locale.as_ptr(),
            locale_length,
            style as i32,
            text.as_ptr(),
            text_length,
            i32::from(with_currency),
            &mut amount,
            currency.as_mut_ptr(),
            &mut error,
        )
    };
    match status {
        0 => decode_parsed_currency(amount, currency, with_currency).map(Some),
        1 => Ok(None),
        _ => Err(error.into_error(locale)),
    }
}

fn decode_parsed_currency(
    amount: f64,
    currency: [u8; 4],
    with_currency: bool,
) -> Result<ParsedCurrency, Error> {
    let currency_code = if with_currency {
        String::from_utf8(currency[..3].to_vec())
            .map_err(|error| Error(format!("ICU4C returned invalid currency bytes: {error}")))?
    } else {
        String::new()
    };
    Ok(ParsedCurrency {
        amount,
        currency_code,
    })
}

pub(super) fn currency_name(
    locale: &CStr,
    currency: &CStr,
    style: CurrencyNameStyle,
) -> Result<Option<String>, Error> {
    let locale_length = checked_length(locale.to_bytes().len(), "locale")?;
    let mut output = NativeString::default();
    let mut error = NativeError::default();
    // SAFETY: checked C strings live through this call; outputs are writable locals.
    // NativeString owns any allocated output and frees it on every return path.
    let status = unsafe {
        wow_icu_currency_name(
            locale.as_ptr(),
            locale_length,
            currency.as_ptr(),
            style as i32,
            &mut output,
            &mut error,
        )
    };
    match status {
        0 => output.copy_string().map(Some),
        1 => Ok(None),
        _ => Err(error.into_error(locale)),
    }
}

pub(super) fn currency_fraction_digits(currency: &CStr) -> Result<Option<i32>, Error> {
    let mut digits = 0;
    let mut error = NativeError::default();
    // SAFETY: validated three-letter C string lives through the call; outputs are
    // writable locals. Status distinguishes unknown currency from valid zero digits.
    let status =
        unsafe { wow_icu_currency_fraction_digits(currency.as_ptr(), &mut digits, &mut error) };
    match status {
        0 => Ok(Some(digits)),
        1 => Ok(None),
        _ => Err(error.into_error(currency)),
    }
}

pub(super) fn format_date_time(
    locale: &CStr,
    milliseconds: f64,
    date_style: DateTimeStyle,
    time_style: DateTimeStyle,
    zone: &str,
) -> Result<String, Error> {
    let locale_length = checked_length(locale.to_bytes().len(), "locale")?;
    let zone_length = checked_length(zone.len(), "time zone")?;
    let mut output = NativeString::default();
    let mut error = NativeError::default();
    // SAFETY: locale and UTF-8 zone remain valid through this synchronous call;
    // checked lengths describe their buffers, and outputs are writable locals.
    let status = unsafe {
        wow_icu_format_date_time(
            locale.as_ptr(),
            locale_length,
            milliseconds,
            date_style as i32,
            time_style as i32,
            zone.as_ptr(),
            zone_length,
            &mut output,
            &mut error,
        )
    };
    if status != 0 {
        return Err(error.into_error(locale));
    }
    output.copy_string()
}

pub(super) fn display_name(target: &CStr, display: &CStr) -> Result<String, Error> {
    let target_length = checked_length(target.to_bytes().len(), "target locale")?;
    let display_length = checked_length(display.to_bytes().len(), "display locale")?;
    let mut output = NativeString::default();
    let mut error = NativeError::default();
    // SAFETY: checked C strings remain live during the call; output owns the
    // shim allocation and frees it on every return path.
    let status = unsafe {
        wow_icu_display_name(
            target.as_ptr(),
            target_length,
            display.as_ptr(),
            display_length,
            &mut output,
            &mut error,
        )
    };
    if status != 0 {
        return Err(error.into_error(target));
    }
    output.copy_string()
}

pub(super) fn transliterate(text: &str, id: &str) -> Result<String, Error> {
    let text_length = checked_length(text.len(), "transliteration text")?;
    let id_length = checked_length(id.len(), "transliterator ID")?;
    let mut output = NativeString::default();
    let mut error = NativeError::default();
    // SAFETY: valid UTF-8 inputs live through the call with checked byte lengths;
    // the shim owns its UTF-16 copies and output is freed by NativeString.
    let status = unsafe {
        wow_icu_transliterate(
            text.as_ptr(),
            text_length,
            id.as_ptr(),
            id_length,
            &mut output,
            &mut error,
        )
    };
    if status != 0 {
        return Err(error.into_error(c"transliteration"));
    }
    output.copy_string()
}

pub(super) fn version() -> String {
    let mut version = [0u8; 4];
    // SAFETY: ICU version output is exactly four uint8_t components.
    unsafe { wow_icu_version(version.as_mut_ptr()) };
    version.map(|component| component.to_string()).join(".")
}
