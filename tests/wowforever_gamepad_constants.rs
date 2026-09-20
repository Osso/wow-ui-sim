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

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_gamepad_cvar_defaults_drive_vendor_mapping_and_events() {
    let env = WowLuaEnv::new().unwrap();
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    env.exec(
        &std::fs::read_to_string(cache.join("Blizzard_GamepadActionBars/ActionBar.lua")).unwrap(),
    )
    .unwrap();
    env.exec(
        &std::fs::read_to_string(
            cache.join("Blizzard_GamepadActionBars/GamepadOverrideBarMixin.lua"),
        )
        .unwrap(),
    )
    .unwrap();
    env.exec(r#"
        for _, entry in ipairs({
            {"GamepadPossessBarOverride", "1", "GAMEPAD_POSSESS_BAR_OVERRIDE_CHANGED"},
            {"GamepadStanceBarOverride", "3", "GAMEPAD_STANCE_BAR_OVERRIDE_CHANGED"},
        }) do
            local name, default, event = unpack(entry)
            assert(GetCVar(name) == default, name .. " current")
            assert(C_CVar.GetCVarDefault(name) == default, name .. " default")
            local anchor = CreateFrame("Frame")
            local alternate = CreateFrame("Frame")
            local bar = CreateFrame("Frame")
            Mixin(bar, GamepadOverrideBarMixin)
            bar.overrideCVar = name
            bar.overrideCVarChangedEvent = event
            bar.overrideMap = {}
            bar.isOverrideBarActive = false
            bar.pagingUnitOwner = anchor
            bar:SetOverrideMapping(tonumber(default), function(owner) assert(owner == anchor); return anchor end, 1)
            bar:SetOverrideMapping(2, function() return alternate end, 2)
            bar:ApplyInitialOverridePositioning()
            assert(bar:GetParent() == anchor)
            local events = {}
            local listener = CreateFrame("Frame")
            listener:RegisterEvent(event)
            listener:RegisterEvent("CVAR_UPDATE")
            listener:SetScript("OnEvent", function(_, received, a, b)
                events[#events + 1] = {received, a, b}
                if received == event then
                    assert(a == tonumber(default) and b == 2)
                    bar:OnOverrideCVarChanged(a, b)
                end
            end)
            assert(C_CVar.SetCVar(name, "2"))
            assert(GetCVar(string.lower(name)) == "2")
            assert(GetCVarDefault(name) == default)
            assert(bar:GetParent() == alternate)
            local updates, overrides = 0, 0
            for _, delivered in ipairs(events) do
                if delivered[1] == event then overrides = overrides + 1 end
                if delivered[1] == "CVAR_UPDATE" then
                    assert(delivered[2] == name and delivered[3] == "2")
                    updates = updates + 1
                end
            end
            assert(updates == 1 and overrides == 1)
            listener:UnregisterAllEvents()
        end
    "#).unwrap();
}

#[test]
fn forever_gamepad_cvar_defaults_are_profile_scoped() {
    let env = WowLuaEnv::new().unwrap();
    let published: bool = env.eval("return GetCVarDefault('GamepadPossessBarOverride') ~= nil and GetCVarDefault('GamepadStanceBarOverride') ~= nil").unwrap();
    assert_eq!(published, cfg!(feature = "client-wowforever"));
}
