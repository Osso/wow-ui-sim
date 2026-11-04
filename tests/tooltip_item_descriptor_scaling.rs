//! WM97b level pipeline through the original Blizzard tooltip consumer.
#![cfg(any(feature = "profile-retail", feature = "client-ptr"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

#[test]
fn tooltip_item_descriptor_original_hyperlink_applies_player_level_curve() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    // Item 122245 selects player-level curve 956 (identity 1..80) and
    // ContentTuning 2051 (range-enabled bounds 1..29). The separated core input
    // is the local player level, so 7 stays 7 and 60 clamps to 29.
    for (player_level, level) in [(7, 7), (60, 29)] {
        env.exec(&format!(
            r#"
            A_Admin.SetPlayerLevel({player_level})
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetHyperlink('item:122245:0:0:0:0:0:0:0:0:0:0:0:0:0:0:0:0::0')
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetHyperlink', 'original accessor selected')
            assert(data and data.id == 122245, 'descriptor keeps its own item ID')
            assert(data.lines[2].leftText == 'Item Level {level}', 'player-level curve with ContentTuning bounds')
            assert(GameTooltip:GetLeftLine(2):GetText() == 'Item Level {level}', 'original consumer renders the resolved level')
            "#,
        ))
        .unwrap();
    }
}
