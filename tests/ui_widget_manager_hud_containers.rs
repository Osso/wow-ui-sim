//! Actual active-profile Retail/PTR HUD OnLoad consumers; no vendor overrides or stubs.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use crate::common;
use common::blizzard_addon_harness::{blizzard_ui_dir, load_blizzard_addon_closure_into_env};
use wow_ui_sim::c_api::ui_widget_manager::{
    IconAndTextRecord, WidgetId, WidgetSetEntry, WidgetSetRecord,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn icon_record(text: &str) -> IconAndTextRecord {
    IconAndTextRecord {
        state: 1, // Shown
        text: text.into(),
        tooltip: "Supplied HUD tooltip".into(),
        dynamic_tooltip: String::new(),
        tooltip_loc: 3,
        shift_text_type: 0,
        widget_size_setting: 160.0,
        texture_kit: String::new(),
        frame_texture_kit: String::new(),
        has_timer: false,
        order_index: 10.0,
        widget_tag: "supplied-hud".into(),
        in_anim_type: 0,
        out_anim_type: 0,
        widget_scale: 0,
        layout_direction: 1,
        model_scene_layer: 0,
        scripted_animation_effect_id: 0.0,
    }
}

fn seed_before_loading(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    let inputs = &mut state.ui_widget_inputs;
    // Supplied records at the native HUD set IDs.
    for (set_id, widget_id, text) in [
        (1.0, 760201.0, "Top supplied"),
        (2.0, 760202.0, "Minimap supplied"),
        (283.0, 760203.0, "Power supplied"),
        (240.0, 760204.0, "Objective supplied"),
    ] {
        inputs.sets.insert(
            WidgetId::from_number(set_id),
            WidgetSetRecord {
                layout_direction: 0,
                vertical_padding: 7.0,
                widgets: vec![WidgetSetEntry {
                    widget_id,
                    widget_type: 0, // IconAndText
                    unit_token: None,
                }],
            },
        );
        inputs
            .icon_and_text
            .insert(WidgetId::from_number(widget_id), icon_record(text));
    }
}

fn assert_selection_queries(env: &WowLuaEnv, expected: [i32; 4]) {
    let actual: (i32, i32, i32, i32) = env
        .eval(
            r#"
            return C_UIWidgetManager.GetTopCenterWidgetSetID(),
                C_UIWidgetManager.GetBelowMinimapWidgetSetID(),
                C_UIWidgetManager.GetPowerBarWidgetSetID(),
                C_UIWidgetManager.GetObjectiveTrackerWidgetSetID()
            "#,
        )
        .expect("HUD selection queries must return numbers before cached addon loading");
    assert_eq!(actual, (expected[0], expected[1], expected[2], expected[3]));
}

fn load_standard_containers(env: &WowLuaEnv) {
    // Query assertions precede even cache discovery: cache failures are prerequisites,
    // not evidence of a failing selector or OnLoad consumer.
    let ui_dir = blizzard_ui_dir();
    eprintln!(
        "[ui-widget-cache] profile={} path={}",
        wow_ui_sim::client_profile::ACTIVE.cache_subdir(),
        ui_dir.display()
    );
    for name in ["Blizzard_UIWidgets", "Blizzard_ObjectiveTracker"] {
        assert!(
            ui_dir.join(name).is_dir(),
            "cached-addon prerequisite missing: {}",
            ui_dir.join(name).display()
        );
    }
    // Same environment setup as the dependency-closure harness, after query proof.
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui_dir.clone()];
    // These globals are implicit prerequisites, not declared UIWidgets TOC deps.
    // Separate real-addon loading establishes them before selected HUD OnLoad.
    let prerequisites = load_blizzard_addon_closure_into_env(
        env,
        &ui_dir,
        &["Blizzard_AuraContainer", "Blizzard_UnitFrame"],
        &[],
    );
    assert!(
        prerequisites
            .iter()
            .any(|name| name == "Blizzard_UnitFrame")
    );
    env.exec("assert(EncounterBar and EncounterBar.Layout, 'real EncounterBar prerequisite')")
        .expect("cached EncounterBar must precede selected HUD loading");
    // AuraContainerUtil belongs to the secure environment, not ordinary globals.
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "real prerequisite consumers must load cleanly: {:?}",
        env.state().borrow().lua_errors
    );
    // Empty overrides: no patched manager, providers, templates, or OnLoad.
    let loaded = load_blizzard_addon_closure_into_env(
        env,
        &ui_dir,
        &[
            "Blizzard_SharedXMLGame",
            "Blizzard_Fonts_Shared",
            "Blizzard_FrameEffects",
            "Blizzard_UIWidgets",
            "Blizzard_ObjectiveTracker",
        ],
        &[],
    );
    for name in ["Blizzard_UIWidgets", "Blizzard_ObjectiveTracker"] {
        assert!(
            loaded.iter().any(|loaded_name| loaded_name == name),
            "cached dependency closure did not load {name}"
        );
    }
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "cached-load prerequisite errors (not selector RED): {:?}",
        env.state().borrow().lua_errors
    );
}

