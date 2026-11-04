#![cfg(feature = "gui")]

//! Integration tests for keybinding dispatch — targeting tests.
//!
//! Covers TargetFrame visibility, F1–F6 party/enemy targeting keybinds.

use crate::common;
#[path = "render_order_support.rs"]
mod render_order_support;

#[cfg(not(feature = "client-wrath"))]
use image::RgbaImage;
use std::path::PathBuf;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::global_frames;
#[cfg(not(feature = "client-wrath"))]
use wow_ui_sim::render::headless::render_to_image;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn fire_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for ev in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(ev);
    }
    common::fire_player_entering_world(env, true, false);
    let _ = env.fire_edit_mode_layouts_updated();
    for ev in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(ev);
    }
}

fn frame_is_shown(env: &WowLuaEnv, frame_name: &str) -> bool {
    let code = format!("return {frame_name} ~= nil and {frame_name}:IsShown() == true");
    env.eval::<bool>(&code).unwrap_or(false)
}

/// Check whether a global frame exists.
fn frame_exists(env: &WowLuaEnv, frame_name: &str) -> bool {
    let code = format!("return {frame_name} ~= nil");
    env.eval::<bool>(&code).unwrap_or(false)
}

fn install_test_error_handler(env: &WowLuaEnv) {
    common::install_error_collector(env, "__test_errors");
}

fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    common::drain_string_table(env, "__test_errors")
}

#[cfg(not(feature = "client-wrath"))]
fn sample_rect_pixel(image: &RgbaImage, rect: (f32, f32, f32, f32), u: f32, v: f32) -> [u8; 4] {
    let x = (rect.0 + rect.2 * u).round() as u32;
    let y = (rect.1 + rect.3 * v).round() as u32;
    image.get_pixel(x, y).0
}

#[cfg(not(feature = "client-wrath"))]
fn max_channel_diff(lhs: [u8; 4], rhs: [u8; 4]) -> u8 {
    (0..4)
        .map(|channel| lhs[channel].abs_diff(rhs[channel]))
        .max()
        .unwrap_or(0)
}

#[cfg(not(feature = "client-wrath"))]
fn target_portrait_rect(env: &WowLuaEnv, width: f32, height: f32) -> (f32, f32, f32, f32) {
    let state = env.state().borrow();
    let target_frame_id = state
        .widgets
        .get_id_by_name("TargetFrame")
        .expect("TargetFrame should exist");
    let target_container_id = state
        .widgets
        .get(target_frame_id)
        .and_then(|frame| frame.children_keys.get("TargetFrameContainer"))
        .copied()
        .expect("TargetFrameContainer should exist");
    let portrait_id = state
        .widgets
        .get(target_container_id)
        .and_then(|frame| frame.children_keys.get("Portrait"))
        .copied()
        .expect("TargetFrameContainer.Portrait should exist");
    let portrait = state
        .widgets
        .get(portrait_id)
        .expect("TargetFrameContainer.Portrait should remain registered");
    assert!(
        !portrait.mask_textures.is_empty(),
        "TargetFrameContainer.Portrait should use the retail CircleMask"
    );
    let rect = wow_ui_sim::iced_app::compute_frame_rect(&state.widgets, portrait_id, width, height);
    (rect.x, rect.y, rect.width, rect.height)
}

const BLIZZARD_ADDONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_Colors",
    "Blizzard_SharedXML",
    "Blizzard_SharedXMLGame",
    "Blizzard_UIPanelTemplates",
    "Blizzard_FrameXMLBase",
    "Blizzard_FrameEffects",
    "Blizzard_LoadLocale",
    "Blizzard_Fonts_Shared",
    "Blizzard_HelpPlate",
    "Blizzard_AccessibilityTemplates",
    "Blizzard_ObjectAPI",
    "Blizzard_UIParent",
    "Blizzard_TextStatusBar",
    "Blizzard_MoneyFrame",
    "Blizzard_POIButton",
    "Blizzard_Flyout",
    "Blizzard_StoreUI",
    "Blizzard_MicroMenu",
    "Blizzard_GameMenuEsc",
    "Blizzard_GameMenu",
    "Blizzard_EditMode",
    "Blizzard_GarrisonBase",
    "Blizzard_GameTooltip",
    "Blizzard_UIParentPanelManager",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    "Blizzard_FrameXML",
    "Blizzard_UIPanels_Game",
    "Blizzard_BuffFrame",
    "Blizzard_SpellDiminishUI",
    "Blizzard_ActionBar",
    "Blizzard_ActionBarController",
    "Blizzard_RaidWarning",
    "Blizzard_VisualAlerts",
    "Blizzard_PrivateAurasUI",
    "Blizzard_AuraContainer",
    "Blizzard_UnitFrame",
];

