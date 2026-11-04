//! Original artifact-title hover reads the viewed artifact's relic slot.
#![cfg(any(feature = "profile-retail", feature = "client-ptr", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::state::RelicSlotInfo;

#[test]
fn tooltip_socketed_relic_original_hover_selects_viewed_slot() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let toc = wow_ui_sim::loader::find_toc_file(&addons.join("Blizzard_ArtifactUI")).unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    for item_id in [1728, 127830] {
        env.state().borrow_mut().viewed_artifact.relic_slots.push(RelicSlotInfo {
            link: format!("item:{item_id}"),
            ..RelicSlotInfo::default()
        });
    }
    env.exec(
        r#"
        assert(ArtifactTitleTemplateMixin and ArtifactPerksMixin, 'original artifact consumers must load')
        local perks = CreateFrame('Frame', nil, UIParent)
        Mixin(perks, ArtifactPerksMixin)
        local title = CreateFrame('Frame', nil, perks)
        Mixin(title, ArtifactTitleTemplateMixin)
        local relic = CreateFrame('Frame', nil, title)
        for index, itemID in ipairs({1728, 127830}) do
            relic.relicSlotIndex = index
            relic.relicLink = C_ArtifactUI.GetRelicInfo(index) and select(4, C_ArtifactUI.GetRelicInfo(index))
            title:OnRelicSlotMouseEnter(relic)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetSocketedRelic', 'socketed-relic accessor missing')
            assert(data and data.id == itemID, 'must use viewed slot, not equipped or constant item')
            local expected = C_TooltipInfo.GetItemByID(itemID)
            assert(#data.lines > 0 and #data.lines == #expected.lines)
            for lineIndex, line in ipairs(expected.lines) do
                assert(data.lines[lineIndex].leftText == line.leftText, 'must preserve generated item lines')
            end
            assert(GameTooltip:GetLeftLine(1):GetText() == expected.lines[1].leftText)
        end
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetSocketedRelic(99)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'missing slot must not select the first relic')
        "#,
    ).unwrap();
}
