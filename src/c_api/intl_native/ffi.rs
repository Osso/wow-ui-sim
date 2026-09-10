//! Private C ABI. ICU types and version-renamed symbols never cross this boundary.
use std::ffi::{CStr, c_char};

use super::{Error, NumberStyle, ParsedCurrency, checked_length};

#[repr(C)]
#[derive(Default)]
struct NativeString {
    data: *mut u8,
    length: i32,
}

impl Drop for NativeString {
    fn drop(&mut self) {
        // SAFETY: data is null or was allocated by the shim; this owner frees it once.
        unsafe { wow_icu_string_free(self.data) };
    }
}

impl NativeString {
    fn copy_string(&self) -> Result<String, Error> {
        if self.data.is_null() || self.length < 0 {
            return Err(Error("ICU4C returned an invalid output buffer".into()));
        }
        // SAFETY: successful shim output owns at least length bytes until this guard drops.
        let bytes = unsafe { std::slice::from_raw_parts(self.data, self.length as usize) };
        String::from_utf8(bytes.to_vec())
            .map_err(|error| Error(format!("ICU4C produced invalid UTF-8: {error}")))
    }
}

#[repr(C)]
#[derive(Default)]
struct NativeError {
    code: i32,
    operation: *const c_char,
}

impl NativeError {
    fn into_error(self, locale: &CStr) -> Error {
        // SAFETY: the shim supplies static NUL-terminated diagnostic strings.
        let operation = unsafe { diagnostic(self.operation) };
        // SAFETY: any integer code is accepted; ICU returns a static diagnostic name.
        let name = unsafe { diagnostic(wow_icu_error_name(self.code)) };
        Error(format!(
            "ICU4C {operation} for locale {locale:?}: {name} ({})",
            self.code
        ))
    }
}

unsafe fn diagnostic(pointer: *const c_char) -> String {
    if pointer.is_null() {
        return "unspecified native error".into();
    }
    // SAFETY: callers supply ICU/shim static diagnostic pointers or null.
    unsafe { CStr::from_ptr(pointer) }
        .to_string_lossy()
        .into_owned()
}

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
    fn wow_icu_string_free(data: *mut u8);
    fn wow_icu_error_name(code: i32) -> *const c_char;
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
        0 => {
            let currency_code = if with_currency {
                String::from_utf8(currency[..3].to_vec()).map_err(|error| {
                    Error(format!("ICU4C returned invalid currency bytes: {error}"))
                })?
            } else {
                String::new()
            };
            Ok(Some(ParsedCurrency {
                amount,
                currency_code,
            }))
        }
        1 => Ok(None),
        _ => Err(error.into_error(locale)),
    }
}

pub(super) fn version() -> String {
    let mut version = [0u8; 4];
    // SAFETY: ICU version output is exactly four uint8_t components.
    unsafe { wow_icu_version(version.as_mut_ptr()) };
    version.map(|component| component.to_string()).join(".")
}
