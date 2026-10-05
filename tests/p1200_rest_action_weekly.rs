#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_rest_action_queries_empty() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().action_bars.insert(3, 19750);
    env.exec(
        r#"
        local allowed,enabled = C_ActionBar.GetActionAutocast(3)
        assert(allowed == false and enabled == false)
        assert(C_ActionBar.HasRangeRequirements(3) == false)
        assert(C_ActionBar.IsActionInRange(3) == nil)
        assert(C_ActionBar.IsActionInRange(3, 'missing') == nil)
        assert(not pcall(C_ActionBar.HasRangeRequirements, 0))
    "#,
    )
    .unwrap();
    let guid: String = env.eval("return UnitGUID('player')").unwrap();
    {
        let mut state = env.state().borrow_mut();
        state
            .action_spell_ranges
            .entry(19750)
            .or_default()
            .has_range_requirements = true;
        state
            .action_spell_ranges
            .get_mut(&19750)
            .unwrap()
            .in_range_by_guid
            .insert(guid, false);
        let pet = &mut state.pet_actions[0];
        pet.has_action = true;
        pet.spell_id = Some(19750);
        pet.auto_cast_allowed = true;
        pet.auto_cast_enabled = true;
    }
    env.exec(
        r#"
        local allowed,enabled = C_ActionBar.GetActionAutocast(3)
        assert(allowed and enabled)
        assert(C_ActionBar.HasRangeRequirements(3))
        assert(C_ActionBar.IsActionInRange(3, 'player') == false)
        assert(C_ActionBar.IsActionInRange(3, 'missing') == nil)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().action_bars.insert(3, 999999);
    env.exec(
        r#"
        local allowed,enabled = C_ActionBar.GetActionAutocast(3)
        assert(not allowed and not enabled)
        assert(not C_ActionBar.HasRangeRequirements(3))
        assert(C_ActionBar.IsActionInRange(3, 'player') == nil)
    "#,
    )
    .unwrap();
}
#[test]
fn p1200_rest_weekly_progress_empty() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(#C_WeeklyRewards.GetSortedProgressForActivity(5, true) == 0)")
        .unwrap();
    use wow_ui_sim::c_api::c_weekly_rewards::ActivityTierProgress;
    env.state().borrow_mut().weekly_reward_progress.insert(
        5,
        vec![
            ActivityTierProgress {
                activity_tier_id: 20,
                difficulty: 1,
                num_points: 7,
            },
            ActivityTierProgress {
                activity_tier_id: 30,
                difficulty: 8,
                num_points: 2,
            },
            ActivityTierProgress {
                activity_tier_id: 10,
                difficulty: 8,
                num_points: 3,
            },
        ],
    );
    env.exec(r#"
        local all = C_WeeklyRewards.GetSortedProgressForActivity(5, false)
        assert(#all == 3 and all[1].activityTierID == 10 and all[2].activityTierID == 30 and all[3].difficulty == 1)
        local combined = C_WeeklyRewards.GetSortedProgressForActivity(5, true)
        assert(#combined == 2 and combined[1].difficulty == 8 and combined[1].numPoints == 5)
        assert(combined[1].activityTierID == 10 and combined[2].numPoints == 7)
        combined[1].numPoints = 999
        assert(C_WeeklyRewards.GetSortedProgressForActivity(5, true)[1].numPoints == 5)
        assert(#C_WeeklyRewards.GetSortedProgressForActivity(2, true) == 0)
    "#).unwrap();
}
