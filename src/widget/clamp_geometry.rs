//! Retail R88n.5r-q3 screen geometry, separate from arbitrary UIParent state.

use crate::client_profile::{ACTIVE, ClientProfile};

pub(crate) const fn uses_retail_clamp_producers() -> bool {
    matches!(ACTIVE, ClientProfile::Retail)
}

/// Reserved geometry target; widget IDs are allocated from 1. The native
/// shared coordinate root is false and guarded, so it never accepts a request.
/// This is not a Lua script object and is never published as a relative frame.
pub(crate) const COORDINATE_ROOT: usize = 0;
pub(crate) const COORDINATE_ROOT_PROPAGATION: bool = false;

// R88n.5r-q3.2 initialization of the read-only shared unit extents.
const HORIZONTAL_EXTENT: f32 = 1.0;
const VERTICAL_EXTENT: f32 = 0.8;
const UNIT_DENOMINATOR: f32 = 1024.0;
// INFERRED interim positivity fallback (#1181); native small-positive threshold
// remains UNKNOWN. Do not test or claim tiny-positive scale fidelity.
const ROUNDING_EFFECTIVE_SCALE_THRESHOLD: f32 = 0.0;

pub(crate) fn inset_to_storage(public: f32) -> f32 {
    if !uses_retail_clamp_producers() {
        return public;
    }
    let coefficient = (1.0 / UNIT_DENOMINATOR) / HORIZONTAL_EXTENT;
    (public * coefficient) * VERTICAL_EXTENT
}

pub(crate) fn inset_to_public(stored: f32) -> f32 {
    if !uses_retail_clamp_producers() {
        return stored;
    }
    let horizontal = HORIZONTAL_EXTENT * UNIT_DENOMINATOR;
    (horizontal * stored) / VERTICAL_EXTENT
}

/// The rest of the simulator stores anchors and dimensions in public units.
/// Enter and leave the same native unit space used by the inset methods.
pub(crate) fn round_retail_layout_value(
    public: f32,
    scale: f32,
    display_scale: f32,
    is_dimension: bool,
) -> f32 {
    if (is_dimension && public == 0.0)
        || !(scale > ROUNDING_EFFECTIVE_SCALE_THRESHOLD && display_scale > 0.0)
    {
        return public;
    }
    let stored = inset_to_storage(public);
    let horizontal = HORIZONTAL_EXTENT * UNIT_DENOMINATOR;
    let pixels = ((((horizontal * stored) / VERTICAL_EXTENT) * scale) / display_scale).round();
    let pixels = if is_dimension && public > 0.0 {
        pixels.max(1.0)
    } else if is_dimension && public < 0.0 {
        pixels.min(-1.0)
    } else {
        pixels
    };
    let rounded = ((pixels * display_scale) / (horizontal * scale)) * VERTICAL_EXTENT;
    inset_to_public(rounded)
}
