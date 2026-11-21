//! Supplied quest-giver progress requirements rendered by Blizzard's original panel.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn progress_env() -> WowLuaEnv {
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &cache,
        &[
            "Blizzard_AccessibilityTemplates",
            "Blizzard_FrameXML",
            "Blizzard_UIPanels_Game",
            "Blizzard_ObjectiveTracker",
            "Blizzard_SplashFrame",
        ],
        &[],
    );
    env.exec("C_CVar.SetCVar('questTextContrast', '0'); C_CVar.SetCVar('instantQuestText', '1')")
        .unwrap();
    env
}

fn show_progress_set(env: &WowLuaEnv, requirements: &str) {
    env.exec(&format!(
        "assert(A_Admin.SetQuestProgressRequirements(80000, {requirements})); A_Admin.OpenMultiQuestNpc(); SelectActiveQuest(2)"
    ))
    .unwrap();
}

fn assert_panel_set(env: &WowLuaEnv, expected: [&str; 5], gold: &str, silver: &str) {
    let rendered: (String, String, String, String, String, String, String) = env
        .eval(
            "return QuestProgressText:GetText(), QuestProgressItem1Name:GetText(), QuestProgressItem2Name:GetText(), QuestProgressItem3Name:GetText(), QuestProgressItem4Name:GetText(), QuestProgressRequiredMoneyFrame.GoldButton.Text:GetText(), QuestProgressRequiredMoneyFrame.SilverButton.Text:GetText()",
        )
        .unwrap();
    assert_eq!(rendered.0, expected[0]);
    assert_eq!(rendered.1, expected[1]);
    assert_eq!(rendered.2, expected[2]);
    assert_eq!(rendered.3, expected[3]);
    assert_eq!(rendered.4, expected[4]);
    assert_eq!(rendered.5, gold);
    assert_eq!(rendered.6, silver);
    let shown: (bool, bool, bool, bool, bool, String) = env
        .eval(
            "return QuestFrameProgressPanel:IsShown(), QuestProgressRequiredMoneyFrame:IsShown(), QuestProgressItem1:IsShown(), QuestProgressItem2:IsShown(), QuestProgressItem3:IsShown(), QuestProgressItem3.Count:GetText()",
        )
        .unwrap();
    assert_eq!(shown, (true, true, true, true, true, "6".into()));
    let currency_name: String = env
        .eval("return C_QuestOffer.GetQuestRequiredCurrencyInfo(1).name")
        .unwrap();
    assert_eq!(currency_name, expected[3]);
    let second_currency_amount: i32 = env
        .eval("return C_QuestOffer.GetQuestRequiredCurrencyInfo(2).requiredAmount")
        .unwrap();
    assert_eq!(second_currency_amount, 6);
    let money: i32 = env.eval("return GetQuestMoneyToGet()").unwrap();
    assert_eq!(money, 234567);
}

#[test]
fn original_progress_panel_renders_two_supplied_turnin_requirement_sets() {
    let env = progress_env();
    show_progress_set(
        &env,
        r#"{money=234567, text='Bring the requested supplies.', items={{name='Rugged Hide', texture='Interface\\Icons\\INV_Misc_LeatherScrap_03', requiredAmount=4}, {name='Iron Bar', texture='Interface\\Icons\\INV_Ingot_03', requiredAmount=2}}, currencies={{currencyID=77, name='Harbor Token', texture=134400, quality=1, requiredAmount=3}, {currencyID=78, name='Dock Seal', texture=134401, quality=2, requiredAmount=6}}, portraitDisplayID=0, portraitName='', portraitText=''}"#,
    );
    assert_panel_set(
        &env,
        [
            "Bring the requested supplies.",
            "Rugged Hide",
            "Iron Bar",
            "Harbor Token",
            "Dock Seal",
        ],
        "23",
        "45",
    );

    env.exec("A_Admin.CloseQuestNpc()").unwrap();
    show_progress_set(
        &env,
        r#"{money=234567, text='Deliver the expedition cargo.', items={{name='Mooncloth', texture='Interface\\Icons\\INV_Fabric_MoonRag_01', requiredAmount=8}, {name='Primal Fire', texture='Interface\\Icons\\INV_Elemental_Primal_Fire', requiredAmount=5}}, currencies={{currencyID=81, name='Expedition Mark', texture=134402, quality=1, requiredAmount=7}, {currencyID=82, name='Survey Stamp', texture=134403, quality=3, requiredAmount=6}}, portraitDisplayID=0, portraitName='', portraitText=''}"#,
    );
    assert_panel_set(
        &env,
        [
            "Deliver the expedition cargo.",
            "Mooncloth",
            "Primal Fire",
            "Expedition Mark",
            "Survey Stamp",
        ],
        "23",
        "45",
    );
}
