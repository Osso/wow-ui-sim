//! Tests for `C_ScenarioInfo` probes backed by `SimState.scenario`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{ScenarioState, ScenarioStep};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn get_scenario_info_returns_nothing_when_not_in_scenario() {
    let env = env();
    let result: Option<i32> = env.eval("return C_ScenarioInfo.GetScenarioInfo()").unwrap();
    assert!(
        result.is_none(),
        "GetScenarioInfo should return nothing out of scenario"
    );
}

#[test]
fn get_scenario_step_info_returns_nothing_when_not_in_scenario() {
    let env = env();
    let result: Option<i32> = env
        .eval("return C_ScenarioInfo.GetScenarioStepInfo()")
        .unwrap();
    assert!(
        result.is_none(),
        "GetScenarioStepInfo should return nothing out of scenario"
    );
}

#[test]
fn get_scenario_bonus_step_reward_returns_nil_when_not_in_scenario() {
    let env = env();
    let result: Option<i32> = env
        .eval("return C_ScenarioInfo.GetScenarioBonusStepRewardQuestID(1)")
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn is_tiered_entrance_false_by_default() {
    let env = env();
    let result: bool = env
        .eval("return C_ScenarioInfo.IsTieredEntranceScenario()")
        .unwrap();
    assert!(!result);
}

#[test]
fn get_scenario_info_returns_seeded_scenario() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.scenario = ScenarioState {
            in_scenario: true,
            name: "Assault on Violet Hold".into(),
            unit_criteria: Default::default(),
            scenario_id: 101,
            current_step: 2,
            num_steps: 3,
            scenario_type: 1,
            texture_kit: "violethold".into(),
            is_tiered_entrance: false,
            tiered_entrance_active_spells: Vec::new(),
            display_theme_color: None,
            steps: Vec::new(),
        };
    }
    let (name, id, current, num): (String, i32, i32, i32) = env
        .eval(
            r#"
            local info = C_ScenarioInfo.GetScenarioInfo()
            return info.name, info.scenarioID, info.currentStage, info.numStages
            "#,
        )
        .unwrap();
    assert_eq!(name, "Assault on Violet Hold");
    assert_eq!(id, 101);
    assert_eq!(current, 2);
    assert_eq!(num, 3);
}

#[test]
fn get_scenario_step_info_returns_current_step() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.scenario = ScenarioState {
            in_scenario: true,
            name: "Test Scenario".into(),
            unit_criteria: Default::default(),
            scenario_id: 5,
            current_step: 1,
            num_steps: 2,
            scenario_type: 0,
            texture_kit: String::new(),
            is_tiered_entrance: false,
            tiered_entrance_active_spells: Vec::new(),
            display_theme_color: None,
            steps: vec![ScenarioStep {
                step_id: 1,
                title: "Kill the boss".into(),
                description: "Defeat the encounter".into(),
                num_criteria: 3,
                completed: false,
                is_bonus_step: false,
                bonus_reward_quest_id: None,
            }],
        };
    }
    let (title, num_criteria): (String, i32) = env
        .eval(
            r#"
            local step = C_ScenarioInfo.GetScenarioStepInfo()
            return step.title, step.numCriteria
            "#,
        )
        .unwrap();
    assert_eq!(title, "Kill the boss");
    assert_eq!(num_criteria, 3);
}

#[test]
fn get_scenario_step_info_by_explicit_step_id() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.scenario = ScenarioState {
            in_scenario: true,
            name: "Multi Step".into(),
            unit_criteria: Default::default(),
            scenario_id: 7,
            current_step: 1,
            num_steps: 2,
            scenario_type: 0,
            texture_kit: String::new(),
            is_tiered_entrance: false,
            tiered_entrance_active_spells: Vec::new(),
            display_theme_color: None,
            steps: vec![
                ScenarioStep {
                    step_id: 1,
                    title: "Step One".into(),
                    description: String::new(),
                    num_criteria: 1,
                    completed: false,
                    is_bonus_step: false,
                    bonus_reward_quest_id: None,
                },
                ScenarioStep {
                    step_id: 2,
                    title: "Step Two".into(),
                    description: String::new(),
                    num_criteria: 2,
                    completed: false,
                    is_bonus_step: false,
                    bonus_reward_quest_id: None,
                },
            ],
        };
    }
    let title: String = env
        .eval("return C_ScenarioInfo.GetScenarioStepInfo(2).title")
        .unwrap();
    assert_eq!(title, "Step Two");
}

