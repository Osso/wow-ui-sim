use super::*;

use tempfile::tempdir;
use wow_ui_sim::loader::load_addon_with_saved_vars;
use wow_ui_sim::saved_variables::SavedVariablesManager;

prefork_full_ui_case! {
fn startup_player_life_bar_matches_player_health(env: &WowLuaEnv) {
        let result: (
            Option<f64>,
            Option<f64>,
            String,
            bool,
            String,
            i32,
            i32,
        ) = env
            .eval(
                r#"
                local healthBar = PlayerFrame_GetHealthBar and PlayerFrame_GetHealthBar()
                local playerFrameState = PlayerFrame and PlayerFrame.state or "nil"
                local vehicleUi = type(UnitHasVehiclePlayerFrameUI) == "function"
                    and UnitHasVehiclePlayerFrameUI("player")
                    or false
                if not healthBar then
                    return nil, nil, playerFrameState, vehicleUi, "nil", UnitHealth("player"), UnitHealthMax("player")
                end
                local _, maxValue = healthBar:GetMinMaxValues()
                return healthBar:GetValue(), maxValue, playerFrameState, vehicleUi, tostring(healthBar.unit), UnitHealth("player"), UnitHealthMax("player")
                "#,
            )
            .expect("player health bar probe should run");

        let (
            bar_value,
            bar_max,
            player_frame_state,
            vehicle_ui,
            bar_unit,
            current_health,
            max_health,
        ) = result;
        assert!(
            current_health > 0,
            "player health should be initialized at startup"
        );
        assert!(
            max_health > 0,
            "player health max should be initialized at startup"
        );
        assert_eq!(
            bar_max,
            Some(max_health as f64),
            "player health bar max should match player max health"
        );
        assert_eq!(
            player_frame_state,
            "player",
            "player frame should stay on player art at startup"
        );
        assert!(
            !vehicle_ui,
            "vehicle player-frame UI should be disabled in the simulator startup surface"
        );
        assert_eq!(
            bar_unit,
            "player",
            "player health bar should stay bound to the player unit"
        );
        assert_eq!(
            bar_value,
            Some(current_health as f64),
            "player health bar should reflect current player health"
        );
    }
}

prefork_full_ui_case! {
fn startup_player_buffs_show_duration_text(env: &WowLuaEnv) {
        let result: (i32, i32, Option<String>) = env
            .eval(
                r#"
                if not BuffFrame or not BuffFrame.auraFrames then
                    return 0, 0, nil
                end

                local visible_buffs = 0
                local visible_durations = 0
                local first_duration = nil

                for _, button in ipairs(BuffFrame.auraFrames) do
                    if button:IsShown()
                        and button.buttonInfo
                        and button.buttonInfo.auraType == "Buff"
                        and button.buttonInfo.expirationTime
                        and button.buttonInfo.expirationTime > 0
                    then
                        visible_buffs = visible_buffs + 1
                        if button.Duration and button.Duration:IsShown() then
                            visible_durations = visible_durations + 1
                            if not first_duration and button.Duration:GetText() then
                                first_duration = button.Duration:GetText()
                            end
                        end
                    end
                end

                return visible_buffs, visible_durations, first_duration
                "#,
            )
            .expect("buff duration probe should run");

        let (visible_buffs, visible_durations, first_duration) = result;
        assert!(
            visible_buffs > 0,
            "startup should expose at least one visible player buff with a duration"
        );
        assert_eq!(
            visible_buffs,
            visible_durations,
            "visible player buffs with durations should render their duration labels"
        );
        assert!(
            first_duration.is_some(),
            "at least one visible buff duration should have text"
        );
    }
}

prefork_full_ui_case! {
fn startup_keeps_action_bar_deprecation_fallbacks_non_recursive(env: &WowLuaEnv) {
        let result: (bool, bool, bool, bool) = env
            .eval(
                r#"
                local texture_ok, texture = pcall(C_ActionBar.GetActionTexture, 13)
                local has_action_ok, has_action = pcall(C_ActionBar.HasAction, 13)
                return texture_ok, texture == nil, has_action_ok, has_action == false
            "#,
            )
            .expect("C_ActionBar probes should return values");

        assert_eq!(
            result,
            (true, true, true, true),
            "Deprecated action-bar fallbacks should not recurse through C_ActionBar"
        );
    }
}

