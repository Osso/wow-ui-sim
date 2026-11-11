//! Behavior pin: EditMode HideBarArt drives retail main-bar end caps.

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::loader::BlizzardAddonOverride;
use wow_ui_sim::lua_api::WowLuaEnv;

const ROOT: &str = "Blizzard_ActionBar";
#[cfg(feature = "client-ptr")]
const STARTUP_OVERRIDES: &[BlizzardAddonOverride<'static>] = &[BlizzardAddonOverride {
    addon: ROOT,
    extra_roots: &["Blizzard_GamepadSharedUtility"],
}];
#[cfg(not(feature = "client-ptr"))]
const STARTUP_OVERRIDES: &[BlizzardAddonOverride<'static>] = &[];

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

fn show_bar_art(env: &WowLuaEnv) {
    set_hide_bar_art(env, false);
}

fn hide_bar_art(env: &WowLuaEnv) {
    set_hide_bar_art(env, true);
}

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

fn force_hide_endcaps(env: &WowLuaEnv) {
    env.eval::<()>("MainActionBar:UpdateEndCaps(true)")
        .expect("forced endcap hide must run cleanly");
}

fn assert_endcaps_visible(env: &WowLuaEnv) {
    let visible: bool = env
        .eval("return MainActionBar.EndCaps:IsShown()")
        .expect("visible endcap probe must run cleanly");
    assert!(visible, "main-bar end caps must be shown");
}

fn assert_endcaps_hidden(env: &WowLuaEnv) {
    let hidden: bool = env
        .eval("return not MainActionBar.EndCaps:IsShown()")
        .expect("hidden endcap probe must run cleanly");
    assert!(hidden, "main-bar end caps must be hidden");
}

fn assert_alliance_endcap_atlases(env: &WowLuaEnv) {
    let (left_atlas, right_atlas): (String, String) = env
        .eval("return MainActionBar.EndCaps.LeftEndCap:GetAtlas(), MainActionBar.EndCaps.RightEndCap:GetAtlas()")
        .expect("endcap atlas probe must run cleanly");
    assert_eq!(left_atlas, "ui-hud-actionbar-gryphon-left");
    assert_eq!(right_atlas, "ui-hud-actionbar-gryphon-right");
}
