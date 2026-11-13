//! Host-supplied content-tracking lists.
//!
//! Empty lists mean that the simulator host supplied no tracked IDs; they do not
//! establish the native client's initial tracking state.

use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct ContentTrackingState {
    /// Parsed tracking type to ordered ID list. The vector preserves duplicates.
    pub tracked_ids_by_type: BTreeMap<u32, Vec<i32>>,
}
