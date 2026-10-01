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
