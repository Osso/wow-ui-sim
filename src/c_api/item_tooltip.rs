//! R80: host-supplied ordinary item stat amounts and producer metadata; R88aw: bag-item status snapshots.
//! Native item-mod amount calculation and calculated SellPrice values remain UNKNOWN.

#[derive(Clone, Debug)]
pub struct ItemTooltipStat {
    pub label: String,
    pub amount: i32,
}

/// Inputs for a catalog item at one resolved working level. This is not a native
/// DB2 mapping. Hosts must not reuse a row for unrepresented instance variants.
#[derive(Clone, Debug, Default)]
pub struct ItemTooltipInput {
    pub stats: Vec<ItemTooltipStat>,
    pub guid: Option<String>,
    pub hyperlink: Option<String>,
    pub override_item_level: Option<u32>,
    pub is_corrupted_item: Option<bool>,
    pub is_azerite_empowered_item: Option<bool>,
    pub is_azerite_item: Option<bool>,
}

/// Explicit status inputs for the Retail bag-item SellPrice gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SellPriceStatusInput {
    StatusRecord {
        flags: u32,
        recorded_time: i64,
        client_clock: i64,
    },
    NoStatusRecord {
        instance_marker_flags: u32,
    },
}