/// Create environment with the Blizzard targeting surface plus Blizzard_UnitFrame.
fn setup_env() -> common::LockedEnv {
    common::lock_env(|| {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);

        let ui = blizzard_ui_dir();
        {
            let mut state = env.state().borrow_mut();
            state.addon_base_paths = vec![ui.clone()];
        }

        if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
            // FrameXML.toc:77 loads TargetFrame.xml; its OnLoad at :651-654
            // initializes the target unit and registers PLAYER_TARGET_CHANGED.
            // TargetFrame.lua:94-101 implements the visibility tested below.
            common::load_profile_framexml(&env);
            // Wrath Bindings.xml:455-457 calls TargetNearestEnemy(), unlike the
            // simulator's synthetic enemy1 binding. Supply a selectable enemy.
            env.exec("A_Admin.SetEnemyPool({name='Targeting fixture enemy', level=11, class_index=1})")
                .expect("seed the native nearest-enemy binding fixture");
        } else {
            for name in BLIZZARD_ADDONS {
                if *name == "Blizzard_UnitFrame" {
                    common::load_blizzard_addon_bootstrap(&env, &ui, name, "Mainline/TargetFrame.lua");
                }
                common::load_required_blizzard_addon(&env, &ui, name);
                env.apply_runtime_addon_load_workarounds(name);
            }
        }
        env.apply_post_load_workarounds();
        fire_startup_events(&env);
        env.apply_post_event_workarounds();
        let _ = global_frames::hide_runtime_hidden_frames(&*env.rilua());
        env
    })
}

// ── Target frame visibility tests (full addon load including Blizzard_UnitFrame) ──

#[test]
fn target_frame_shown_after_targeting() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        assert!(
            frame_exists(&env, "TargetFrame"),
            "TargetFrame should exist after full addon load"
        );

        // TargetFrame starts hidden (hide_runtime_hidden_frames) or via startup;
        // ensure it's hidden before testing
        if frame_is_shown(&env, "TargetFrame") {
            env.exec("TargetFrame:Hide()").unwrap();
        }

        // F1 = target self → TargetFrame should show
        env.send_key_press("F1", None).expect("F1 keybind failed");
        let _ = drain_test_errors(&env); // non-fatal errors from TargetFrame:Update()
        assert!(
            frame_is_shown(&env, "TargetFrame"),
            "TargetFrame should be shown after targeting self with F1"
        );

        // ClearTarget() clears the target → TargetFrame should hide.
        env.exec("ClearTarget()")
            .expect("ClearTarget should clear the target");
        let _ = drain_test_errors(&env);
        assert!(
            !frame_is_shown(&env, "TargetFrame"),
            "TargetFrame should be hidden after clearing the target with ClearTarget()"
        );
    }
}

#[test]
fn target_frame_shown_for_enemy() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        if frame_is_shown(&env, "TargetFrame") {
            env.exec("TargetFrame:Hide()").unwrap();
        }

        // TAB = target nearest enemy → TargetFrame should show
        env.send_key_press("TAB", None).expect("TAB keybind failed");
        let _ = drain_test_errors(&env); // non-fatal errors from TargetFrame:Update()
        assert!(
            frame_is_shown(&env, "TargetFrame"),
            "TargetFrame should be shown after targeting enemy with TAB"
        );
    }
}

// Wrath FrameXML/TargetFrame.xml:227-239 declares a standalone 64x64 portrait
// without a mask. Mainline TargetFrame.xml:68 uses the CircleMask asserted here.
#[cfg(not(feature = "client-wrath"))]
#[test]
fn target_frame_portrait_corners_match_background_when_replaced_with_class_icon() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);
        env.exec("SetCVar(\"ReplaceOtherPlayerPortraits\", \"1\")")
            .expect("ReplaceOtherPlayerPortraits CVar should enable class icon portraits");

        env.send_key_press("F2", None).expect("F2 keybind failed");
        let _ = drain_test_errors(&env);
        assert!(
            frame_is_shown(&env, "TargetFrame"),
            "TargetFrame should be shown after targeting party1 with F2"
        );

        let width = 1600u32;
        let height = 1200u32;
        let portrait_rect = target_portrait_rect(&env, width as f32, height as f32);

        let mut visible_mgr =
            render_order_support::make_texture_manager();
        let visible_batch =
            render_order_support::build_screenshot_like_batch(&env, width, height, Some("TargetFrame"));
        let visible_render = render_to_image(&visible_batch, &mut visible_mgr, width, height, None);

        env.exec("TargetFrame.TargetFrameContainer.Portrait:Hide()")
            .expect("target portrait should hide");
        wow_ui_sim::startup::run_extra_update_ticks(&env, 1);

        let mut hidden_mgr =
            render_order_support::make_texture_manager();
        let hidden_batch =
            render_order_support::build_screenshot_like_batch(&env, width, height, Some("TargetFrame"));
        let hidden_render = render_to_image(&hidden_batch, &mut hidden_mgr, width, height, None);

        let center_visible = sample_rect_pixel(&visible_render, portrait_rect, 0.5, 0.5);
        let center_hidden = sample_rect_pixel(&hidden_render, portrait_rect, 0.5, 0.5);
        assert!(
            max_channel_diff(center_visible, center_hidden) >= 40,
            "hiding the portrait should materially change the portrait center: visible={center_visible:?} hidden={center_hidden:?}"
        );

        for (u, v, label) in [
            (0.08, 0.08, "top-left"),
            (0.92, 0.08, "top-right"),
            (0.08, 0.92, "bottom-left"),
            (0.92, 0.92, "bottom-right"),
        ] {
            let visible_corner = sample_rect_pixel(&visible_render, portrait_rect, u, v);
            let hidden_corner = sample_rect_pixel(&hidden_render, portrait_rect, u, v);
            assert!(
                max_channel_diff(visible_corner, hidden_corner) <= 18,
                "target portrait {label} corner should already match the background when masked: visible={visible_corner:?} hidden={hidden_corner:?}"
            );
        }
    }
}

