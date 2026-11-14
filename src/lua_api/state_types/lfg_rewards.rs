//! Host-supplied LFG tooltip reward snapshots.

#[derive(Clone, Debug)]
pub struct LfgTooltipReward {
    pub reward_id: u32,
    pub quantity: i32,
    pub is_currency: bool,
}

#[derive(Clone, Debug, Default)]
pub struct LfgTooltipRewardGroup {
    pub dungeon_id: i32,
    pub dungeon_type_id: i32,
    pub rewards: Vec<LfgTooltipReward>,
    pub shortage_rewards: [Vec<LfgTooltipReward>; 3],
}
