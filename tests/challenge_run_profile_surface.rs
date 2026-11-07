//! Declaration-backed per-member publication, including unsupported profiles.
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn supplied_criteria_order_step_selection_and_detached_snapshots() {
    if ACTIVE == ClientProfile::Wrath {
        return;
    }
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        A_Admin.SetScenarioCriteria({stepID=4, title='First stage', description='Supplied'}, {
            {criteriaID=21, description='First boss', quantityString='0', totalQuantity=1},
            {criteriaID=22, description='Forces', quantityString='25%', quantity=25,
                totalQuantity=100, isWeightedProgress=true},
        })
        assert(C_ScenarioInfo.GetScenarioStepInfo().numCriteria == 2)
        local boss = C_ScenarioInfo.GetCriteriaInfo(1)
        assert(boss.criteriaID == 21 and boss.description == 'First boss')
        assert(boss.quantity == 0 and boss.totalQuantity == 1 and not boss.completed)
        assert(boss.criteriaType == 0 and boss.flags == 0 and boss.assetID == 0)
        assert(boss.duration == 0 and boss.elapsed == 0 and not boss.failed)
        assert(not boss.isWeightedProgress and not boss.isFormatted)
        local forces = C_ScenarioInfo.GetCriteriaInfo(2)
        assert(forces.criteriaID == 22 and forces.quantityString == '25%')
        assert(forces.quantity == 25 and forces.isWeightedProgress)
        forces.quantity = 999
        assert(C_ScenarioInfo.GetCriteriaInfo(2).quantity == 25)
        A_Admin.SetScenarioCriteria({stepID=5, title='Second stage', description='Supplied'}, {
            {criteriaID=23, description='Second boss', quantityString='1', quantity=1,
                totalQuantity=1, completed=true, criteriaType=165, assetID=888},
        })
        assert(C_ScenarioInfo.GetScenarioStepInfo().stepID == 5)
        local second = C_ScenarioInfo.GetCriteriaInfo(1)
        assert(second.criteriaID == 23 and second.completed and second.quantity == 1)
        assert(second.criteriaType == 165 and second.assetID == 888)
        assert(C_ScenarioInfo.GetCriteriaInfoByStep(4, 2).quantity == 25)
        assert(C_ScenarioInfo.GetCriteriaInfoByStep(5, 1).description == 'Second boss')
        assert(C_ScenarioInfo.GetCriteriaInfo(2) == nil)
        assert(C_ScenarioInfo.GetCriteriaInfoByStep(99, 1) == nil)
        assert(C_ScenarioInfo.GetCriteriaInfo(0) == nil)
        A_Admin.ClearChallengeRun()
        assert(C_ScenarioInfo.GetCriteriaInfo(1) == nil)
        assert(C_ScenarioInfo.GetCriteriaInfoByStep(4, 2) == nil)
    "#).unwrap();
    let independent = WowLuaEnv::new().unwrap();
    independent.exec("assert(C_ScenarioInfo.GetCriteriaInfo(1) == nil)").unwrap();
}

#[test]
fn active_keystone_affixes_fill_four_slots_and_stop_at_zero() {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return;
    }
    let env = WowLuaEnv::new().unwrap();
    let (level, capped, zero_stops, leading_zero): (i32, i32, i32, i32) = env
        .eval(
            r#"
            local function start(affixIDs)
                A_Admin.StartChallengeRun({mapChallengeModeID=9011, eventMapID=7011, level=9,
                    affixIDs=affixIDs, wasActiveKeystoneCharged=true, numDeaths=0, timeLost=0})
                local level, affixes = C_ChallengeMode.GetActiveKeystoneInfo()
                A_Admin.ClearChallengeRun()
                return level, #affixes
            end
            local level, capped = start({3, 5, 7, 9, 10, 12})
            local _, zero_stops = start({3, 0, 7})
            local _, leading_zero = start({0, 5})
            return level, capped, zero_stops, leading_zero
            "#,
        )
        .unwrap();
    assert_eq!(level, 9, "the supplied level is published");
    assert_eq!(capped, 4, "the affix array holds four slots");
    assert_eq!(zero_stops, 1, "the first zero slot truncates the list");
    assert_eq!(leading_zero, 0, "a leading zero publishes no affix");
}

#[test]
fn challenge_run_members_follow_pinned_profile_declarations() {
    let env = WowLuaEnv::new().unwrap();
    if ACTIVE == ClientProfile::Wrath {
        env.exec(
            r#"
            assert(rawget(C_ChallengeMode, 'GetActiveChallengeMapID') == nil)
            assert(rawget(C_ChallengeMode, 'GetActiveKeystoneInfo') == nil)
            assert(rawget(C_ChallengeMode, 'GetDeathCount') == nil)
            assert(rawget(C_ScenarioInfo, 'GetCriteriaInfo') == nil)
            assert(rawget(C_ScenarioInfo, 'GetCriteriaInfoByStep') == nil)
            assert(select('#', GetWorldElapsedTimers()) == 0)
            for _, id in ipairs({1, 77}) do
                local name, elapsed, kind = GetWorldElapsedTime(id)
                assert(name == id and elapsed == 0 and kind == 0)
                assert(select('#', GetWorldElapsedTime(id)) == 3)
            end
        "#,
        )
        .unwrap();
        return;
    }
    env.exec(
        r#"
        assert(C_ChallengeMode.GetActiveChallengeMapID() == nil)
        assert(not C_ChallengeMode.IsChallengeModeActive())
        if C_ChallengeMode.GetActiveKeystoneInfo then
            assert(select('#', C_ChallengeMode.GetActiveKeystoneInfo()) == 3)
            local level, affixes, charged = C_ChallengeMode.GetActiveKeystoneInfo()
            assert(level == 0 and type(affixes) == 'table' and next(affixes) == nil)
            assert(charged == false)
        end
        A_Admin.StartChallengeRun({mapChallengeModeID=9011, eventMapID=7011, level=3,
            affixIDs={9}, wasActiveKeystoneCharged=false, numDeaths=1, timeLost=8})
        assert(C_ChallengeMode.GetActiveChallengeMapID() == 9011)
        assert(C_ChallengeMode.IsChallengeModeActive())
        A_Admin.SetScenarioCriteria({stepID=4, title='Fourth', description='Supplied'}, {
            {criteriaID=11, description='Forces', quantityString='41%', quantity=41,
                totalQuantity=100, isWeightedProgress=true},
        })
        assert(C_ScenarioInfo.GetScenarioStepInfo().stepID == 4)
        assert(C_ScenarioInfo.GetScenarioStepInfo().numCriteria == 1)
        assert(C_ScenarioInfo.GetCriteriaInfo(1).quantity == 41)
        assert(C_ScenarioInfo.GetCriteriaInfoByStep(4, 1).quantityString == '41%')
        assert(C_ScenarioInfo.GetCriteriaInfoByStep(5, 1) == nil)
        A_Admin.ClearChallengeRun()
        assert(C_ChallengeMode.GetActiveChallengeMapID() == nil)
        assert(C_ScenarioInfo.GetCriteriaInfo(1) == nil)
    "#,
    )
    .unwrap();
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        env.exec(
            r#"
            assert(select('#', C_ChallengeMode.GetDeathCount()) == 0)
        "#,
        )
        .unwrap();
    } else {
        env.exec(
            r#"
            assert(rawget(C_ChallengeMode, 'GetActiveKeystoneInfo') == nil)
            assert(rawget(C_ChallengeMode, 'GetDeathCount') == nil)
        "#,
        )
        .unwrap();
    }
}
