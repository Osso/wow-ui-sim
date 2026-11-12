#![cfg(all(
    feature = "gui",
    any(feature = "client-ptr", feature = "client-wowforever")
))]
use crate::common;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

// Exact cached-source diagnostic forms, not a suffix/substring allowance.
const PTR_R64_FIRST_LINES: [&str; 2] = [
    "@Interface/AddOns/Blizzard_PTRFeedbackGlue/../Blizzard_PTRFeedback/Blizzard_PTRFeedback.lua: ...ackGlue/../Blizzard_PTRFeedback/Blizzard_PTRFeedback.lua:729: attempt to index global 'CharCustomizeFrame' (a nil value)",
    "@Interface/AddOns/Blizzard_PTRFeedbackGlue/Blizzard_PTRFeedback_Events_Glue.lua: ...ard_PTRFeedbackGlue/Blizzard_PTRFeedback_Events_Glue.lua:71: attempt to index global 'CharCustomizeFrame' (a nil value)",
];

fn known_ptr_r64_diagnostic(error: &str) -> bool {
    let first = error.lines().next().unwrap_or_default();
    let first = first.strip_prefix("Lua Error: ").unwrap_or(first);
    PTR_R64_FIRST_LINES.contains(&first)
}

#[test]
fn glue_login_r64_allowance_rejects_arbitrary_prefixes_and_other_errors() {
    for line in PTR_R64_FIRST_LINES {
        assert!(known_ptr_r64_diagnostic(line));
        assert!(known_ptr_r64_diagnostic(&format!("Lua Error: {line}")));
        assert!(known_ptr_r64_diagnostic(&format!(
            "{line}\nstack traceback:"
        )));
        assert!(!known_ptr_r64_diagnostic(&format!("unexpected: {line}")));
        assert!(!known_ptr_r64_diagnostic(&format!(
            "Lua Error: unexpected: {line}"
        )));
        assert!(!known_ptr_r64_diagnostic(&format!("another error\n{line}")));
        assert!(!known_ptr_r64_diagnostic(
            &line.replace("CharCustomizeFrame", "OtherFrame")
        ));
        assert!(!known_ptr_r64_diagnostic(
            &line.replace(":729:", ":730:").replace(":71:", ":72:")
        ));
        assert!(!known_ptr_r64_diagnostic(
            &line.replace("@Interface/AddOns/", "@Other/AddOns/")
        ));
    }
}

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
            // Review140: exact first line, allowing only the known Lua Error wrapper.
            let known_r64 = cfg!(feature = "client-ptr") && known_ptr_r64_diagnostic(error);
            if known_r64 {
                eprintln!("known PTR loader residue (R64): {error}");
            }
            !known_r64
        }).collect();
        assert!(unexpected.is_empty(), "unexpected unassisted login errors: {unexpected:#?}");
    }
}