#[test]
fn get_scenario_bonus_step_reward_returns_nil_for_non_bonus() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.scenario = ScenarioState {
            in_scenario: true,
            name: "Bonus Test".into(),
            unit_criteria: Default::default(),
            scenario_id: 10,
            current_step: 1,
            num_steps: 2,
            scenario_type: 0,
            texture_kit: String::new(),
            is_tiered_entrance: false,
            tiered_entrance_active_spells: Vec::new(),
            display_theme_color: None,
            steps: vec![ScenarioStep {
                step_id: 1,
                title: "Normal Step".into(),
                description: String::new(),
                num_criteria: 1,
                completed: false,
                is_bonus_step: false,
                bonus_reward_quest_id: None,
            }],
        };
    }
    let result: Option<i32> = env
        .eval("return C_ScenarioInfo.GetScenarioBonusStepRewardQuestID(1)")
        .unwrap();
    assert!(result.is_none(), "non-bonus step should yield nil");
}

#[test]
fn get_scenario_bonus_step_reward_returns_quest_id_for_bonus() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.scenario = ScenarioState {
            in_scenario: true,
            name: "Bonus Quest Scenario".into(),
            unit_criteria: Default::default(),
            scenario_id: 20,
            current_step: 2,
            num_steps: 2,
            scenario_type: 0,
            texture_kit: String::new(),
            is_tiered_entrance: false,
            tiered_entrance_active_spells: Vec::new(),
            display_theme_color: None,
            steps: vec![
                ScenarioStep {
                    step_id: 1,
                    title: "Main".into(),
                    description: String::new(),
                    num_criteria: 1,
                    completed: true,
                    is_bonus_step: false,
                    bonus_reward_quest_id: None,
                },
                ScenarioStep {
                    step_id: 2,
                    title: "Bonus".into(),
                    description: String::new(),
                    num_criteria: 1,
                    completed: false,
                    is_bonus_step: true,
                    bonus_reward_quest_id: Some(99999),
                },
            ],
        };
    }
    let quest_id: i32 = env
        .eval("return C_ScenarioInfo.GetScenarioBonusStepRewardQuestID(2)")
        .unwrap();
    assert_eq!(quest_id, 99999);
}

#[test]
fn is_tiered_entrance_reflects_state() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.scenario.is_tiered_entrance = true;
    }
    let result: bool = env
        .eval("return C_ScenarioInfo.IsTieredEntranceScenario()")
        .unwrap();
    assert!(
        result,
        "IsTieredEntranceScenario should reflect seeded flag"
    );
}

#[test]
fn is_bonus_step_field_populated_in_step_info() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.scenario = ScenarioState {
            in_scenario: true,
            name: "Bonus Marker".into(),
            unit_criteria: Default::default(),
            scenario_id: 30,
            current_step: 1,
            num_steps: 1,
            scenario_type: 0,
            texture_kit: String::new(),
            is_tiered_entrance: false,
            tiered_entrance_active_spells: Vec::new(),
            display_theme_color: None,
            steps: vec![ScenarioStep {
                step_id: 1,
                title: "Bonus Step".into(),
                description: String::new(),
                num_criteria: 0,
                completed: false,
                is_bonus_step: true,
                bonus_reward_quest_id: Some(12345),
            }],
        };
    }
    let (is_bonus, reward_id): (bool, i32) = env
        .eval(
            r#"
            local step = C_ScenarioInfo.GetScenarioStepInfo(1)
            return step.isBonusStep, step.rewardQuestID
            "#,
        )
        .unwrap();
    assert!(is_bonus);
    assert_eq!(reward_id, 12345);
}