#[test]
fn cached_standard_hud_onload_consumes_preseeded_selection_and_records() {
    common::with_timeout(88, || {
        wow_ui_sim::xml::register_intrinsic_templates();
        let env = WowLuaEnv::new().expect("failed to create Lua environment");
        seed_before_loading(&env);
        load_standard_containers(&env);
        assert_selection_queries(&env, [1, 2, 283, 240]);
        env.exec(r#"
            local cases = {
                {'UIWidgetTopCenterContainerFrame', 1, 760201, 'Top supplied'},
                {'UIWidgetBelowMinimapContainerFrame', 2, 760202, 'Minimap supplied'},
                {'UIWidgetPowerBarContainerFrame', 283, 760203, 'Power supplied'},
                {'ObjectiveTrackerUIWidgetContainer', 240, 760204, 'Objective supplied'},
            }
            for _, case in ipairs(cases) do
                local container = assert(_G[case[1]], case[1] .. ' missing')
                assert(container:IsRegisteredForWidgetSet(case[2]), case[1] .. ' selected wrong set')
                local widget = assert(container.widgetFrames[case[3]], case[1] .. ' record not consumed')
                assert(widget.Text:GetText() == case[4], case[1] .. ' supplied text not consumed')
                for _, other in ipairs(cases) do
                    if other[3] ~= case[3] then
                        assert(container.widgetFrames[other[3]] == nil, 'cross-set consumer leakage')
                    end
                end
            end
        "#).unwrap();
        assert!(env.state().borrow().lua_errors.is_empty());
        // No manual RegisterForWidgetSet, OnLoad replay, update event or show.
        // No IsVisible assertion: power starts hidden; objective has alpha0.
    });
}

#[test]
fn cached_standard_hud_registers_native_set_ids_without_records() {
    common::with_timeout(88, || {
        wow_ui_sim::xml::register_intrinsic_templates();
        let env = WowLuaEnv::new().expect("failed to create Lua environment");
        // Preserve initialized state; no HUD records are supplied.
        load_standard_containers(&env);
        assert_selection_queries(&env, [1, 2, 283, 240]);
        env.exec(
            r#"
            local names = {
                {'UIWidgetTopCenterContainerFrame', 1},
                {'UIWidgetBelowMinimapContainerFrame', 2},
                {'UIWidgetPowerBarContainerFrame', 283},
                {'ObjectiveTrackerUIWidgetContainer', 240},
            }
            for _, item in ipairs(names) do
                local name, setID = item[1], item[2]
                local container = assert(_G[name], name .. ' missing')
                assert(container:IsRegisteredForWidgetSet(setID), name .. ' selected wrong set')
                assert(container.widgetFrames == nil or next(container.widgetFrames) == nil,
                    name .. ' consumed unsupplied records')
                assert(container.widgetPools:GetNumActive() == 0, name .. ' acquired unsupplied widgets')
                assert(container.horizontalRowContainerPool:GetNumActive() == 0, name .. ' acquired rows')
                assert(UIWidgetManager.registeredWidgetContainers[container],
                    name .. ' did not register HUD')
            end
        "#,
        )
        .unwrap();
        assert!(env.state().borrow().lua_errors.is_empty());
    });
}
