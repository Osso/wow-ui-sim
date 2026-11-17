use crate::common;

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::{
    fire_one_on_update_tick, fire_startup_events_for_screen, process_pending_timers,
};
use wow_ui_sim::toc::TocFile;

const TEST_ADDONS: &[&str] = &["Wowless", "WowlessData"];

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn addons_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/AddOns")
}

fn scan_game_addons() -> Vec<(String, PathBuf)> {
    let mut addons = Vec::new();
    let base_path = addons_dir();
    let Ok(entries) = std::fs::read_dir(base_path) else {
        return addons;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') || TEST_ADDONS.contains(&name) {
            continue;
        }
        let Some(toc_path) = find_toc_file(&path) else {
            continue;
        };
        let Ok(toc) = TocFile::from_file(&toc_path) else {
            continue;
        };
        if toc.allows_screen(ScreenKind::Game)
            && !toc.is_ptr_only()
            && !toc.is_game_type_restricted()
        {
            addons.push((name.to_string(), toc_path));
        }
    }

    wow_ui_sim::loader::sort_addons_by_dependencies(&mut addons);
    addons
}

fn load_game_screen() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![
            wow_ui_sim::client_profile::blizzard_ui_addons_dir(),
            PathBuf::from("./Interface/AddOns"),
        ];
    }
    wow_ui_sim::xml::register_intrinsic_templates();

    common::load_profile_framexml(&env);
    let ui = blizzard_ui_dir();
    let blizzard = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &blizzard {
        if let Err(err) = load_addon(&env.loader_env(), toc_path) {
            panic!("[load {name}] FAILED: {err}");
        }
    }

    let addons = scan_game_addons();
    for (name, toc_path) in &addons {
        if let Err(err) = load_addon(&env.loader_env(), toc_path) {
            panic!("[addon {name}] FAILED: {err}");
        }
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

fn new_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![
            wow_ui_sim::client_profile::blizzard_ui_addons_dir(),
            PathBuf::from("./Interface/AddOns"),
        ];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

const FIRST_PANEL_TAB_DIVERGENCE_LUA: &str = r#"
    local function first_divergence(frame_name, count)
        local frame = _G[frame_name]
        if not frame or type(frame.Tabs) ~= "table" then
            return nil
        end
        for i = 1, count do
            local tab = frame.Tabs[i]
            local expected = _G[frame_name .. "Tab" .. i]
            if expected and tab ~= expected then
                return table.concat({
                    frame_name,
                    tostring(i),
                    tab and tab:GetName() or "nil",
                    expected:GetName(),
                    tab and tab:GetParent() and tab:GetParent():GetName() or "nil",
                }, "|")
            end
        end
        return nil
    end

    return first_divergence("CharacterFrame", 3)
        or first_divergence("MerchantFrame", 2)
        or first_divergence("FriendsFrame", 4)
        or first_divergence("RaidParentFrame", 3)
        or first_divergence("PVEFrame", 5)
        or first_divergence("MailFrame", 2)
        or ""
"#;

fn first_panel_tab_divergence(env: &WowLuaEnv) -> Option<String> {
    let summary: String = env
        .eval(FIRST_PANEL_TAB_DIVERGENCE_LUA)
        .expect("eval first panel tab divergence");

    (!summary.is_empty()).then_some(summary)
}

const PANEL_TAB_ANCHOR_TRACE_LUA: &str = r#"
    __panel_tab_trace = {}
    __panel_tab_trace_current_addon = nil

    local original_panel_templates_anchor_tabs = PanelTemplates_AnchorTabs

    local function tracked_panel(frame_name)
        return frame_name == "CharacterFrame"
            or frame_name == "MerchantFrame"
            or frame_name == "FriendsFrame"
            or frame_name == "RaidParentFrame"
            or frame_name == "PVEFrame"
            or frame_name == "MailFrame"
    end

    local function tab_name(value)
        return value and value.GetName and value:GetName() or "nil"
    end

    function PanelTemplates_AnchorTabs(frame, numTabs)
        local frame_name = frame and frame.GetName and frame:GetName()
        if frame_name and tracked_panel(frame_name) then
            for i = 2, frame.numTabs or 0 do
                local last_tab = frame.Tabs and frame.Tabs[i - 1] or _G[frame_name .. "Tab" .. (i - 1)]
                local this_tab = frame.Tabs and frame.Tabs[i] or _G[frame_name .. "Tab" .. i]
                local expected_tab = _G[frame_name .. "Tab" .. i]
                if this_tab ~= expected_tab then
                    __panel_tab_trace[#__panel_tab_trace + 1] = table.concat({
                        tostring(__panel_tab_trace_current_addon),
                        frame_name,
                        tostring(i),
                        tab_name(last_tab),
                        tab_name(this_tab),
                        tab_name(expected_tab),
                        tostring(type(frame.Tabs) == "table" and #frame.Tabs or -1),
                    }, "|")
                end
            end
        end

        return original_panel_templates_anchor_tabs(frame, numTabs)
    end
"#;

fn install_panel_tab_anchor_trace(env: &WowLuaEnv) {
    env.exec(PANEL_TAB_ANCHOR_TRACE_LUA)
        .expect("install panel tab anchor trace");
}

#[test]
fn game_boot_has_no_unexpected_lua_errors() {
    test_timeout! {
        let env = new_game_env();
        let ui = blizzard_ui_dir();
        let blizzard = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
        for (name, toc_path) in &blizzard {
            if let Err(err) = load_addon(&env.loader_env(), toc_path) {
                panic!("[load {name}] FAILED: {err}");
            }
        }

        let addons = scan_game_addons();
        for (name, toc_path) in &addons {
            if let Err(err) = load_addon(&env.loader_env(), toc_path) {
                panic!("[addon {name}] FAILED: {err}");
            }
        }

        env.apply_post_load_workarounds();
        common::install_error_collector(&env, "__game_boot_errors");
        take_lua_errors(&env);
        fire_startup_events_for_screen(&env, ScreenKind::Game);
        let errors = env.state().borrow().lua_errors.clone();
        let traces = common::drain_string_table(&env, "__game_boot_errors");
        assert!(
            errors.is_empty(),
            "game boot still has lua errors: {errors:#?}\ntraces:\n{:#?}",
            traces
        );
    }
}

fn set_panel_tab_trace_addon(env: &WowLuaEnv, addon_name: &str) {
    env.exec(&format!("__panel_tab_trace_current_addon = {addon_name:?}"))
        .expect("set current panel tab trace addon");
}

fn take_lua_errors(env: &WowLuaEnv) -> Vec<String> {
    let mut state = env.state().borrow_mut();
    std::mem::take(&mut state.lua_errors)
}

fn assert_no_lua_errors_after_stage(env: &WowLuaEnv, stage: &str) {
    let errors = take_lua_errors(env);
    assert!(
        errors.is_empty(),
        "{stage} still has lua errors: {errors:#?}"
    );
}

#[test]
fn game_boot_lua_errors_pipeline_finishes() {
    test_timeout! {
        let env = load_game_screen();
        take_lua_errors(&env);
        env.apply_post_event_workarounds();
        assert_no_lua_errors_after_stage(&env, "game boot post-event workarounds");
        env.state().borrow_mut().widgets.rebuild_anchor_index();
        assert_no_lua_errors_after_stage(&env, "game boot anchor-index rebuild");
        process_pending_timers(&env);
        assert_no_lua_errors_after_stage(&env, "game boot process_pending_timers");
        fire_one_on_update_tick(&env);
        assert_no_lua_errors_after_stage(&env, "game boot on_update tick");
        let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(&*env.rilua());
        assert_no_lua_errors_after_stage(&env, "game boot hide runtime hidden frames");
    }
}

#[test]
fn hide_runtime_hidden_frames_hides_quest_info_free_floating_frames() {
    test_timeout! {
        let env = load_game_screen();
        let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(&*env.rilua());
        let required_money_shown: bool = env
            .eval("return QuestInfoRequiredMoneyFrame ~= nil and QuestInfoRequiredMoneyFrame:IsShown()")
            .expect("probe QuestInfoRequiredMoneyFrame shown state");
        let group_size_shown: bool = env
            .eval("return QuestInfoGroupSize ~= nil and QuestInfoGroupSize:IsShown()")
            .expect("probe QuestInfoGroupSize shown state");
        assert!(
            !required_money_shown,
            "QuestInfoRequiredMoneyFrame should be hidden after runtime hidden frame pass"
        );
        assert!(
            !group_size_shown,
            "QuestInfoGroupSize should be hidden after runtime hidden frame pass"
        );
    }
}

#[test]
fn panel_tabs_do_not_diverge_during_blizzard_load() {
    test_timeout! {
        let env = new_game_env();
        let ui = blizzard_ui_dir();
        let blizzard = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
        let mut after_uipanels_game = false;

        for (name, toc_path) in &blizzard {
            if let Err(err) = load_addon(&env.loader_env(), toc_path) {
                panic!("[load {name}] FAILED: {err}");
            }
            if name == "Blizzard_UIPanels_Game" {
                after_uipanels_game = true;
            }
            if !after_uipanels_game {
                continue;
            }
            if let Some(divergence) = first_panel_tab_divergence(&env) {
                panic!("panel tab divergence after loading {name}: {divergence}");
            }
        }
    }
}

#[test]
fn panel_tabs_are_stable_inside_anchor_tabs_during_blizzard_uipanels_load() {
    test_timeout! {
        let env = new_game_env();
        let ui = blizzard_ui_dir();
        let blizzard = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);

        for (name, toc_path) in &blizzard {
            if let Err(err) = load_addon(&env.loader_env(), toc_path) {
                panic!("[load {name}] FAILED: {err}");
            }
            if name == "Blizzard_SharedXML" {
                install_panel_tab_anchor_trace(&env);
            }
            if name != "Blizzard_UIPanels_Game" {
                continue;
            }

            let trace: String = env
                .eval(
                    r#"
                    return table.concat(__panel_tab_trace or {}, "\n")
                    "#,
                )
                .expect("collect panel tab anchor trace");

            assert!(
                trace.is_empty(),
                "PanelTemplates_AnchorTabs saw transient tab divergence:\n{trace}"
            );
            break;
        }
    }
}

#[test]
fn panel_tabs_are_stable_inside_anchor_tabs_during_full_blizzard_load() {
    test_timeout! {
        let env = new_game_env();
        let ui = blizzard_ui_dir();
        let blizzard = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);

        for (name, toc_path) in &blizzard {
            if name == "Blizzard_SharedXML" {
                load_addon(&env.loader_env(), toc_path)
                    .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
                install_panel_tab_anchor_trace(&env);
                continue;
            }

            set_panel_tab_trace_addon(&env, name);
            if let Err(err) = load_addon(&env.loader_env(), toc_path) {
                panic!("[load {name}] FAILED: {err}");
            }
        }

        let trace: String = env
            .eval(
                r#"
                return table.concat(__panel_tab_trace or {}, "\n")
                "#,
            )
            .expect("collect full panel tab anchor trace");

        assert!(
            trace.is_empty(),
            "PanelTemplates_AnchorTabs saw transient tab divergence:\n{trace}"
        );
    }
}