// ── F2–F5 → TargetUnit('party1')–('party4') ─────────────────────────────

#[test]
fn keybind_f2_targets_party1() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);
        if frame_is_shown(&env, "TargetFrame") {
            env.exec("TargetFrame:Hide()").unwrap();
        }
        env.send_key_press("F2", None).expect("F2 keybind failed");
        let _ = drain_test_errors(&env);
        assert!(
            frame_is_shown(&env, "TargetFrame"),
            "TargetFrame should be shown after targeting party1 with F2"
        );
    }
}

#[test]
fn keybind_f3_targets_party2() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);
        if frame_is_shown(&env, "TargetFrame") {
            env.exec("TargetFrame:Hide()").unwrap();
        }
        env.send_key_press("F3", None).expect("F3 keybind failed");
        let _ = drain_test_errors(&env);
        assert!(
            frame_is_shown(&env, "TargetFrame"),
            "TargetFrame should be shown after targeting party2 with F3"
        );
    }
}

#[test]
fn keybind_f4_targets_party3() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);
        if frame_is_shown(&env, "TargetFrame") {
            env.exec("TargetFrame:Hide()").unwrap();
        }
        env.send_key_press("F4", None).expect("F4 keybind failed");
        let _ = drain_test_errors(&env);
        assert!(
            frame_is_shown(&env, "TargetFrame"),
            "TargetFrame should be shown after targeting party3 with F4"
        );
    }
}

#[test]
fn keybind_f5_targets_party4() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);
        if frame_is_shown(&env, "TargetFrame") {
            env.exec("TargetFrame:Hide()").unwrap();
        }
        env.send_key_press("F5", None).expect("F5 keybind failed");
        let _ = drain_test_errors(&env);
        assert!(
            frame_is_shown(&env, "TargetFrame"),
            "TargetFrame should be shown after targeting party4 with F5"
        );
    }
}

// ── F6 → TargetUnit('enemy1') ────────────────────────────────────────────

#[test]
fn keybind_f6_targets_enemy() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);
        if frame_is_shown(&env, "TargetFrame") {
            env.exec("TargetFrame:Hide()").unwrap();
        }
        env.send_key_press("F6", None).expect("F6 keybind failed");
        let _ = drain_test_errors(&env);
        assert!(
            frame_is_shown(&env, "TargetFrame"),
            "TargetFrame should be shown after targeting enemy with F6"
        );
    }
}

#[test]
fn unit_frame_onleave_fades_out_game_tooltip() {
    test_timeout! {
        #[cfg(feature = "client-wrath")]
        let env = common::action_bar_fixture::wrath_env();
        #[cfg(not(feature = "client-wrath"))]
        let env = setup_env();

        env.exec(
            r#"
            local frame = CreateFrame("Frame", "UnitFrameTooltipRegression", UIParent)
            frame.unit = "player"
            UnitFrame_OnEnter(frame)
            UnitFrame_OnLeave(frame)
        "#,
        )
        .expect("UnitFrame_OnEnter/OnLeave should succeed");

        let tooltip_visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
        assert!(
            !tooltip_visible,
            "GameTooltip should be hidden after UnitFrame_OnLeave"
        );

        // Wrath UnitFrame.lua:135-142 calls FadeOut without clearing these fields.
        if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
            return;
        }
        let has_owner: bool = env.eval("return GameTooltip:GetOwner() ~= nil").unwrap();
        assert!(
            !has_owner,
            "GameTooltip owner should be cleared after UnitFrame_OnLeave"
        );

        let update_tooltip_cleared: bool = env
            .eval("return UnitFrameTooltipRegression.UpdateTooltip == nil")
            .unwrap();
        assert!(
            update_tooltip_cleared,
            "UnitFrame_OnLeave should clear frame.UpdateTooltip"
        );
    }
}
