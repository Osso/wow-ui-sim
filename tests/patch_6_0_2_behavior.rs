//! Bonus objective queries use existing scenario step state, not invented defaults.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::ScenarioStep;

fn assert_bonus_step_queries(env: &WowLuaEnv) {
    {
        let mut state = env.state().borrow_mut();
        state.scenario.in_scenario = true;
        state.scenario.steps = vec![
            ScenarioStep {
                step_id: 1,
                title: "Main objective".into(),
                description: String::new(),
                num_criteria: 1,
                completed: false,
                is_bonus_step: false,
                bonus_reward_quest_id: None,
            },
            ScenarioStep {
                step_id: 4,
                title: "Rescue prisoners".into(),
                description: String::new(),
                num_criteria: 2,
                completed: false,
                is_bonus_step: true,
                bonus_reward_quest_id: Some(35001),
            },
            ScenarioStep {
                step_id: 9,
                title: "Collect supplies".into(),
                description: String::new(),
                num_criteria: 1,
                completed: true,
                is_bonus_step: true,
                bonus_reward_quest_id: None,
            },
        ];
    }
    let result: bool = env.eval(r#"
        local steps = C_Scenario.GetBonusSteps()
        assert(type(steps) == 'table' and #steps == 2)
        assert(steps[1] == 4 and steps[2] == 9)
        assert(C_Scenario.GetBonusStepRewardQuestID(4) == 35001)
        assert(C_Scenario.GetBonusStepRewardQuestID(1) == nil)
        assert(C_Scenario.GetBonusStepRewardQuestID(9) == nil)
        assert(C_Scenario.GetBonusStepRewardQuestID(999) == nil)
        steps[1] = 999
        assert(C_Scenario.GetBonusSteps()[1] == 4)
        return true
    "#).expect("ordered noncontiguous bonus IDs, optional reward, detached output");
    assert!(result);

    env.state().borrow_mut().scenario.steps.remove(1);
    let result: bool = env.eval(r#"
        local steps = C_Scenario.GetBonusSteps()
        assert(#steps == 1 and steps[1] == 9)
        assert(C_Scenario.GetBonusStepRewardQuestID(4) == nil)
        return true
    "#).expect("queries follow step removal");
    assert!(result);

    env.state().borrow_mut().scenario.in_scenario = false;
    let result: bool = env.eval(r#"
        assert(#C_Scenario.GetBonusSteps() == 0)
        assert(C_Scenario.GetBonusStepRewardQuestID(9) == nil)
        assert(not pcall(C_Scenario.GetBonusStepRewardQuestID, {}))
        return true
    "#).expect("inactive scenario excludes retained steps and invalid IDs fail");
    assert!(result);
}

fn assert_retired_member_absence(env: &WowLuaEnv) {
    let result: bool = env.eval(r#"
        for _, row in ipairs({
            {C_Scenario, 'GetBonusCriteriaInfo'},
            {C_Scenario, 'GetBonusStepInfo'},
            {C_Vignettes, 'GetVignetteInstanceID'},
        }) do
            assert(rawget(row[1], row[2]) == nil)
            assert(row[1][row[2]] == nil)
        end
        return true
    "#).expect("consumer-free retirements stay absent on raw and normal lookup");
    assert!(result);
}

#[test]
fn patch_6_0_2_bare_retired_member_absence() {
    let env = WowLuaEnv::new().expect("bare environment");
    assert_retired_member_absence(&env);
}

prefork_full_ui_case! {
fn patch_6_0_2_cached_retired_member_absence(env: &WowLuaEnv) {
    assert_retired_member_absence(env);
}
}

#[test]
fn patch_6_0_2_bare_bonus_step_queries() {
    let env = WowLuaEnv::new().expect("bare environment");
    assert_bonus_step_queries(&env);
}

prefork_full_ui_case! {
fn patch_6_0_2_cached_bonus_step_queries(env: &WowLuaEnv) {
    assert_bonus_step_queries(env);
}
}
