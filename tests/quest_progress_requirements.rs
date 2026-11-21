//! Supplied quest-giver progress requirements rendered by Blizzard's original panel.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const FIRST_REQUIREMENTS: &str = r#"{money=234567, text='Bring the requested supplies.', items={{name='Rugged Hide', texture='Interface\\Icons\\INV_Misc_LeatherScrap_03', requiredAmount=4}, {name='Iron Bar', texture='Interface\\Icons\\INV_Ingot_03', requiredAmount=2}}, currencies={{currencyID=77, name='Harbor Token', texture=134400, quality=1, requiredAmount=3}, {currencyID=78, name='Dock Seal', texture=134401, quality=2, requiredAmount=6}}, portraitDisplayID=0, portraitName='', portraitText=''}"#;
const SECOND_REQUIREMENTS: &str = r#"{money=234567, text='Deliver the expedition cargo.', items={{name='Mooncloth', texture='Interface\\Icons\\INV_Fabric_MoonRag_01', requiredAmount=8}, {name='Primal Fire', texture='Interface\\Icons\\INV_Elemental_Primal_Fire', requiredAmount=5}}, currencies={{currencyID=81, name='Expedition Mark', texture=134402, quality=1, requiredAmount=7}, {currencyID=82, name='Survey Stamp', texture=134403, quality=3, requiredAmount=6}}, portraitDisplayID=701, portraitName='Harbor Marshal', portraitText='Report to the marshal.'}"#;

fn progress_env() -> WowLuaEnv {
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &cache,
        &[
            "Blizzard_AccessibilityTemplates",
            "Blizzard_FrameXML",
            "Blizzard_UIPanels_Game",
            "Blizzard_ObjectiveTracker",
            "Blizzard_UIParent",
            "Blizzard_UIParentPanelManager",
            "Blizzard_SplashFrame",
        ],
        &[],
    );
    env.exec("C_CVar.SetCVar('questTextContrast', '0'); C_CVar.SetCVar('instantQuestText', '1'); SplashFrame:Hide()")
        .unwrap();
    env
}

fn show_progress_set(env: &WowLuaEnv, requirements: &str) {
    env.exec(&format!(
        "assert(A_Admin.SetQuestProgressRequirements(80000, {requirements})); A_Admin.OpenMultiQuestNpc(); SelectActiveQuest(2)"
    ))
    .unwrap();
}

fn assert_panel_text_and_money(env: &WowLuaEnv, expected: [&str; 5]) {
    let shown: String = env
        .eval(
            "return table.concat({tostring(QuestProgressText:GetText()), tostring(QuestProgressItem1Name:GetText()), tostring(QuestProgressItem2Name:GetText()), tostring(QuestProgressItem3Name:GetText()), tostring(QuestProgressItem4Name:GetText()), tostring(QuestProgressRequiredMoneyFrame.GoldButton.Text:GetText()), tostring(QuestProgressRequiredMoneyFrame.SilverButton.Text:GetText()), tostring(QuestFrame:IsShown()), tostring(QuestFrame:IsVisible()), tostring(QuestFrameProgressPanel:IsShown()), tostring(QuestProgressRequiredMoneyFrame:IsShown()), tostring(GetProgressText()), tostring(GetNumQuestItems()), tostring(GetNumQuestCurrencies()), tostring(GetQuestMoneyToGet()), tostring(SplashFrame:IsShown())}, '|')",
        )
        .unwrap();
    let expected = format!(
        "{}|{}|{}|{}|{}|23|45|true|true|true|true|{}|2|2|234567|false",
        expected[0], expected[1], expected[2], expected[3], expected[4]
    );
    assert_eq!(shown, expected);
}

fn assert_panel_rows(env: &WowLuaEnv, item_amount: i32, currency_name: &str, currency_amount: i32) {
    let buttons: (bool, bool, bool, bool, String, String) = env
        .eval(
            "return QuestProgressItem1:IsShown(), QuestProgressItem2:IsShown(), QuestProgressItem3:IsShown(), QuestProgressItem4:IsShown(), QuestProgressItem1.Count:GetText(), QuestProgressItem3.Count:GetText()",
        )
        .unwrap();
    assert_eq!(
        buttons,
        (
            true,
            true,
            true,
            true,
            item_amount.to_string(),
            currency_amount.to_string()
        )
    );
    let item: (String, i32) = env
        .eval("local n, _, count = GetQuestItemInfo('required', 1); return n, count")
        .unwrap();
    assert_eq!(
        item.0,
        env.eval::<String>("return QuestProgressItem1Name:GetText()")
            .unwrap()
    );
    assert_eq!(item.1, item_amount);
    let currency: (String, i32) = env
        .eval("local c = C_QuestOffer.GetQuestRequiredCurrencyInfo(1); return c.name, c.requiredAmount")
        .unwrap();
    assert_eq!(currency, (currency_name.into(), currency_amount));
    assert_eq!(env.eval::<i32>("return GetNumQuestItems()").unwrap(), 2);
    assert_eq!(
        env.eval::<i32>("return GetNumQuestCurrencies()").unwrap(),
        2
    );
    assert_eq!(
        env.eval::<i32>("return GetQuestMoneyToGet()").unwrap(),
        234567
    );
}

fn assert_progress_set(
    env: &WowLuaEnv,
    expected: [&str; 5],
    amount: i32,
    currency: &str,
    currency_amount: i32,
) {
    assert_panel_text_and_money(env, expected);
    assert_panel_rows(env, amount, currency, currency_amount);
}

fn assert_turnin_portrait(env: &WowLuaEnv, expected: (i32, &str, &str)) {
    let portrait: (i32, String, String) = env.eval("return GetQuestPortraitTurnIn()").unwrap();
    assert_eq!(portrait, (expected.0, expected.1.into(), expected.2.into()));
}

fn assert_first_progress_set(env: &WowLuaEnv) {
    show_progress_set(env, FIRST_REQUIREMENTS);
    assert_progress_set(
        env,
        [
            "Bring the requested supplies.",
            "Rugged Hide",
            "Iron Bar",
            "Harbor Token",
            "Dock Seal",
        ],
        4,
        "Harbor Token",
        3,
    );
    assert_turnin_portrait(env, (0, "", ""));
}

fn assert_second_progress_set(env: &WowLuaEnv) {
    env.exec("A_Admin.CloseQuestNpc()").unwrap();
    show_progress_set(env, SECOND_REQUIREMENTS);
    assert_progress_set(
        env,
        [
            "Deliver the expedition cargo.",
            "Mooncloth",
            "Primal Fire",
            "Expedition Mark",
            "Survey Stamp",
        ],
        8,
        "Expedition Mark",
        7,
    );
    assert_turnin_portrait(env, (701, "Report to the marshal.", "Harbor Marshal"));
}

#[test]
fn original_progress_panel_renders_two_supplied_turnin_requirement_sets() {
    let env = progress_env();
    assert_first_progress_set(&env);
    assert_second_progress_set(&env);
}