#[test]
fn c_action_bar_reads_configured_default_bar_indices() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        let result: (
            i32,
            i32,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            i32,
            i32,
            i32,
        ) = env
            .eval(
                r#"
                return C_ActionBar.GetCurrentActionBarByClass(),
                       C_ActionBar.GetExtraBarIndex(),
                       C_ActionBar.GetVehicleBarIndex(),
                       C_ActionBar.GetOverrideBarIndex(),
                       C_ActionBar.GetTempShapeshiftBarIndex(),
                       C_ActionBar.GetMultiCastBarIndex(),
                       C_ActionBar.GetBonusBarIndex(),
                       C_ActionBar.GetBonusBarOffset()
            "#,
            )
            .expect("C_ActionBar default bar indices should evaluate");

        assert_eq!(
            result,
            (1, 13, Some(12), Some(14), Some(1), 7, 0, 0),
            "Inactive special bars still expose the simulator's configured indices"
        );
    }
}

#[test]
fn blizzard_console_saved_variables_machine_seed_without_saved_vars_manager() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);

        let toc_path = blizzard_ui_dir().join("Blizzard_Console/Blizzard_Console.toc");
        load_addon(&env.loader_env(), &toc_path)
            .expect("Blizzard_Console should load without a saved vars manager");

        let saved_vars_type: String = env
            .eval("return type(Blizzard_Console_SavedVars)")
            .expect("saved vars probe should run");

        assert_eq!(
            saved_vars_type, "table",
            "SavedVariablesMachine globals should still be seeded when persistence is disabled"
        );
    }
}

#[test]
fn damage_meter_saved_variables_default_without_partial_empty_seed() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);

        let edit_mode_toc = blizzard_ui_dir().join("Blizzard_EditMode/Blizzard_EditMode.toc");
        load_addon(&env.loader_env(), &edit_mode_toc).expect("Blizzard_EditMode should load");

        let toc_path = blizzard_ui_dir().join("Blizzard_DamageMeter/Blizzard_DamageMeter.toc");
        load_addon(&env.loader_env(), &toc_path)
            .expect("Blizzard_DamageMeter should load without a saved vars manager");

        let (saved_vars_type, window_data_list_type) = damage_meter_saved_vars_shape(&env);

        assert!(
            saved_vars_type == "nil" || window_data_list_type == "table",
            "DamageMeter saved vars should stay nil or expose windowDataList, not a partially-seeded table"
        );
    }
}

#[test]
fn damage_meter_saved_variables_default_with_empty_saved_vars_storage() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);
        let temp = tempdir().expect("tempdir");
        let mut saved_vars = SavedVariablesManager::with_storage_dir(temp.path().to_path_buf());

        let edit_mode_toc = blizzard_ui_dir().join("Blizzard_EditMode/Blizzard_EditMode.toc");
        load_addon_with_saved_vars(&env.loader_env(), &edit_mode_toc, &mut saved_vars)
            .expect("Blizzard_EditMode should load with an empty saved vars manager");

        let toc_path = blizzard_ui_dir().join("Blizzard_DamageMeter/Blizzard_DamageMeter.toc");
        load_addon_with_saved_vars(&env.loader_env(), &toc_path, &mut saved_vars)
            .expect("Blizzard_DamageMeter should load with an empty saved vars manager");

        let (saved_vars_type, window_data_list_type) = damage_meter_saved_vars_shape(&env);

        assert!(
            saved_vars_type == "nil" || window_data_list_type == "table",
            "DamageMeter saved vars should stay nil or expose windowDataList, not a partially-seeded table"
        );
    }
}

prefork_full_ui_case! {
fn startup_legacy_dropdown_check_regions_remain_textures(env: &WowLuaEnv) {
        let result: String = env
            .eval(
                r##"
                local cases = {
                    { "DropDownList1Button1", DropDownList1Button1, DropDownList1Button1Check },
                    { "DropDownList1Button2", DropDownList1Button2, DropDownList1Button2Check },
                    { "DropDownList2Button1", DropDownList2Button1, DropDownList2Button1Check },
                    { "DropDownList2Button2", DropDownList2Button2, DropDownList2Button2Check },
                }
                for _, case in ipairs(cases) do
                    local name, button, region = case[1], case[2], case[3]
                    if region:GetObjectType() ~= "Texture" then
                        return name .. "_type=" .. tostring(region:GetObjectType())
                    end
                    if button.CheckButton ~= nil then
                        return name .. "_checkbutton=" .. tostring(button.CheckButton:GetObjectType())
                    end
                end
                return "ok"
                "##,
            )
            .expect("legacy dropdown check-region probe should run");

        assert_eq!(
            result, "ok",
            "legacy dropdown $parentCheck regions are textures, not CheckButton parent-key children"
        );
    }
}

