//! WM97 descriptor effects through the original Blizzard tooltip accessor.
#![cfg(any(feature = "profile-retail", feature = "client-ptr"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

#[test]
fn tooltip_item_descriptor_original_hyperlink_preserves_bonus_lists() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    // Pinned ItemBonus lists 591/592 add 4/5 levels. Repeated lists apply twice.
    for (bonuses, level) in [("1:591", 33), ("1:592", 34), ("2:591:591", 37)] {
        env.exec(&format!(
            r#"
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetHyperlink('item:1728:0:0:0:0:0:0:0:0:0:0:0:{bonuses}:0:0:0:0::0')
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetHyperlink', 'original accessor selected')
            assert(data and data.id == 1728, 'same catalog ID selected')
            assert(data.lines[2].leftText == 'Item Level {level}', 'bonus lists must affect item level')
            assert(GameTooltip:GetLeftLine(2):GetText() == 'Item Level {level}', 'original consumer renders resolved level')
            "#,
        ))
        .unwrap();
    }
}
