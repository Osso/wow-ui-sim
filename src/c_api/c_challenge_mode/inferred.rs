//! Simulator projections for opaque Keystone-removal cursor inputs.

use super::KeystoneLocation;
use crate::lua_api::state::CursorItemOrigin;

/// INFERRED: q3 does not establish the staged item's native stack count; model one item.
pub(super) fn inferred_keystone_cursor_stack_count() -> i32 {
    1
}

/// INFERRED: project a selected bag location to cursor origin. The native associated GUID
/// and cursor source mode remain unknown; equipment locations are not projected.
pub(super) fn inferred_selected_bag_cursor_origin(
    location: Option<KeystoneLocation>,
) -> Option<CursorItemOrigin> {
    match location {
        Some(KeystoneLocation::Bag(bag, slot)) => Some(CursorItemOrigin::Bag { bag, slot }),
        _ => None,
    }
}

/// INFERRED: interpret the simulator's textual GUID prefix, where `0-…` means category zero.
/// Native GUID category behavior does not establish this string representation.
pub(super) fn inferred_guid_category_is_nonzero(guid: &str) -> bool {
    guid.split_once('-')
        .is_some_and(|(category, _)| !category.is_empty() && category != "0")
}
