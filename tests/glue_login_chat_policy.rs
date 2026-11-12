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
        let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
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
        let errors = env.state().borrow().lua_errors.clone();
        let unexpected: Vec<_> = errors.iter().filter(|error| {
            // Review136: only these two exact PTR loader residues belong to R64.
            // Match the first-line diagnostic, never a substring in a traceback.
            let known_r64 = cfg!(feature = "client-ptr") && [
                "Blizzard_PTRFeedback.lua:729: attempt to index global 'CharCustomizeFrame' (a nil value)",
                "Blizzard_PTRFeedback_Events_Glue.lua:71: attempt to index global 'CharCustomizeFrame' (a nil value)",
            ].iter().any(|suffix| error.lines().next().is_some_and(|line| line.ends_with(suffix)));
            if known_r64 {
                eprintln!("known PTR loader residue (R64): {error}");
            }
            !known_r64
        }).collect();
        assert!(unexpected.is_empty(), "unexpected unassisted login errors: {unexpected:#?}");
    }
}
