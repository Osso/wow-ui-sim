//! Explicit renown reward inputs; the owning SimState map starts empty.
//! No reward records, faction eligibility, or Lua publication are inferred here.

/// Input row matching cached `MajorFactionRenownRewardInfo` declarations.
/// Optional booleans preserve absent, false, and true as distinct values.
#[derive(Clone, Debug, PartialEq)]
pub struct RenownRewardInfo {
    pub renown_reward_id: i64,
    pub ui_order: i32,
    pub is_account_unlock: bool,
    pub item_id: Option<i64>,
    pub spell_id: Option<i64>,
    pub mount_id: Option<i64>,
    pub transmog_id: Option<i64>,
    pub transmog_set_id: Option<i64>,
    pub title_mask_id: Option<i64>,
    pub transmog_illusion_source_id: Option<i64>,
    pub icon: Option<i64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub toast_description: Option<String>,
    /// Cached primary documentation declares `number`, not an enum type.
    pub reward_type: Option<i32>,
    pub is_collected: Option<bool>,
}