prefork_full_ui_case! {
fn startup_chat_config_dynamic_wide_checkboxes_keep_checkbutton_parent_key(env: &WowLuaEnv) {
        let result: (bool, bool, bool, bool, bool) = env
            .eval(
                r##"
                local ok, err = pcall(function()
                    CreateFrame("CheckButton", "StartupWideCheckboxProbe", UIParent, "MovableChatConfigWideCheckboxWithSwatchTemplate")
                end)
                local frame = StartupWideCheckboxProbe
                local check = StartupWideCheckboxProbeCheck
                return
                    ok,
                    type(err) == "nil",
                    frame ~= nil,
                    check ~= nil,
                    frame ~= nil and frame.CheckButton == check
                "##,
            )
            .expect("dynamic chat checkbox probe should run");

        assert_eq!(
            result,
            (true, true, true, true, true),
            "Dynamic chat checkboxes should keep their CheckButton child wired to the parent frame"
        );
    }
}

prefork_full_ui_case! {
fn chat_config_create_checkboxes_does_not_emit_checkbutton_error(env: &WowLuaEnv) {
        let before = env.state().borrow().lua_errors.len();
        env.exec(
            r##"
            ChatConfig_CreateCheckboxes(ChatConfigChatSettingsLeft, CHAT_CONFIG_CHAT_LEFT, "ChatConfigWideCheckboxWithSwatchTemplate", PLAYER_MESSAGES)
            "##,
        )
        .expect("chat checkbox creation should succeed");
        let after = env.state().borrow().lua_errors.clone();
        let targeted: Vec<String> = after
            .into_iter()
            .skip(before)
            .filter(|message| message.contains("CheckButton"))
            .collect();

        assert!(
            targeted.is_empty(),
            "ChatConfig_CreateCheckboxes should not emit CheckButton errors:
    {}",
            targeted.join("
    ")
        );
    }
}

#[test]
fn startup_chat_config_channel_checkbox_creation_keeps_checkbutton_children() {
    test_timeout! {
        let mut messages = Vec::new();
        let env = load_targeted_startup_env(&mut messages);
        env.exec(
            r##"
            local created = {}
            local originalCreateFrame = CreateFrame
            __chat_config_original_create_frame = originalCreateFrame
            CreateFrame = function(frameType, name, parent, template, ...)
                local frame = originalCreateFrame(frameType, name, parent, template, ...)
                local nameStr = tostring(name)
                local templateStr = tostring(template)
                if nameStr:find("Checkbox") or nameStr:find("Check") or templateStr:find("Checkbox") or templateStr:find("Check") then
                    created[#created + 1] = table.concat({
                        tostring(frameType),
                        nameStr,
                        templateStr,
                        tostring(frame ~= nil and frame.CheckButton ~= nil),
                        tostring(_G[nameStr .. "Check"] ~= nil),
                    }, "|")
                end
                return frame
            end
            __chat_config_created = created
            "##,
        )
        .expect("chat config CreateFrame wrapper should install");
        {
            let mut state = env.state().borrow_mut();
            state.lua_errors.clear();
            state.lua_error_records.clear();
            state.lua_error_counts.clear();
        }
        collect_targeted_startup_messages(&env, &mut messages);
        env.exec(
            r##"
            if __chat_config_original_create_frame ~= nil then
                CreateFrame = __chat_config_original_create_frame
            end
            "##,
        )
        .ok();
        let created: String = env
            .eval(
                r##"
                return table.concat(__chat_config_created or {}, "\n")
                "##,
            )
            .expect("created checkbox log should stringify");
        let targeted: Vec<String> = env
            .state()
            .borrow()
            .lua_errors
            .clone()
            .into_iter()
            .filter(|message| message.contains("CheckButton"))
            .collect();

        assert!(
            targeted.is_empty(),
            "Startup channel checkbox creation should not emit CheckButton errors.\ncreated:\n{created}\nerrors:\n  {}",
            targeted.join("\n  ")
        );
    }
}

