#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![common::blizzard_ui_dir()];
    common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::settle_headless_startup(&env);
    env
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn beta_policy_controls_original_game_menu_and_run_consumers() {
    test_timeout! {
        let env = load_game_ui();
        env.exec(r#"
            A_Admin.SetKioskSession({ enabled = false, competitive = false })
            A_Admin.SetScriptsDisallowedForBeta(true)
            GameMenuFrame_Show()
            local function find_addons_button()
                for button in GameMenuFrame.buttonPool:EnumerateActive() do
                    if button:GetText() == ADDONS then return button end
                end
            end
            local button = find_addons_button()
            assert(button and not button:IsEnabled(), "beta policy disables AddOns menu button")
            GameMenuFrame_EscapePressed()
            betaPolicyRunExecuted = nil
            ChatFrame1EditBox:SetText("/run betaPolicyRunExecuted = true")
            ChatEdit_SendText(ChatFrame1EditBox, false)
            assert(betaPolicyRunExecuted == nil, "/run is blocked by beta policy")

            A_Admin.SetScriptsDisallowedForBeta(false)
            GameMenuFrame_Show()
            button = find_addons_button()
            assert(button and button:IsEnabled(), "cleared beta policy enables AddOns menu button")
            GameMenuFrame_EscapePressed()
            ChatFrame1EditBox:SetText("/run betaPolicyRunExecuted = true")
            ChatEdit_SendText(ChatFrame1EditBox, false)
            assert(betaPolicyRunExecuted == true, "/run executes after beta policy clears")
        "#).expect("original beta-policy menu and slash consumers");
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_original_menu_and_run_consumers_keep_default_policy() {
    test_timeout! {
        let env = load_game_ui();
        env.exec(r#"
            GameMenuFrame_Show()
            local addonsButton
            for button in GameMenuFrame.buttonPool:EnumerateActive() do
                if button:GetText() == ADDONS then addonsButton = button end
            end
            assert(addonsButton and addonsButton:IsEnabled(), "PTR AddOns button remains enabled")
            GameMenuFrame_EscapePressed()
            ptrBetaPolicyRunExecuted = nil
            ChatFrame1EditBox:SetText("/run ptrBetaPolicyRunExecuted = true")
            ChatEdit_SendText(ChatFrame1EditBox, false)
            assert(ptrBetaPolicyRunExecuted == true, "PTR /run keeps executing")
        "#).expect("unchanged PTR menu and slash consumers");
    }
}
