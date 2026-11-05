//! Original WeeklyRewards item hover over the exact reward-entry key seam.
#![cfg(any(feature = "profile-retail", feature = "client-ptr"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

#[test]
fn tooltip_weekly_reward_original_hover_reads_stored_descriptor() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let toc = wow_ui_sim::loader::find_toc_file(&addons.join("Blizzard_WeeklyRewards")).unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    // WM97 DQ8: an exact 64-bit key selects an occupied, descriptor-bearing entry.
    // Large keys use the lossless 0x + 16 hex spelling; the pinned list 591 adds
    // four levels to catalog level 29.
    env.exec(
        r#"
        A_Admin.SetWeeklyRewardsReady(true)
        A_Admin.SetWeeklyRewardEntry('0x0020000000000001',
            'item:1728:0:0:0:0:0:0:0:0:0:0:0:1:591:0:0:0:0::0')
        A_Admin.SetWeeklyRewardEntry(42, 'item:127830:0:0:0:0:0:0:0:0:0:0:0:0:0:0:0:0::0')
        local button = CreateFrame('Frame', nil, UIParent)
        Mixin(button, WeeklyRewardActivityItemMixin)
        button.displayedItemDBID = '0x0020000000000001'
        button:OnEnter()
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetWeeklyReward', 'weekly-reward accessor missing')
        assert(data and data.id == 1728, 'large lossless key must select its own entry')
        assert(data.lines[2].leftText == 'Item Level 33', 'stored bonus list must reach the level line')
        assert(GameTooltip:GetLeftLine(2):GetText() == 'Item Level 33')
        button.displayedItemDBID = 42
        button:OnEnter()
        data = GameTooltip:GetPrimaryTooltipData()
        assert(data and data.id == 127830, 'numeric key selects the other entry')
        "#,
    )
    .unwrap();
    env.exec(
        r#"
        for _, key in ipairs({'0x0020000000000002', 0, 43}) do
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetWeeklyReward(key)
            assert(GameTooltip:GetPrimaryTooltipData() == nil, 'unknown or zero key must not select')
            assert(select('#', C_TooltipInfo.GetWeeklyReward(key)) == 0, 'EMULATED shared miss arity')
        end
        A_Admin.SetWeeklyRewardsReady(false)
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetWeeklyReward('0x0020000000000001')
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'not-ready state must not publish')
        "#,
    )
    .unwrap();
}
