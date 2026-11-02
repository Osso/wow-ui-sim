//! Original consumers: Blizzard_Tutorials_Professions.lua:496–535
//! (NewSpecChecker) and :540–567 (SpecPointsChecker).
#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::profession_spec_inputs::{ProfessionSpecInputs, SkillRank};
use wow_ui_sim::lua_api::WowLuaEnv;

const BLACKSMITHING: u32 = 164;
const DRAGON_ISLES_BLACKSMITHING: u32 = 2822;
const KHAZ_ALGAR_MINING: u32 = 2881;
/// SkillLineXTraitTree 70391: tab 302 belongs to 2822, tab 983 to 2881.
const DRAGON_ISLES_ARMORSMITHING_TAB: u32 = 302;
const KHAZ_ALGAR_MINING_TAB: u32 = 983;
/// WM55-Q4: Midnight Blacksmithing (ParentTierIndex 15) is read first.
const MIDNIGHT_BLACKSMITHING_KNOWLEDGE: i32 = 3151;

/// Records micro-button alerts raised after startup.
fn hook_alerts(env: &WowLuaEnv) {
    env.exec(
        r#"
        ProfessionAlerts = {}
        assert(ProfessionMicroButton.showProfessionSpellHighlights ~= true)
        hooksecurefunc('MainMenuMicroButton_ShowAlert', function(button, text)
            table.insert(ProfessionAlerts, { button == ProfessionMicroButton, text })
        end)
        "#,
    )
    .expect("alert hook installs");
}

fn alerts(env: &WowLuaEnv) -> (i32, String, bool) {
    env.eval(
        r#"
        local alert = ProfessionAlerts[1] or {}
        assert(#ProfessionAlerts == 0 or alert[1], 'alert targets ProfessionMicroButton')
        return #ProfessionAlerts, alert[2] or '', ProfessionMicroButton.showProfessionSpellHighlights == true
        "#,
    )
    .expect("alerts read")
}

fn unlockable_tab(skill: u32, tab: u32) -> ProfessionSpecInputs {
    let mut inputs = ProfessionSpecInputs::default();
    inputs.related_skill_ranks.insert(
        skill,
        SkillRank {
            current: 25,
            bonus: 0,
        },
    );
    inputs.enabled_skill_lines.insert(skill);
    inputs.tab_states.insert(tab, 2);
    inputs
}

fn new_spec_alert(env: &WowLuaEnv, inputs: ProfessionSpecInputs) -> (i32, String, bool) {
    hook_alerts(env);
    env.state().borrow_mut().profession_spec_inputs = inputs;
    env.exec("A_Admin.FireEvent('SKILL_LINES_CHANGED')")
        .expect("NewSpecChecker handles SKILL_LINES_CHANGED");
    alerts(env)
}

prefork_full_ui_case! {
fn new_spec_checker_alerts_with_the_blacksmithing_name(env: &WowLuaEnv) {
    let inputs = unlockable_tab(DRAGON_ISLES_BLACKSMITHING, DRAGON_ISLES_ARMORSMITHING_TAB);
    let expected = "You can now choose a new Blacksmithing Specialization to unlock";
    assert_eq!(new_spec_alert(env, inputs), (1, expected.into(), true));
}
}

prefork_full_ui_case! {
fn new_spec_checker_alerts_with_the_mining_name(env: &WowLuaEnv) {
    let inputs = unlockable_tab(KHAZ_ALGAR_MINING, KHAZ_ALGAR_MINING_TAB);
    let expected = "You can now choose a new Mining Specialization to unlock";
    assert_eq!(new_spec_alert(env, inputs), (1, expected.into(), true));
}
}

fn points_alert(env: &WowLuaEnv, quantity: i32) -> (i32, String, bool) {
    hook_alerts(env);
    {
        let mut state = env.state().borrow_mut();
        let inputs = &mut state.profession_spec_inputs;
        inputs.enabled_skill_lines.insert(BLACKSMITHING);
        inputs.points_eligible_skill_lines.insert(BLACKSMITHING);
        state
            .currency_info
            .get_mut(&MIDNIGHT_BLACKSMITHING_KNOWLEDGE)
            .expect("CurrencyTypes row is seeded")
            .quantity = quantity;
    }
    env.exec("A_Admin.FireEvent('CURRENCY_DISPLAY_UPDATE')")
        .expect("SpecPointsChecker handles CURRENCY_DISPLAY_UPDATE");
    alerts(env)
}

prefork_full_ui_case! {
fn spec_points_checker_alerts_at_five_knowledge(env: &WowLuaEnv) {
    let expected = "You have unspent Profession Specialization Knowledge.";
    assert_eq!(points_alert(env, 5), (1, expected.into(), true));
}
}

prefork_full_ui_case! {
fn spec_points_checker_stays_quiet_at_four_knowledge(env: &WowLuaEnv) {
    assert_eq!(points_alert(env, 4), (0, String::new(), false));
}
}
