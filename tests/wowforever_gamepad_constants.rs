//! Forever's native constants consumed by the unmodified gamepad action bar.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
#[cfg(feature = "client-wowforever")]
fn action_bar_initializes_both_documented_button_groups() {
    let env = WowLuaEnv::new().unwrap();
    let source = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join("Blizzard_GamepadActionBars/ActionBar.lua");
    env.exec(&std::fs::read_to_string(source).unwrap()).unwrap();
    env.exec(r#"
        local bar = CreateFrame("Frame")
        bar.Left = {}; bar.Right = {}
        for _, group in ipairs({bar.Left, bar.Right}) do
            for i = 1, 4 do
                local button = CreateFrame("Button", nil, bar)
                button.HotKey = button:CreateFontString()
                button.ButtonIcon = button:CreateTexture()
                function button:SetButtonIndexOnActionBar(index) self.index = index end
                function button:SetActionBarParent(parent) self.actionBar = parent end
                group["ActionButton" .. i] = button
            end
        end
        GamepadActionBarMixin.InitActionButtons(bar)
        assert(#bar.actionButtons == 8)
        for i, button in ipairs(bar.actionButtons) do
            assert(button == (i <= 4 and bar.Left["ActionButton" .. i] or bar.Right["ActionButton" .. (i - 4)]))
            assert(button.index == i and button.actionBar == bar)
            assert(button:GetAttribute("showgrid") == 1 and button:IsShown())
            assert(not button.HotKey:IsShown())
            assert(button.ButtonIcon:GetWidth() == 20 and button.ButtonIcon:GetHeight() == 20)
        end
    "#).unwrap();
}

#[test]
fn gamepad_constants_are_profile_scoped() {
    let env = WowLuaEnv::new().unwrap();
    let published: bool = env
        .eval("return rawget(Constants, 'GamepadActionBarConstants') ~= nil")
        .unwrap();
    assert_eq!(published, cfg!(feature = "client-wowforever"));
}
