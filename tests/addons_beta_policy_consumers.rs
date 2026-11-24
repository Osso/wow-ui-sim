#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

const RETAIL_AND_FOREVER_CASE: &str = r#"
    assert(not C_AddOns.GetScriptsDisallowedForBeta(), "beta policy defaults false")
    assert(not AreDangerousScriptsAllowed(), "dangerous scripts default to disallowed")
    A_Admin.SetKioskSession({ enabled = false, competitive = false })
    A_Admin.SetScriptsDisallowedForBeta(false)
    SetAllowDangerousScripts(false)
    betaPolicyRunExecuted = nil
    ChatFrame1EditBox:SetText("/run betaPolicyRunExecuted = true")
    ChatEdit_SendText(ChatFrame1EditBox, false)
    assert(betaPolicyRunExecuted == nil, "dangerous script warning blocks /run")
    local warning = StaticPopup_FindVisible("DANGEROUS_SCRIPTS_WARNING")
    assert(warning and warning:IsShown(), "original /run path shows warning")
    warning.button1:Click()
    assert(AreDangerousScriptsAllowed(), "original warning acceptance enables scripts")
    ChatFrame1EditBox:SetText("/run betaPolicyRunExecuted = true")
    ChatEdit_SendText(ChatFrame1EditBox, false)
    assert(betaPolicyRunExecuted == true, "accepted /run executes supplied code")

    A_Admin.SetScriptsDisallowedForBeta(true)
    GameMenuFrame_Show()
    local addonsButton
    for button in GameMenuFrame.buttonPool:EnumerateActive() do
        if button:GetText() == ADDONS then addonsButton = button end
    end
    assert(addonsButton and not addonsButton:IsEnabled(), "beta policy disables AddOns button")
    GameMenuFrame_EscapePressed()
    betaPolicyRunExecuted = nil
    ChatFrame1EditBox:SetText("/run betaPolicyRunExecuted = true")
    ChatEdit_SendText(ChatFrame1EditBox, false)
    assert(betaPolicyRunExecuted == nil, "beta policy blocks /run")
    assert(StaticPopup_FindVisible("DANGEROUS_SCRIPTS_WARNING") == nil, "beta policy skips warning")

    A_Admin.SetScriptsDisallowedForBeta(false)
    GameMenuFrame_Show()
    for button in GameMenuFrame.buttonPool:EnumerateActive() do
        if button:GetText() == ADDONS then addonsButton = button end
    end
    assert(addonsButton and addonsButton:IsEnabled(), "cleared beta policy enables AddOns button")
"#;

const PTR_CASE: &str = r#"
    GameMenuFrame_Show()
    local addonsButton
    for button in GameMenuFrame.buttonPool:EnumerateActive() do
        if button:GetText() == ADDONS then addonsButton = button end
    end
    assert(addonsButton and addonsButton:IsEnabled(), "PTR AddOns button remains enabled")
"#;

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
        env.exec(RETAIL_AND_FOREVER_CASE).expect("original beta-policy consumers");
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_original_game_menu_keeps_default_policy() {
    test_timeout! {
        let env = load_game_ui();
        env.exec(PTR_CASE).expect("unchanged PTR game-menu consumer");
    }
}
