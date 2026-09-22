//! The private duration-only ICU ABI shared by PTR and Forever.
use std::ffi::c_char;

use crate::c_api::native_icu::{
    Error, NativeError, NativeString, checked_length, locale_string, validate_finite,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct DurationPart {
    pub value: f64,
    pub unit: i32,
    pub fraction_digits: i32,
}

unsafe extern "C" {
    fn wow_icu_duration_units(
        locale: *const c_char,
        locale_length: i32,
        parts: *const DurationPart,
        count: i32,
        width: i32,
        whitespace: i32,
        output: *mut NativeString,
        error: *mut NativeError,
    ) -> i32;
}

fn validate_parts(parts: &[DurationPart], width: i32, whitespace: i32) -> Result<(), Error> {
    if !(1..=4).contains(&parts.len())
        || !(0..=2).contains(&width)
        || !(0..=2).contains(&whitespace)
    {
        return Err(Error(
            "invalid duration unit list, width, or whitespace mode".into(),
        ));
    }
    for part in parts {
        validate_finite(part.value)?;
        let valid_unit = (0..=3).contains(&part.unit);
        let valid_precision = matches!(part.fraction_digits, 0 | 3);
        if part.value < 0.0 || !valid_unit || !valid_precision {
            return Err(Error(
                "invalid duration unit value, interval, or precision".into(),
            ));
        }
    }
    Ok(())
}

pub(super) fn format_duration_units(
    locale: &str,
    parts: &[DurationPart],
    width: i32,
    whitespace: i32,
) -> Result<String, Error> {
    validate_parts(parts, width, whitespace)?;
    let locale = locale_string(locale)?;
    let locale_length = checked_length(locale.to_bytes().len(), "locale")?;
    let count = checked_length(parts.len(), "duration unit count")?;
    let mut output = NativeString::default();
    let mut error = NativeError::default();
    // SAFETY: repr(C) parts and locale remain live; output has one RAII owner.
    let status = unsafe {
        wow_icu_duration_units(
            locale.as_ptr(),
            locale_length,
            parts.as_ptr(),
            count,
            width,
            whitespace,
            &mut output,
            &mut error,
        )
    };
    if status != 0 {
        return Err(error.into_error(&locale));
    }
    output.copy_string()
}
