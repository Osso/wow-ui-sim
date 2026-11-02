#![cfg(feature = "client-wowforever")]

use crate::common;
use wow_ui_sim::loader::{discover_blizzard_startup_addons_for_screen, load_startup_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

#[test]
fn login_loads_real_customization_frame_before_ptr_feedback_hooks() {
    test_timeout! {
        let env = common::lock_env(|| {
            let env = WowLuaEnv::new().expect("create Forever login environment");
            env.set_screen_size(1024.0, 768.0);
            env.set_screen_mode(ScreenKind::Login);
            let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
            let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Login);
            let index = |name: &str| startup.iter().position(|addon| addon.name == name)
                .unwrap_or_else(|| panic!("selected Login startup should include {name}"));
            let feedback_index = index("Blizzard_PTRFeedbackGlue");
            assert!(index("Blizzard_CustomizationUI") < index("Blizzard_CharacterCustomize"));
            assert!(index("Blizzard_CharacterCustomize") < index("Blizzard_CharacterCreate"));
            assert!(index("Blizzard_CharacterCreate") < feedback_index);

            for addon in startup.iter().take(feedback_index + 1) {
                load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
                    .unwrap_or_else(|error| panic!("load {}: {error}", addon.name));
            }
            env.apply_post_load_workarounds();
            settle_headless_startup(&env);
            env
        });

        env.exec(r#"
            assert(CharCustomizeFrame ~= nil and not CharCustomizeFrame:IsShown())
            assert(CharacterCreateFrame ~= nil and not CharacterCreateFrame:IsShown())
            assert(CharCustomizeFrame.parentFrame == CharacterCreateFrame)
            assert(CharCustomizeFrame:GetParent() == CharacterCreateFrame)
            assert(type(PTR_IssueReporter.GlueInit) == "function")
        "#).expect("the real customization frame should remain hidden on Login");
        let state = env.state().borrow();
        let frame = state.widgets.get_by_name("CharCustomizeFrame")
            .expect("selected customization XML should create CharCustomizeFrame");
        let owner = frame.owner_addon.expect("customization frame should have an addon owner");
        assert_eq!(state.addons[owner as usize].folder_name, "Blizzard_CharacterCustomize");
        let errors: Vec<_> = state.lua_errors.iter()
            .filter(|error| error.contains("Blizzard_PTRFeedback") || error.contains("CharCustomizeFrame"))
            .collect();
        assert!(errors.is_empty(), "feedback customization-hook errors: {errors:#?}");
    }
}
