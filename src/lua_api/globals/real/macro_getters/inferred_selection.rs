//! INFERRED representation policies for the simulator's existing macro rows.
//!
//! R88l.19 proves name search and unsigned selection, not exact name collation
//! or integer widths. The all-empty vacancy sentinel is not a native rule.

use crate::lua_api::state::MacroInfo;

pub(super) fn is_occupied(row: &MacroInfo) -> bool {
    // Existing writers create/clear holes with default rows. The adapter
    // cannot distinguish those from an explicitly supplied all-empty record.
    !(row.name.is_empty() && row.icon.is_empty() && row.body.is_empty())
}

pub(super) fn index(number: f64) -> Option<usize> {
    // Adopt an i64 input domain, then a u32 one-based wrap as simulator policy.
    const I64_LIMIT: f64 = (1_u64 << 63) as f64;
    if !(-I64_LIMIT..I64_LIMIT).contains(&number) {
        return None;
    }
    Some((number.trunc() as i64 as u32).wrapping_sub(1) as usize)
}

pub(super) fn by_name<'a>(rows: &'a [MacroInfo], name: &[u8]) -> Option<&'a MacroInfo> {
    rows.iter()
        .find(|row| is_occupied(row) && row.name.as_bytes() == name)
}
