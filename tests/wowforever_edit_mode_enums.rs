#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_source(env: &WowLuaEnv, path: &str) {
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    env.exec(&std::fs::read_to_string(root.join(path)).unwrap())
        .unwrap();
}

#[test]
fn forever_edit_mode_source_enum_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        documentedEnums = {}
        APIDocumentation = { AddDocumentationTable = function(_, doc)
            for _, entry in ipairs(doc.Tables or {}) do
                if entry.Type == "Enumeration" then documentedEnums[entry.Name] = entry end
            end
        end }
    "#,
    )
    .unwrap();
    load_source(
        &env,
        "Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua",
    );
    env.exec(
        r#"
        local missing = {}
        for name, doc in pairs(documentedEnums) do
            for _, field in ipairs(doc.Fields) do
                if not Enum[name] or Enum[name][field.Name] ~= field.EnumValue then
                    table.insert(missing, name .. "." .. field.Name .. "=" .. field.EnumValue)
                end
            end
        end
        for _, name in ipairs({"EditModeAccountSetting", "EditModeGroupFinderSetting",
            "EditModeLossOfControlSetting", "EditModeMainActionBarEndCapSetting",
            "EditModeMainActionBarEndCapSystemIndices", "EditModeMicroMenuSetting",
            "EditModeMinimapSetting", "EditModePresetLayouts", "EditModeRaidWarningSetting",
            "EditModeSwingTimerSetting", "EditModeSwingTimerSystemIndices",
            "EditModeSwingTimerVisibility", "EditModeSystem", "EditModeUnitFrameSetting",
            "RaidDispelOverlayType"}) do
            for _, key in ipairs({"MinValue", "MaxValue", "NumValues"}) do
                local meta = Enum[name .. "Meta"]
                if not meta or meta[key] ~= documentedEnums[name][key] then
                    table.insert(missing, name .. "Meta." .. key)
                end
            end
        end
        table.sort(missing)
        assert(#missing == 0, table.concat(missing, ", "))
    "#,
    )
    .unwrap();
}

#[test]
fn forever_edit_mode_preset_consumer() {
    let env = WowLuaEnv::new().unwrap();
    load_source(
        &env,
        "Blizzard_EditMode/Camelot/EditModePresetLayoutConstants.lua",
    );
    load_source(&env, "Blizzard_EditMode/Mainline/EditModePresetLayouts.lua");
    env.exec(
        r#"
        local timers = EDIT_MODE_MODERN_SYSTEM_MAP[Enum.EditModeSystem.SwingTimer]
        local mainHand = timers[Enum.EditModeSwingTimerSystemIndices.MainHand].settings
        assert(mainHand[Enum.EditModeSwingTimerSetting.Width] == 213)
        assert(mainHand[Enum.EditModeSwingTimerSetting.Visibility] == Enum.EditModeSwingTimerVisibility.Always)
        local caps = EDIT_MODE_MODERN_SYSTEM_MAP[Enum.EditModeSystem.MainActionBarEndCap]
        assert(caps[Enum.EditModeMainActionBarEndCapSystemIndices.EndCapLeft].settings[Enum.EditModeMainActionBarEndCapSetting.Hidden] == 0)
    "#,
    )
    .unwrap();
}

#[test]
fn forever_edit_mode_default_layout_consumer() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_EditMode.GetEditModeDefaultLayout() == 0)")
        .unwrap();
    load_source(&env, "Blizzard_SharedXMLBase/TableUtil.lua");
    load_source(
        &env,
        "Blizzard_EditMode/Camelot/EditModePresetLayoutConstants.lua",
    );
    load_source(&env, "Blizzard_EditMode/Mainline/EditModePresetLayouts.lua");
    load_source(
        &env,
        "Blizzard_EditMode/Shared/EditModePresetLayoutSystemUtils.lua",
    );
    load_source(
        &env,
        "Blizzard_EditMode/Shared/EditModePresetLayoutsManager.lua",
    );
    env.exec(r#"
        local system = Enum.EditModeSystem.SwingTimer
        local index = Enum.EditModeSwingTimerSystemIndices.MainHand
        local expected = EDIT_MODE_MODERN_SYSTEM_MAP[system][index].anchorInfo
        local actual = EditModePresetLayoutManager:GetDefaultSystemAnchorInfo(system, index)
        assert(actual ~= expected)
        assert(actual.point == expected.point and actual.relativePoint == expected.relativePoint)
        assert(actual.offsetX == expected.offsetX and actual.offsetY == expected.offsetY)
        actual.offsetX = 9999
        assert(EditModePresetLayoutManager:GetDefaultSystemAnchorInfo(system, index).offsetX == expected.offsetX)
    "#).unwrap();
    env.state().borrow_mut().set_input_interface_style(
        wow_ui_sim::c_api::c_input_interface_style::InputInterfaceStyle::Gamepad,
    );
    env.exec("assert(C_EditMode.GetEditModeDefaultLayout() == 0)")
        .unwrap();
}

#[test]
fn forever_edit_mode_display_consumer() {
    let env = WowLuaEnv::new().unwrap();
    load_source(&env, "Blizzard_SharedXMLBase/TableUtil.lua");
    load_source(
        &env,
        "Blizzard_EditMode/Shared/EditModeSettingDisplayInfo.lua",
    );
    env.exec(r#"
        local settings = EditModeSettingDisplayInfoManager.systemSettingDisplayInfo[Enum.EditModeSystem.SwingTimer]
        assert(settings[1].setting == Enum.EditModeSwingTimerSetting.Scale)
        assert(settings[1].minValue == 50 and settings[1].maxValue == 200)
        assert(settings[2].setting == Enum.EditModeSwingTimerSetting.Opacity)
        assert(settings[2].minValue == 50 and settings[2].maxValue == 100)
    "#).unwrap();
}
