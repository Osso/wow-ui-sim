//! Supplied reward caches; their native refresh lifecycle is not modeled.

#[derive(Debug, Clone, Default)]
pub struct PvpRewardCache {
    pub honor: i64,
    pub experience: i64,
    pub items: Vec<PvpRewardItem>,
    pub currencies: Vec<PvpRewardCurrency>,
}

#[derive(Debug, Clone)]
pub struct PvpRewardItem {
    pub id: i64,
    pub name: String,
    pub texture: i64,
    pub quantity: i64,
}

#[derive(Debug, Clone)]
pub struct PvpRewardCurrency {
    pub id: i64,
    pub quantity: i64,
}

#[derive(Debug, Clone, Default)]
pub struct PvpRewardCaches {
    pub random_bg: PvpRewardCache,
    pub arena_skirmish: PvpRewardCache,
    pub random_epic_bg: PvpRewardCache,
}
