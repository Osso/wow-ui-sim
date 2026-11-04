//! PTR bank-reagent decision inputs (WM92, PTR 70668, not pinned 70493).

/// UNKNOWN universal character default. EMULATED game scenario: a present
/// shared object with its restrictive-rule flag clear. Fixtures supply this
/// explicitly; no bank-open, location, ownership or server inference is made.
pub(crate) fn unknown_character_default() -> Option<bool> {
    Some(false)
}
