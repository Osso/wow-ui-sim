//! Supplied current-area identity and AreaTable flags.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct AreaPvpInputs {
    pub current_area_id: Option<u32>,
    pub area_flags: HashMap<u32, u32>,
}
