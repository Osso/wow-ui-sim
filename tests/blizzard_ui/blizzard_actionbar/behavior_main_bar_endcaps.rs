//! Behavior pin: EditMode HideBarArt drives retail main-bar end caps.

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
#[cfg(feature = "client-ptr")]
use wow_ui_sim::loader::BlizzardAddonOverride;
use wow_ui_sim::lua_api::WowLuaEnv;

const ROOT: &str = "Blizzard_ActionBar";
#[cfg(feature = "client-ptr")]
const STARTUP_OVERRIDES: &[BlizzardAddonOverride<'static>] = &[BlizzardAddonOverride {
    addon: ROOT,
    extra_roots: &["Blizzard_GamepadSharedUtility"],
}];
// Camelot/EditModePresetLayoutConstants.lua:11 disables manageEndCaps;
// EditModeSystemTemplates.lua:1014 therefore does not apply HideBarArt to them.
#[cfg(not(any(feature = "client-ptr", feature = "client-wowforever")))]
#[test]
fn main_bar_endcaps_follow_hide_bar_art_setting() {
    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        show_bar_art(env);
        assert_endcaps_visible(env);
        assert_alliance_endcap_atlases(env);

        hide_bar_art(env);
        assert_endcaps_hidden(env);

        show_bar_art(env);
        assert_endcaps_visible(env);

        force_hide_endcaps(env);
        assert_endcaps_hidden(env);
    });
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn main_bar_endcaps_follow_hide_bar_art_setting() {
    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], STARTUP_OVERRIDES, |env, _loaded| {
        show_bar_art(env);
        assert_endcaps_visible(env);
        assert_alliance_endcap_atlases(env);

        hide_bar_art(env);
        assert_endcaps_hidden(env);

        show_bar_art(env);
        assert_endcaps_visible(env);

        force_hide_endcaps(env);
        assert_endcaps_hidden(env);
    });
    }
}

// Camelot endcaps have independent Hidden settings, not parent HideBarArt.
// EditModeSystemTemplates.lua:3786-3802 applies the setting; the original
// Camelot/MainMenuBarEndCaps.lua:4-22 callbacks override Hidden in Edit Mode.
#[cfg(feature = "client-wowforever")]
#[test]
fn main_bar_endcaps_follow_camelot_hidden_setting_and_edit_mode() {
    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        assert_alliance_endcap_atlases(env);
        env.exec(r#"
            local caps = MainActionBar.EndCaps
            local hidden = Enum.EditModeMainActionBarEndCapSetting.Hidden
            for _, cap in ipairs({caps.LeftEndCap, caps.RightEndCap}) do
                for _, value in ipairs({0, 1, 0}) do
                    cap:UpdateSystemSettingValue(hidden, value)
                    assert(cap:IsShown() == (value == 0), "Hidden setting controls this endcap")
                end
                cap:UpdateSystemSettingValue(hidden, 1)
            end
            -- EditModeManager.lua:98-105,116-125 publishes this state before
            -- dispatching Enter/Exit; isolate the original endcap callbacks.
            EditModeManagerFrame.editModeActive = true
            EventRegistry:TriggerEvent("EditMode.Enter")
            assert(EditModeManagerFrame:IsEditModeActive())
            assert(caps.LeftEndCap:IsShown() and caps.RightEndCap:IsShown(), "Edit Mode shows hidden caps")
            EditModeManagerFrame.editModeActive = false
            EventRegistry:TriggerEvent("EditMode.Exit")
            assert(not caps.LeftEndCap:IsShown() and not caps.RightEndCap:IsShown(), "Exit restores hidden caps")
        "#).expect("original Camelot endcap setting and visibility callbacks");
    });
    }
}

#[cfg(not(feature = "client-wowforever"))]
fn show_bar_art(env: &WowLuaEnv) {
    set_hide_bar_art(env, false);
}

#[cfg(not(feature = "client-wowforever"))]
fn hide_bar_art(env: &WowLuaEnv) {
    set_hide_bar_art(env, true);
}

#[cfg(not(feature = "client-wowforever"))]
fn set_hide_bar_art(env: &WowLuaEnv, hide: bool) {
    let hide_value = i32::from(hide);
    env.exec(&format!(
        r#"
        MainActionBar:UpdateSystemSettingValue(Enum.EditModeActionBarSetting.HideBarArt, {hide_value})
        MainActionBar:RefreshBarArt(true)
        "#
    ))
    .expect("HideBarArt fixture must run cleanly");
}

#[cfg(not(feature = "client-wowforever"))]
fn force_hide_endcaps(env: &WowLuaEnv) {
    env.eval::<()>("MainActionBar:UpdateEndCaps(true)")
        .expect("forced endcap hide must run cleanly");
}

#[cfg(not(feature = "client-wowforever"))]
fn assert_endcaps_visible(env: &WowLuaEnv) {
    let visible: bool = env
        .eval("return MainActionBar.EndCaps:IsShown()")
        .expect("visible endcap probe must run cleanly");
    assert!(visible, "main-bar end caps must be shown");
}

#[cfg(not(feature = "client-wowforever"))]
fn assert_endcaps_hidden(env: &WowLuaEnv) {
    let hidden: bool = env
        .eval("return not MainActionBar.EndCaps:IsShown()")
        .expect("hidden endcap probe must run cleanly");
    assert!(hidden, "main-bar end caps must be hidden");
}

#[cfg(not(any(feature = "client-ptr", feature = "client-wowforever")))]
fn assert_alliance_endcap_atlases(env: &WowLuaEnv) {
    let (left_atlas, right_atlas): (String, String) = env
        .eval("return MainActionBar.EndCaps.LeftEndCap:GetAtlas(), MainActionBar.EndCaps.RightEndCap:GetAtlas()")
        .expect("endcap atlas probe must run cleanly");
    assert_eq!(left_atlas, "ui-hud-actionbar-gryphon-left");
    assert_eq!(right_atlas, "ui-hud-actionbar-gryphon-right");
}

// Forever ActionBar.toc:35-36 selects Camelot/MainMenuBarEndCaps.xml:8-12:
// the end cap is a Frame and its Texture owns the atlas, as on PTR.
#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
fn assert_alliance_endcap_atlases(env: &WowLuaEnv) {
    let (left_atlas, right_atlas): (String, String) = env
        .eval("return MainActionBar.EndCaps.LeftEndCap.Texture:GetAtlas(), MainActionBar.EndCaps.RightEndCap.Texture:GetAtlas()")
        .expect("endcap atlas probe must run cleanly");
    assert_eq!(left_atlas, "ui-hud-actionbar-gryphon-left");
    assert_eq!(right_atlas, "ui-hud-actionbar-gryphon-right");
}
