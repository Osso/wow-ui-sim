//! Native search owns ICU resources; this module owns the returned range buffer.
use std::ffi::c_char;

use super::ffi::NativeError;
use super::{Error, checked_length, locale_string};

#[repr(i32)]
#[derive(Clone, Copy, Debug)]
pub enum SearchStrength {
    Primary = 0,
    Secondary = 1,
    Tertiary = 2,
    Quaternary = 3,
    Identical = 4,
}

#[repr(C)]
struct NativeMatch {
    start: i32,
    end: i32,
}

#[repr(C)]
#[derive(Default)]
struct NativeMatches {
    data: *mut NativeMatch,
    length: i32,
}

impl Drop for NativeMatches {
    fn drop(&mut self) {
        // SAFETY: the shim owns this allocation; errors reset it to null.
        unsafe { wow_icu_matches_free(self.data) };
    }
}

unsafe extern "C" {
    fn wow_icu_find_matches(
        locale: *const c_char,
        locale_length: i32,
        text: *const u8,
        text_length: i32,
        pattern: *const u8,
        pattern_length: i32,
        strength: i32,
        output: *mut NativeMatches,
        error: *mut NativeError,
    ) -> i32;
    fn wow_icu_matches_free(matches: *mut NativeMatch);
}

/// Returns zero-based UTF-8 starts for forward, non-overlapping, positive-length
/// ICU4C matches. Canonical normalization is enabled for every strength.
pub fn find_string_matches(
    locale: &str,
    text: &str,
    pattern: &str,
    strength: SearchStrength,
) -> Result<Vec<usize>, Error> {
    let locale = locale_string(locale)?;
    let locale_length = checked_length(locale.as_bytes().len(), "search locale")?;
    let text_length = checked_length(text.len(), "search text")?;
    let pattern_length = checked_length(pattern.len(), "search pattern")?;
    let mut matches = NativeMatches::default();
    let mut error = NativeError::default();
    // SAFETY: inputs remain live for this synchronous call; checked byte lengths
    // match their buffers. The shim writes initialized ABI-compatible outputs.
    let status = unsafe {
        wow_icu_find_matches(
            locale.as_ptr(),
            locale_length,
            text.as_ptr(),
            text_length,
            pattern.as_ptr(),
            pattern_length,
            strength as i32,
            &mut matches,
            &mut error,
        )
    };
    if status != 0 {
        return Err(error.into_error(&locale));
    }
    matches.byte_offsets(text)
}

impl NativeMatches {
    fn ranges(&self, text_length: usize) -> Result<&[NativeMatch], Error> {
        let count = usize::try_from(self.length)
            .map_err(|_| Error("ICU4C returned a negative search match count".into()))?;
        if count > text_length || count > isize::MAX as usize / size_of::<NativeMatch>() {
            return Err(Error("ICU4C returned an invalid search match count".into()));
        }
        if count == 0 {
            return Ok(&[]);
        }
        if self.data.is_null() {
            return Err(Error("ICU4C returned a null search match buffer".into()));
        }
        // SAFETY: successful shim output allocated length initialized entries,
        // owned by this guard; the size is bounded above and the pointer is nonnull.
        Ok(unsafe { std::slice::from_raw_parts(self.data, count) })
    }

    fn byte_offsets(&self, text: &str) -> Result<Vec<usize>, Error> {
        let ranges = self.ranges(text.len())?;
        let boundaries = utf16_byte_boundaries(text)?;
        let mut offsets = Vec::new();
        offsets
            .try_reserve_exact(ranges.len())
            .map_err(|error| Error(format!("allocate search offsets: {error}")))?;
        let mut previous_end = 0;
        for range in ranges {
            let start = scalar_offset(&boundaries, range.start)?;
            let end = scalar_offset(&boundaries, range.end)?;
            if start < previous_end || end <= start {
                return Err(Error(
                    "ICU4C returned overlapping or empty search ranges".into(),
                ));
            }
            offsets.push(start);
            previous_end = end;
        }
        Ok(offsets)
    }
}

fn utf16_byte_boundaries(text: &str) -> Result<Vec<usize>, Error> {
    let capacity = text
        .encode_utf16()
        .count()
        .checked_add(1)
        .ok_or_else(|| Error("search boundary map length overflow".into()))?;
    let mut boundaries = Vec::new();
    boundaries
        .try_reserve_exact(capacity)
        .map_err(|error| Error(format!("allocate search boundary map: {error}")))?;
    boundaries.push(0);
    for (byte, scalar) in text.char_indices() {
        if scalar.len_utf16() == 2 {
            boundaries.push(usize::MAX);
        }
        boundaries.push(byte + scalar.len_utf8());
    }
    Ok(boundaries)
}

fn scalar_offset(boundaries: &[usize], offset: i32) -> Result<usize, Error> {
    let index = usize::try_from(offset)
        .map_err(|_| Error("ICU4C returned a negative search offset".into()))?;
    boundaries
        .get(index)
        .copied()
        .filter(|value| *value != usize::MAX)
        .ok_or_else(|| Error("ICU4C search offset is not a Unicode scalar boundary".into()))
}
