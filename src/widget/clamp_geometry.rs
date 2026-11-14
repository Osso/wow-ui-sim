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