#[test]
fn startup_expected_number_error_has_traceback() {
    test_timeout! {
        let env = load_and_startup_env();
        let targeted: Vec<String> = env
            .state()
            .borrow()
            .lua_error_records
            .iter()
            .filter(|record| record.message.contains("expected number, got nil at argument 1"))
            .map(|record| {
                let addon = record.addon_name.as_deref().unwrap_or("<none>");
                format!("[{addon}] {}", record.message)
            })
            .collect();

        assert!(
            targeted.is_empty(),
            "Startup should not report the numeric-argument regression:
    {}",
            targeted.join("
    ")
        );
    }
}

#[test]
fn startup_catalog_shop_numeric_error_after_load_clear() {
    test_timeout! {
        let mut messages = Vec::new();
        let env = load_targeted_startup_env(&mut messages);
        {
            let mut state = env.state().borrow_mut();
            state.lua_errors.clear();
            state.lua_error_records.clear();
            state.lua_error_counts.clear();
        }
        collect_targeted_startup_messages(&env, &mut messages);
        let targeted: Vec<String> = env
            .state()
            .borrow()
            .lua_errors
            .clone()
            .into_iter()
            .filter(|message| message.contains("expected number, got nil at argument 1"))
            .collect();
        let traced: Vec<String> = messages
            .into_iter()
            .filter(|message| message.contains("expected number, got nil at argument 1"))
            .collect();

        assert!(
            targeted.is_empty(),
            "CatalogShop numeric error should be absent after clearing load-time errors if it is load-only.\nstate errors:\n  {}\ntracebacks:\n  {}",
            targeted.join("\n  "),
            traced.join("\n  ")
        );
    }
}

#[test]
fn loading_blizzard_addons_does_not_emit_catalog_shop_numeric_error() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);

        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons(&ui);

        for (name, toc_path) in &addons {
            let before = env.state().borrow().lua_error_records.len();
            load_addon(&env.loader_env(), toc_path)
                .unwrap_or_else(|error| panic!("{name} should load: {error}"));
            let records = env.state().borrow().lua_error_records.clone();
            let targeted: Vec<String> = records
                .into_iter()
                .skip(before)
                .filter(|record| {
                    record.addon_name.as_deref() == Some("Blizzard_CatalogShop")
                        && record
                            .message
                            .contains("expected number, got nil at argument 1")
                })
                .map(|record| {
                    let addon = record.addon_name.unwrap_or_else(|| "<none>".to_string());
                    format!("[{addon}] {}", record.message)
                })
                .collect();

            assert!(
                targeted.is_empty(),
                "{name} load introduced the CatalogShop numeric error:\n  {}",
                targeted.join("\n  ")
            );
        }
    }
}

#[test]
fn apply_post_load_workarounds_does_not_introduce_catalog_shop_numeric_error() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);

        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons(&ui);
        for (_name, toc_path) in &addons {
            load_addon(&env.loader_env(), toc_path).expect("Failed to load Blizzard addon");
        }

        {
            let mut state = env.state().borrow_mut();
            state.lua_errors.clear();
            state.lua_error_records.clear();
            state.lua_error_counts.clear();
        }

        env.apply_post_load_workarounds();
        let targeted: Vec<String> = env
            .state()
            .borrow()
            .lua_errors
            .clone()
            .into_iter()
            .filter(|message| message.contains("expected number, got nil at argument 1"))
            .collect();

        assert!(
            targeted.is_empty(),
            "apply_post_load_workarounds should not introduce the CatalogShop numeric error:\n  {}",
            targeted.join("\n  ")
        );
    }
}

#[test]
fn startup_checkbutton_errors_report_addon_names() {
    test_timeout! {
        let env = load_and_startup_env();
        let mut targeted: Vec<String> = env
            .state()
            .borrow()
            .lua_error_records
            .iter()
            .filter(|record| record.message.contains("CheckButton"))
            .map(|record| {
                let addon = record.addon_name.as_deref().unwrap_or("<none>");
                format!("[{addon}] {}", record.message)
            })
            .collect();
        targeted.sort();
        targeted.dedup();

        assert!(
            targeted.is_empty(),
            "Startup should not report CheckButton regressions:
    {}",
            targeted.join("
    ")
        );
    }
}

#[test]
fn startup_targeted_errors_have_tracebacks() {
    test_timeout! {
        let mut messages = Vec::new();
        let env = load_with_early_error_collector(&mut messages);
        collect_targeted_startup_messages(&env, &mut messages);
        let targeted: Vec<String> = messages
            .into_iter()
            .filter(|message| {
                message.contains("expected number, got nil at argument 1")
                    || message.contains("CheckButton")
            })
            .collect();

        assert!(
            targeted.is_empty(),
            "Startup targeted errors should be gone:
    {}",
            targeted.join("
    ")
        );
    }
}

#[test]
fn load_time_targeted_errors_have_tracebacks() {
    test_timeout! {
        let mut messages = Vec::new();
        let _env = load_with_early_error_collector(&mut messages);
        let targeted: Vec<String> = messages
            .into_iter()
            .filter(|message| {
                message.contains("CheckButton")
                    || message.contains("expected number, got nil at argument 1")
                    || message.contains("expected number, got string at argument 1")
            })
            .collect();

        assert!(
            targeted.is_empty(),
            "Load-time targeted errors should be gone:
    {}",
            targeted.join("
    ")
        );
    }
}
