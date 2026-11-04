//! PTR 70493 declaration-backed Edit Mode consumer boundaries.
#![cfg(feature = "client-ptr")]

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addon_closure_for_screen_with_overrides, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn load_edit_mode() -> (WowLuaEnv, PathBuf) {
    let root = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&root);
    let addons = discover_blizzard_addon_closure_for_screen_with_overrides(
        &root,
        ScreenKind::Game,
        &["Blizzard_EditMode"],
        &[],
    );
    let (_, toc_path) = addons
        .iter()
        .find(|(name, _)| name == "Blizzard_EditMode")
        .unwrap();
    let toc = TocFile::from_file(toc_path).unwrap();
    for relative in [
        "Mainline/EditModePresetLayouts.lua",
        "Shared/EditModeSettingDisplayInfo.lua",
        "Shared/EditModePresetLayoutsManager.lua",
    ] {
        assert!(
            toc.files
                .iter()
                .any(|path| path == std::path::Path::new(relative)),
            "selected original {relative}"
        );
    }
    for (name, toc) in addons {
        let loaded = load_addon(&env.loader_env(), &toc).unwrap();
        if name == "Blizzard_EditMode" {
            for warning in loaded.warnings {
                eprintln!("original EditMode warning: {warning}");
            }
        }
    }
    (env, root.join("Blizzard_EditMode"))
}

#[test]
fn whole_mainline_preset_preserves_distinct_endcap_and_swing_timer_records() {
    let (env, root) = load_edit_mode();
    let source = std::fs::read_to_string(root.join("Mainline/EditModePresetLayouts.lua")).unwrap();
    env.exec_named(
        &source,
        "@Interface/AddOns/Blizzard_EditMode/Mainline/EditModePresetLayouts.lua",
    )
    .expect("complete unchanged original Mainline preset constructors");
    env.exec(
        r#"
        local modern = EDIT_MODE_MODERN_SYSTEM_MAP
        local classic = EDIT_MODE_CLASSIC_SYSTEM_MAP
        local gamepad = EDIT_MODE_GAMEPAD_SYSTEM_MAP
        assert(type(modern) == 'table' and type(classic) == 'table')
        assert(type(gamepad) == 'table' and modern ~= gamepad)
        local caps = modern[26]
        local left, right = caps[1], caps[2]
        assert(left ~= nil and right ~= nil and left ~= right)
        local count = 0
        for _ in pairs(caps) do count = count + 1 end
        assert(count == 2)
        assert(left.settings[0] == 0 and right.settings[0] == 0)
        assert(left.settings ~= right.settings)
        local a, b = left.anchorInfo, right.anchorInfo
        assert(a ~= b)
        assert(a.point == 'RIGHT' and a.relativeTo == 'MainActionBar')
        assert(a.relativePoint == 'LEFT' and a.offsetX == 30 and a.offsetY == 5)
        assert(b.point == 'LEFT' and b.relativeTo == 'BagsBar')
        assert(b.relativePoint == 'RIGHT' and b.offsetX == -30 and b.offsetY == 5)
        local indices = Enum.EditModeMainActionBarEndCapSystemIndices
        assert(indices.EndCapLeft == 1 and indices.EndCapRight == 2)
        local meta = Enum.EditModeMainActionBarEndCapSystemIndicesMeta
        assert(meta.NumValues == 2 and meta.MinValue == 1 and meta.MaxValue == 2)
        assert(classic[26] == caps and gamepad[26] == caps)
        local swingIndices = Enum.EditModeSwingTimerSystemIndices
        assert(swingIndices.MainHand == 1 and swingIndices.OffHand == 2)
        assert(swingIndices.Ranged == 3)
        local swingMeta = Enum.EditModeSwingTimerSystemIndicesMeta
        assert(swingMeta.NumValues == 3 and swingMeta.MinValue == 1)
        assert(swingMeta.MaxValue == 3)
        assert(classic[29] == modern[29] and gamepad[29] ~= modern[29])
        for _, preset in ipairs({modern, classic, gamepad}) do
            local bars = preset[29]
            local barCount = 0
            for _ in pairs(bars) do barCount = barCount + 1 end
            assert(barCount == 3)
            assert(bars[1] ~= bars[2] and bars[2] ~= bars[3] and bars[1] ~= bars[3])
            for index, offsetY in ipairs({450, 425, 400}) do
                local bar = bars[index]
                local settings, anchor = bar.settings, bar.anchorInfo
                assert(settings[0] == 5 and settings[1] == 50 and settings[2] == 0)
                assert(settings[5] == 1 and settings[6] == 1)
                local width = preset == gamepad and 120 or 213
                local height = preset == gamepad and 6 or 15
                assert(settings[3] == width and settings[4] == height)
                assert(anchor.point == 'BOTTOM' and anchor.relativeTo == 'UIParent')
                assert(anchor.relativePoint == 'BOTTOM' and anchor.offsetX == 0)
                assert(anchor.offsetY == offsetY)
            end
        end
    "#,
    )
    .unwrap();
}

