#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn tooltip_lifecycle_original_insert_frame_uses_default_spacing() {
    let env = WowLuaEnv::new().unwrap();
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let source =
        std::fs::read_to_string(addons.join("Blizzard_SharedXML/SharedTooltipTemplates.lua"))
            .unwrap();
    env.exec(&source).unwrap();
    env.exec(r#"
        Round = function(value) return math.floor(value + 0.5) end
        local tip = CreateFrame("GameTooltip", "LifecycleSpacingTip", UIParent)
        tip:AddLine("first")
        tip:AddLine("second")
        local height = tip:GetLeftLine(2):GetLineHeight()
        local embedded = CreateFrame("Frame", nil, UIParent)
        embedded:SetSize(10, height * 2 + 1)
        local before = tip:NumLines()
        local reserved = GameTooltip_InsertFrame(tip, embedded, 0)
        local h = Round(height)
        local expected = math.ceil(Round(height * 2 + 1) / (h + 2))
        assert(tip:NumLines() - before == expected, "default spacing must reserve correct lines")
        assert(reserved == expected * h + (expected - 1) * 2, "default spacing must contribute to reserved height")
        assert(select('#', tip:GetCustomLineSpacing()) == 0, "unset spacing returns no values")
        tip:SetCustomLineSpacing(0)
        assert(tip:GetCustomLineSpacing() == 0, "explicit zero spacing remains an override")
    "#).unwrap();
}
