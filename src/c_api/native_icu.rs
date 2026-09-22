//! Shared ICU buffer, diagnostic, and input validation for private C API backends.
use std::ffi::{CStr, CString, c_char};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(pub(crate) String);

#[repr(C)]
#[derive(Default)]
pub(crate) struct NativeString {
    data: *mut u8,
    length: i32,
}

impl Drop for NativeString {
    fn drop(&mut self) {
        // SAFETY: the shim allocated this buffer; this owner frees it once.
        unsafe { wow_icu_string_free(self.data) };
    }
}

impl NativeString {
    pub(crate) fn copy_string(&self) -> Result<String, Error> {
        if self.data.is_null() || self.length < 0 {
            return Err(Error("ICU4C returned an invalid output buffer".into()));
        }
        // SAFETY: successful shim output owns length bytes until this guard drops.
        let bytes = unsafe { std::slice::from_raw_parts(self.data, self.length as usize) };
        String::from_utf8(bytes.to_vec())
            .map_err(|error| Error(format!("ICU4C produced invalid UTF-8: {error}")))
    }
}

#[repr(C)]
#[derive(Default)]
pub(crate) struct NativeError {
    code: i32,
    operation: *const c_char,
}

impl NativeError {
    pub(crate) fn into_error(self, context: &CStr) -> Error {
        // SAFETY: the shim supplies static NUL-terminated diagnostic strings.
        let operation = unsafe { diagnostic(self.operation) };
        // SAFETY: ICU accepts any integer code and returns a static diagnostic.
        let name = unsafe { diagnostic(wow_icu_error_name(self.code)) };
        Error(format!(
            "ICU4C {operation} for {context:?}: {name} ({})",
            self.code
        ))
    }
}

unsafe fn diagnostic(pointer: *const c_char) -> String {
    if pointer.is_null() {
        return "unspecified native error".into();
    }
    // SAFETY: callers supply static ICU/shim diagnostic pointers or null.
    unsafe { CStr::from_ptr(pointer) }
        .to_string_lossy()
        .into_owned()
}

unsafe extern "C" {
    fn wow_icu_string_free(data: *mut u8);
    fn wow_icu_error_name(code: i32) -> *const c_char;
}

pub(crate) fn validate_finite(value: f64) -> Result<(), Error> {
    if !value.is_finite() {
        return Err(Error("ICU4C formatting requires a finite number".into()));
    }
    Ok(())
}

pub(crate) fn locale_string(locale: &str) -> Result<CString, Error> {
    checked_length(locale.len(), "locale")?;
    if locale.is_empty() {
        return Err(Error("ICU4C locale must not be empty".into()));
    }
    CString::new(locale).map_err(|_| Error("ICU4C locale must not contain NUL".into()))
}

pub(crate) fn checked_length(length: usize, field: &str) -> Result<i32, Error> {
    // Leave one i32-representable unit for C/ICU's trailing terminator.
    if length >= i32::MAX as usize {
        return Err(Error(format!(
            "ICU4C {field} exceeds the supported i32 length"
        )));
    }
    Ok(length as i32)
}
