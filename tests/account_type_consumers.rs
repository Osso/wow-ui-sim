#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn account_type_controls_restricted_lfg_panel(env: &WowLuaEnv) {
    env.exec(r#"
        assert(C_AddOns.IsAddOnLoaded("Blizzard_GroupFinder"))
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
    "#).expect("the loaded GroupFinder consumer must follow supplied account state");
}
}

prefork_full_ui_case! {
fn account_type_controls_trial_store_alert(env: &WowLuaEnv) {
    env.exec(r#"
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
    "#).expect("the original StoreMicroButton consumer must distinguish normal and trial accounts");
}
}
