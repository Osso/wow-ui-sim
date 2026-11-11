//! R47 CVar-backed experience-preset selection; native writer details pending.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::cvars::CVarStorage;

pub(crate) const CVAR: &str = "hasSelectedExperiencePreset";

pub(crate) fn has_selected(cvars: &CVarStorage) -> rilua::LuaResult<bool> {
    match cvars.get(CVAR) {
        Some(value) => integer_prefix_nonzero(&value),
        None => Ok(ACTIVE == ClientProfile::Ptr),
    }
}

/// Project the supplied ordinary decimal/hex numeric-prefix cases without
/// changing generic GetCVarBool semantics. Native overflow/locale behavior is
/// UNKNOWN; no conformance is claimed for those inputs.
fn integer_prefix_nonzero(value: &str) -> rilua::LuaResult<bool> {
    // R50 still owns these forms. This is an explicit simulator coverage error,
    // not a guessed native parse result or native error contract.
    if value.starts_with('+')
        || value.starts_with('-')
        || value.chars().next().is_some_and(char::is_whitespace)
        || (value.starts_with('0') && value.as_bytes().get(1).is_some_and(u8::is_ascii_digit))
    {
        return Err(rilua::runtime_error(
            "experience-preset CVar format awaits native R50 contract",
        ));
    }
    let (digits, radix) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .map(|digits| (digits, 16))
        .unwrap_or((value, 10));
    Ok(digits
        .chars()
        .map_while(|ch| ch.to_digit(radix))
        .any(|digit| digit != 0))
}
