#![cfg(feature = "client-retail")]

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

#[test]
fn blocked_action_reaches_blizzard_with_taint_owner() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|err| panic!("load {name}: {err}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env.state().borrow_mut().lua_errors.clear();
    let (count, action, visible): (i32, String, bool) = env
        .eval(
            r#"
        assert(type(GameEvent.HandleActionBlocked) == "function")
        assert(type(AddonTooltip_ActionBlocked) == "function")
        local f = CreateFrame("Frame", "I13ProtectedAction", UIParent)
        local action
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("ADDON_ACTION_BLOCKED")
        listener:SetScript("OnEvent", function(_, _, _, name) action = name end)
        A_Admin.SetFrameProtected("I13ProtectedAction", true)
        A_Admin.SetInCombat(true)
        debug.setstacktaint("I13TestAddon")
        f:Hide()
        debug.setstacktaint(nil)
        return ADDON_ACTIONS_BLOCKED.I13TestAddon or 0, action or "", f:IsShown()
    "#,
        )
        .unwrap();
    assert_eq!(
        count, 1,
        "Blizzard must attribute the blocked action to the taint owner"
    );
    assert_eq!(action, "I13ProtectedAction:Hide()");
    assert!(visible);
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "{:?}",
        env.state().borrow().lua_errors
    );
}
