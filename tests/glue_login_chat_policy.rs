#![cfg(all(
    feature = "gui",
    any(feature = "client-ptr", feature = "client-wowforever")
))]
use crate::common;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

#[test]
fn glue_login_chat_policy_comes_from_original_glueparent_without_settling_fixtures() {
    test_timeout! {
        let env = common::lock_env(|| WowLuaEnv::new().unwrap());
        env.set_screen_mode(ScreenKind::Login);
        let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path();
        for (name, toc) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Login) {
            load_addon(&env.loader_env(), &toc).unwrap_or_else(|error| panic!("{name}: {error}"));
        }
        // No post-load/post-event workarounds or settle_headless_startup chat hide.
        // Invoke only the original screen-change path and inspect its original policy.
        env.exec(r#"
            GlueParent_SetScreen('login')
            local source = debug.getinfo(AllowChatFramesToShow).source
            assert(source:find('Blizzard_GlueParent/Mainline/GlueParent.lua', 1, true), source)
            assert(GlueParent.currentScreen == 'login' and AccountLogin:IsShown())
            assert(AllowChatFramesToShow(ChatFrame1) == false)
            assert(not ChatFrame1:IsShown(), 'chat visibility before any settling fixture')
        "#).unwrap();
        assert!(env.state().borrow().lua_errors.is_empty());
    }
}
