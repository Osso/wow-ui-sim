//! Explicit active-brawl input; no default game record or query producer.

#[derive(Debug, Clone, PartialEq)]
pub struct PvpBrawlInfo {
    pub brawl_id: i64,
    pub name: String,
    pub short_description: String,
    pub long_description: String,
    pub can_queue: bool,
    pub min_level: i32,
    pub max_level: i32,
    pub groups_allowed: bool,
    pub cross_faction_allowed: bool,
    pub time_left_until_next_change: Option<f64>,
    pub brawl_type: i32,
    pub map_names: Vec<String>,
    pub includes_all_arenas: bool,
    pub min_item_level: f64,
    pub should_hide_reward_icon: bool,
}

/// Supplied special-event record; native integer fields remain integer inputs.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecialEventBrawlInfo {
    pub brawl_id: i64,
    pub name: String,
    pub short_description: String,
    pub long_description: String,
    pub can_queue: bool,
    pub min_level: i32,
    pub max_level: i32,
    pub groups_allowed: bool,
    pub cross_faction_allowed: bool,
    pub time_left_until_next_change: Option<i64>,
    pub brawl_type: i32,
    pub map_names: Vec<String>,
    pub includes_all_arenas: bool,
    pub min_item_level: i32,
    pub should_hide_reward_icon: bool,
}

impl From<SpecialEventBrawlInfo> for PvpBrawlInfo {
    fn from(record: SpecialEventBrawlInfo) -> Self {
        Self {
            brawl_id: record.brawl_id,
            name: record.name,
            short_description: record.short_description,
            long_description: record.long_description,
            can_queue: record.can_queue,
            min_level: record.min_level,
            max_level: record.max_level,
            groups_allowed: record.groups_allowed,
            cross_faction_allowed: record.cross_faction_allowed,
            time_left_until_next_change: record
                .time_left_until_next_change
                .map(|value| value as f64),
            brawl_type: record.brawl_type,
            map_names: record.map_names,
            includes_all_arenas: record.includes_all_arenas,
            min_item_level: f64::from(record.min_item_level),
            should_hide_reward_icon: record.should_hide_reward_icon,
        }
    }
}
