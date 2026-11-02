//! Supplied world-state values for outdoor PvP timers.
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct OutdoorPvpWorldState {
    pub values: HashMap<i32, i64>,
}
