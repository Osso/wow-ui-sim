#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::loader::BlizzardAddonOverride;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

#[cfg(feature = "client-retail")]
const ADDON_ROOTS: &[&str] = &["Blizzard_GroupFinder", "Blizzard_MicroMenu"];
#[cfg(feature = "client-wowforever")]
const ADDON_ROOTS: &[&str] = &["Blizzard_MicroMenu"];

#[cfg(feature = "client-retail")]
const LFG_ACCOUNT_SCRIPT: &str = r#"
    assert(C_AddOns.IsAddOnLoaded("Blizzard_GroupFinder"))
    A_Admin.SetAccountType({ accountType = 0, restricted = false,
        expansionTrial = false, expansionTrialExpiry = 0, restrictedLevel = 20,
        restrictedMoney = 0 })
    local normalValid = LFGListFrame_IsPanelValid(LFGListFrame, LFGListFrame.NothingAvailable)
    assert(not normalValid, "normal account must reject the restricted-account panel")
    A_Admin.SetAccountType({ accountType = 4, restricted = true,
        expansionTrial = true, expansionTrialExpiry = 3600, restrictedLevel = 20,
        restrictedMoney = 100000 })
    assert(not IsTrialAccount() and IsVeteranTrialAccount(), "account type 4 must be veteran trial only")
    A_Admin.SetAccountType({ accountType = 5, restricted = true,
        expansionTrial = true, expansionTrialExpiry = 3600, restrictedLevel = 20,
        restrictedMoney = 100000 })
    assert(IsTrialAccount() and not IsVeteranTrialAccount(), "account type 5 must be trial only")
    assert(IsRestrictedAccount() and IsExpansionTrial(), "restricted and active expansion flags must be visible")
    local expansionTrial, remaining = GetExpansionTrialInfo()
    assert(expansionTrial and remaining == 3600, "active expiry must report remaining seconds")
    local level, money, profession = GetRestrictedAccountData()
    assert(level == 20 and money == 100000 and profession == 0, "profession cap must be zero")
    assert(LFGListFrame_IsPanelValid(LFGListFrame, LFGListFrame.NothingAvailable),
        "restricted account must accept the original panel")
"#;

const STORE_ACCOUNT_SCRIPT: &str = r#"
    assert(StoreMicroButton ~= nil)
    A_Admin.SetManualClock(0)
    A_Admin.SetPlayerLevel(20)
    SetCVarBitfield("closedInfoFrames", LE_FRAME_TUTORIAL_TRIAL_BANKED_XP, false)
    A_Admin.SetAccountType({ accountType = 0, restricted = false,
        expansionTrial = false, expansionTrialExpiry = 0, restrictedLevel = 20,
        restrictedMoney = 0 })
    local normalAlert = StoreMicroButton:EvaluateAlertVisibility(UnitLevel("player"))
    assert(not normalAlert, "normal account must not show the trial-cap alert")
    A_Admin.SetAccountType({ accountType = 5, restricted = true,
        expansionTrial = true, expansionTrialExpiry = 3600, restrictedLevel = 20,
        restrictedMoney = 100000 })
    SetCVarBitfield("closedInfoFrames", LE_FRAME_TUTORIAL_TRIAL_BANKED_XP, false)
    local trialAlert = StoreMicroButton:EvaluateAlertVisibility(UnitLevel("player"))
    assert(trialAlert, "trial account at its supplied level cap must show the alert")
    assert(IsTrialAccount() and not IsVeteranTrialAccount() and IsRestrictedAccount())
    assert(IsExpansionTrial())
    local expansionTrial, remaining = GetExpansionTrialInfo()
    assert(expansionTrial and remaining == 3600)
    A_Admin.AdvanceTime(3601)
    assert(not IsExpansionTrial(), "expired expansion trial must be inactive")
    local expired, noRemaining = GetExpansionTrialInfo()
    assert(not expired and noRemaining == nil, "expired trial must return false, nil")
    local level, money, profession = GetRestrictedAccountData()
    assert(level == 20 and money == 100000 and profession == 0)
    assert(GetCVarBitfield("closedInfoFrames", LE_FRAME_TUTORIAL_TRIAL_BANKED_XP))
"#;

fn load_account_consumers() -> WowLuaEnv {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        ADDON_ROOTS,
        &[] as &[BlizzardAddonOverride<'_>],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_MicroMenu"));
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[cfg(feature = "client-retail")]
#[test]
fn account_type_controls_restricted_lfg_panel() {
    crate::common::with_timeout(120, || {
        load_account_consumers()
            .exec(LFG_ACCOUNT_SCRIPT)
            .expect("GroupFinder must follow supplied account state");
    });
}

#[test]
fn account_type_controls_trial_store_alert() {
    crate::common::with_timeout(120, || {
        load_account_consumers()
            .exec(STORE_ACCOUNT_SCRIPT)
            .expect("StoreMicroButton must distinguish normal and trial accounts");
    });
}
