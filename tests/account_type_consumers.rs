#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::loader::BlizzardAddonOverride;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn load_account_consumers() -> WowLuaEnv {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_GroupFinder", "Blizzard_MicroMenu"],
        &[] as &[BlizzardAddonOverride<'_>],
    );
    assert!(
        loaded.iter().any(|name| name == "Blizzard_GroupFinder"),
        "selected GroupFinder addon missing from loaded closure: {loaded:?}"
    );
    assert!(
        loaded.iter().any(|name| name == "Blizzard_MicroMenu"),
        "selected MicroMenu addon missing from loaded closure: {loaded:?}"
    );
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn account_type_controls_restricted_lfg_panel() {
    crate::common::with_timeout(120, || {
        let env = load_account_consumers();
        env.exec(
            r#"
            A_Admin.SetAccountType({
                trial = false,
                veteranTrial = false,
                restricted = false,
                expansionTrial = false,
                expansionTrialRemaining = 0,
                restrictedLevel = 20,
                restrictedMoney = 0,
                restrictedProfession = 0,
            })
            local normalPanelValid = LFGListFrame_IsPanelValid(
                LFGListFrame,
                LFGListFrame.NothingAvailable
            )
            assert(not normalPanelValid,
                "unrestricted account must reject the restricted-account panel")

            A_Admin.SetAccountType({
                trial = true,
                veteranTrial = false,
                restricted = true,
                expansionTrial = true,
                expansionTrialRemaining = 3600,
                restrictedLevel = 20,
                restrictedMoney = 100000,
                restrictedProfession = 100,
            })
            assert(IsTrialAccount() and IsRestrictedAccount())
            assert(IsExpansionTrial())
            local expansionTrial, remaining = GetExpansionTrialInfo()
            assert(expansionTrial and remaining == 3600)
            local level, money, profession = GetRestrictedAccountData()
            assert(level == 20 and money == 100000 and profession == 100)
            assert(LFGListFrame_IsPanelValid(
                LFGListFrame,
                LFGListFrame.NothingAvailable
            ), "restricted account must accept the original restricted-account panel")
            "#,
        )
        .expect("the loaded GroupFinder consumer must follow supplied account state");
    });
}

#[test]
fn account_type_controls_trial_store_alert() {
    crate::common::with_timeout(120, || {
        let env = load_account_consumers();
        env.exec(
            r#"
            assert(StoreMicroButton ~= nil)
            A_Admin.SetPlayerLevel(20)
            SetCVarBitfield("closedInfoFrames", LE_FRAME_TUTORIAL_TRIAL_BANKED_XP, false)
            A_Admin.SetAccountType({
                trial = false,
                veteranTrial = false,
                restricted = false,
                expansionTrial = false,
                expansionTrialRemaining = 0,
                restrictedLevel = 20,
                restrictedMoney = 0,
                restrictedProfession = 0,
            })
            local normalAlert = StoreMicroButton:EvaluateAlertVisibility(UnitLevel("player"))
            assert(not normalAlert, "normal account must not show the trial-cap alert")

            A_Admin.SetAccountType({
                trial = true,
                veteranTrial = true,
                restricted = true,
                expansionTrial = false,
                expansionTrialRemaining = 0,
                restrictedLevel = 20,
                restrictedMoney = 100000,
                restrictedProfession = 100,
            })
            SetCVarBitfield("closedInfoFrames", LE_FRAME_TUTORIAL_TRIAL_BANKED_XP, false)
            local trialAlert = StoreMicroButton:EvaluateAlertVisibility(UnitLevel("player"))
            assert(trialAlert, "trial account at its supplied level cap must show the alert")
            assert(IsVeteranTrialAccount(), "veteran trial status must come from account state")
            assert(GetCVarBitfield("closedInfoFrames", LE_FRAME_TUTORIAL_TRIAL_BANKED_XP))
            "#,
        )
        .expect(
            "the original StoreMicroButton consumer must distinguish normal and trial accounts",
        );
    });
}