#[test]
fn complete_display_info_initializes_original_system_maps() {
    let (env, root) = load_edit_mode();
    let source =
        std::fs::read_to_string(root.join("Shared/EditModeSettingDisplayInfo.lua")).unwrap();
    env.exec_named(
        &source,
        "@Interface/AddOns/Blizzard_EditMode/Shared/EditModeSettingDisplayInfo.lua",
    )
    .expect("complete unchanged original PTR display-info initialization");
    env.exec(
        r#"
        local manager = EditModeSettingDisplayInfoManager
        assert(type(manager.GetSystemSettingDisplayInfoMap) == 'function')
        assert(Enum.EditModeSystem.SwingTimer == 29)
        local swing = manager:GetSystemSettingDisplayInfoMap(29)
        assert(swing[0].setting == 0 and swing[0].minValue == 50)
        assert(swing[0].maxValue == 200 and swing[0].stepSize == 10)
        assert(swing[0]:ConvertValueForDisplay(3) == 80)
        assert(swing[1].minValue == 50 and swing[1].maxValue == 100)
        assert(swing[3].minValue == 213 and swing[3].maxValue == 852)
        assert(swing[4].minValue == 15 and swing[4].maxValue == 60)
        assert(swing[5].type == Enum.EditModeSettingDisplayType.Checkbox)
        assert(swing[6].type == Enum.EditModeSettingDisplayType.Checkbox)
        local visibility = swing[2].options
        assert(#visibility == 3)
        assert(visibility[1].value == 0 and visibility[2].value == 1)
        assert(visibility[3].value == 2)
        local loss = manager:GetSystemSettingDisplayInfoMap(28)
        assert(loss[0].minValue == 50 and loss[0].maxValue == 200)
        assert(loss[0].stepSize == 10)
        local group = manager:GetSystemSettingDisplayInfoMap(27)
        assert(group[0].minValue == 50 and group[0].maxValue == 150)
        assert(group[0].stepSize == 5)
        local cap = manager:GetSystemSettingDisplayInfoMap(26)
        assert(cap[0].type == Enum.EditModeSettingDisplayType.Checkbox)
        local gamepad = manager:GetSystemSettingDisplayInfoMap(30)
        for setting = 0, 4 do
            assert(gamepad[setting].setting == setting)
            assert(gamepad[setting].type == Enum.EditModeSettingDisplayType.Checkbox)
        end
        assert(manager:GetSystemSettingDisplayInfoMap(2) ~= nil)
        -- PTR 70493 EditModeManagerConstantsDocumentation.lua:797–828;
        -- original EditModeSettingDisplayInfo.lua:427–438 publishes this slider.
        local unitFrame = manager:GetSystemSettingDisplayInfoMap(3)
        assert(unitFrame[22], 'missing declared BuffIconSize slider at key 22')
        assert(unitFrame[22].setting == 22)
        assert(unitFrame[22].minValue == 50 and unitFrame[22].maxValue == 200)
        assert(unitFrame[22].stepSize == 10)
        assert(unitFrame[22]:ConvertValueForDisplay(3) == 80)
        assert(unitFrame[25] == nil, 'undeclared BuffIconSize key 25')
    "#,
    )
    .unwrap();
}

#[test]
fn loss_of_control_original_onload_completes_with_display_info_map() {
    let (env, edit_mode_root) = load_edit_mode();
    let root = edit_mode_root.parent().unwrap();
    let addons = discover_blizzard_addon_closure_for_screen_with_overrides(
        root,
        ScreenKind::Game,
        &["Blizzard_FrameXML"],
        &[],
    );
    let (_, toc_path) = addons
        .iter()
        .find(|(name, _)| name == "Blizzard_FrameXML")
        .unwrap();
    let toc = TocFile::from_file(toc_path).unwrap();
    for relative in ["LossOfControlFrame.lua", "LossOfControlFrame.xml"] {
        assert!(
            toc.files
                .iter()
                .any(|path| path == std::path::Path::new(relative)),
            "selected original {relative}"
        );
    }
    for (_, toc) in addons {
        load_addon(&env.loader_env(), &toc).unwrap();
    }
    env.exec(
        r#"
        local frame = LossOfControlFrame
        assert(frame.system == 28)
        local map = EditModeSettingDisplayInfoManager:GetSystemSettingDisplayInfoMap(28)
        assert(frame.settingDisplayInfoMap == map and map[0].maxValue == 200)
        assert(frame.Selection.system == frame)
        assert(frame:IsEventRegistered('CVAR_UPDATE'))
        assert(frame:IsEventRegistered('VARIABLES_LOADED'))
        assert(frame.TimeLeft.baseNumberWidth ==
            frame.TimeLeft.NumberText:GetStringWidth() + 6)
        assert(frame.TimeLeft.secondsWidth == frame.TimeLeft.SecondsText:GetStringWidth())
        assert(type(frame.Anim:GetScript('OnFinished')) == 'function')
    "#,
    )
    .expect("original LossOfControlFrame OnLoad completes past system map assignment");
}

#[test]
fn boss_growth_dropdown_consumes_three_declared_directions() {
    let (env, root) = load_edit_mode();
    // The complete selected file has other declaration gaps. Exercise its unchanged
    // dropdown constructor separately, without supplying any replacement enums.
    let source =
        std::fs::read_to_string(root.join("Shared/EditModeSettingDisplayInfo.lua")).unwrap();
    let start = source.find("\t\t-- Grow Direction").unwrap();
    let end = start + source[start..].find("\t\t-- ShowCastTime").unwrap();
    let padding = "\n".repeat(
        source[..start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count(),
    );
    env.exec_named(
        &format!(
            "{padding}ObservedBossGrowth = {{{}\n}}",
            &source[start..end]
        ),
        "@Interface/AddOns/Blizzard_EditMode/Shared/EditModeSettingDisplayInfo.lua",
    )
    .expect("original dropdown constructor at lines 250-261");
    env.exec(
        r#"
        local options = ObservedBossGrowth[1].options
        assert(#options == 3)
        for index, expected in ipairs({0, 1, 2}) do
            assert(type(options[index].value) == 'number')
            assert(options[index].value == expected, 'original direction option')
        end
        assert(Enum.BossFrameGrowDirection.Left == 0)
        assert(Enum.BossFrameGrowDirection.Center == 1)
        assert(Enum.BossFrameGrowDirection.Right == 2)
        local meta = Enum.BossFrameGrowDirectionMeta
        assert(meta.NumValues == 3 and meta.MinValue == 0 and meta.MaxValue == 2)
    "#,
    )
    .unwrap();
}

#[test]
fn swing_timer_constructor_consumes_seven_declared_settings() {
    let (env, root) = load_edit_mode();
    let source =
        std::fs::read_to_string(root.join("Shared/EditModeSettingDisplayInfo.lua")).unwrap();
    let system = source.find("\t[Enum.EditModeSystem.SwingTimer] =").unwrap();
    let start = system + source[system..].find("\t\t-- Scale").unwrap();
    let next = start
        + source[start..]
            .find("\t[Enum.EditModeSystem.VehicleSeatIndicator]")
            .unwrap();
    let end = start + source[start..next].rfind("\t},").unwrap();
    let padding = "\n".repeat(
        source[..start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count(),
    );
    env.exec_named(
        &format!(
            "{padding}ObservedSwingTimerSettings = {{{}\n}}",
            &source[start..end]
        ),
        "@Interface/AddOns/Blizzard_EditMode/Shared/EditModeSettingDisplayInfo.lua",
    )
    .expect("original SwingTimer setting constructor starting at line 800");
    env.exec(
        r#"
        local options = ObservedSwingTimerSettings
        assert(#options == 7)
        local expected = {0, 1, 3, 4, 5, 6, 2}
        for index, value in ipairs(expected) do
            assert(type(options[index].setting) == 'number')
            assert(options[index].setting == value, 'original SwingTimer setting option')
        end
        local visibilityOptions = options[7].options
        assert(#visibilityOptions == 3)
        for index, value in ipairs({0, 1, 2}) do
            assert(type(visibilityOptions[index].value) == 'number')
            assert(visibilityOptions[index].value == value, 'original visibility option')
        end
        local visibility = Enum.EditModeSwingTimerVisibility
        assert(visibility.Always == 0 and visibility.InCombat == 1 and visibility.Hidden == 2)
        local visibilityMeta = Enum.EditModeSwingTimerVisibilityMeta
        assert(visibilityMeta.NumValues == 3 and visibilityMeta.MinValue == 0)
        assert(visibilityMeta.MaxValue == 2)
        local settings = Enum.EditModeSwingTimerSetting
        assert(settings.Scale == 0 and settings.Opacity == 1)
        assert(settings.Visibility == 2 and settings.Width == 3)
        assert(settings.Height == 4 and settings.ShowBarTitle == 5 and settings.ShowTime == 6)
        local meta = Enum.EditModeSwingTimerSettingMeta
        assert(meta.NumValues == 7 and meta.MinValue == 0 and meta.MaxValue == 6)
    "#,
    )
    .unwrap();
}

#[test]
fn preset_manager_indexes_declared_gamepad_layout_separately() {
    let (env, root) = load_edit_mode();
    // Supplied maps isolate this separately reached manager from the earlier
    // Mainline preset's unrelated missing settings. No functions are replaced.
    env.exec(
        r#"
        EDIT_MODE_MODERN_SYSTEM_MAP = {}
        EDIT_MODE_CLASSIC_SYSTEM_MAP = {}
        EDIT_MODE_GAMEPAD_SYSTEM_MAP = {
            [Enum.EditModeSystem.Minimap] = { settings = {}, anchorInfo = { offsetX = 73 } }
        }
    "#,
    )
    .unwrap();
    let source =
        std::fs::read_to_string(root.join("Shared/EditModePresetLayoutsManager.lua")).unwrap();
    env.exec_named(
        &source,
        "@Interface/AddOns/Blizzard_EditMode/Shared/EditModePresetLayoutsManager.lua",
    )
    .expect("original manager Gamepad index at line 41");
    env.exec(
        r#"
        assert(Enum.EditModePresetLayouts.Modern == 0)
        assert(Enum.EditModePresetLayouts.Classic == 1)
        assert(Enum.EditModePresetLayouts.Gamepad == 2)
        local meta = Enum.EditModePresetLayoutsMeta
        assert(meta.NumValues == 3 and meta.MinValue == 0 and meta.MaxValue == 2)
        local manager = EditModePresetLayoutManager
        local layouts = manager:GetCopyOfPresetLayouts()
        assert(#layouts == 3 and layouts[3].layoutIndex == 2)
        assert(layouts[3].interfaceStyle == Enum.InputDeviceInterfaceType.Gamepad)
        local map = manager:GetPresetLayoutMapByIndex(2)
        assert(map == EDIT_MODE_GAMEPAD_SYSTEM_MAP)
        assert(manager:GetPresetLayoutMapByIndex(0) == EDIT_MODE_MODERN_SYSTEM_MAP)
        assert(manager:GetPresetLayoutMapByIndex(1) == EDIT_MODE_CLASSIC_SYSTEM_MAP)
        local anchor = manager:GetPresetLayoutSystemAnchorInfo(2, Enum.EditModeSystem.Minimap)
        assert(anchor.offsetX == 73)
        anchor.offsetX = -19
        assert(map[Enum.EditModeSystem.Minimap].anchorInfo.offsetX == 73)
    "#,
    )
    .unwrap();
}
