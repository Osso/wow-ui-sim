//! `C_Housing.IsHousingServiceEnabled` — SimState-backed round-trip.

use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "retail-12-0-0")]
#[test]
fn neighborhood_initiative_active_viewing_and_level_gate() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local N = C_NeighborhoodInitiative
        assert(select('#', N.SetActiveNeighborhood('Neighborhood-A')) == 0)
        assert(N.GetActiveNeighborhood() == 'Neighborhood-A')
        N.SetViewingNeighborhood('Neighborhood-A')
        assert(N.IsViewingActiveNeighborhood())
        N.SetViewingNeighborhood('Neighborhood-B')
        assert(not N.IsViewingActiveNeighborhood())
        assert(N.GetActiveNeighborhood() == 'Neighborhood-A')
        local required = N.GetRequiredLevel()
        assert(type(required) == 'number')
        A_Admin.SetPlayerLevel(required - 1)
        assert(not N.PlayerMeetsRequiredLevel())
        A_Admin.SetPlayerLevel(required)
        assert(N.PlayerMeetsRequiredLevel())
    "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn neighborhood_initiative_request_delivers_after_registration() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local N = C_NeighborhoodInitiative
        N.SetViewingNeighborhood('Neighborhood-A')
        initiativeEvents = {}
        N.RequestNeighborhoodInitiativeInfo()
        N.RequestInitiativeActivityLog()
        assert(not N.GetNeighborhoodInitiativeInfo().isLoaded)
        assert(not N.GetInitiativeActivityLogInfo().isLoaded)
        local f = CreateFrame('Frame')
        f:RegisterEvent('NEIGHBORHOOD_INITIATIVE_UPDATED')
        f:RegisterEvent('INITIATIVE_ACTIVITY_LOG_UPDATED')
        f:SetScript('OnEvent', function(_, event, ...)
            assert(select('#', ...) == 0)
            local info = event == 'NEIGHBORHOOD_INITIATIVE_UPDATED'
                and N.GetNeighborhoodInitiativeInfo() or N.GetInitiativeActivityLogInfo()
            assert(info.isLoaded and info.neighborhoodGUID == 'Neighborhood-A')
            table.insert(initiativeEvents, event)
        end)
        assert(#initiativeEvents == 0)
    "#,
    )
    .unwrap();
    env.process_timers().unwrap();
    env.exec("assert(#initiativeEvents == 2)").unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn housing_market_and_fixture_debug_return_contracts() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(C_Housing.IsHousingMarketShopEnabled()) == 'boolean')
        assert(select('#', C_Housing.OnHouseFinderClickPlot(42)) == 0)
        assert(select('#', C_HouseExterior.GetFixtureDebugInfoForGUID('Fixture-42')) == 1)
        assert(C_HouseExterior.GetFixtureDebugInfoForGUID('Fixture-42') == nil)
        assert(select('#', C_HouseExterior.GetSelectedFixtureDebugInfo()) == 1)
        assert(C_HouseExterior.GetSelectedFixtureDebugInfo() == nil)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn neighborhood_initiative_concrete_records_access_and_group_membership() {
    use wow_ui_sim::c_api::c_neighborhood_initiative::model::*;
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        let model = &mut sim.housing.initiative;
        model.required_level = 40;
        model.player_has_access = true;
        model.group_neighborhoods.insert("Neighborhood-A".into());
        model.neighborhoods.insert(
            "Neighborhood-A".into(),
            NeighborhoodInitiativeInfo {
                initiative_id: 17,
                current_cycle_id: 3,
                progress_required: 100,
                current_progress: 37,
                player_total_contribution: 12,
                duration: 3600,
                title: "Build the garden".into(),
                description: "Plant seeds".into(),
                tasks: vec![InitiativeTaskInfo {
                    id: 71,
                    task_name: "Plant five seeds".into(),
                    progress_contribution_amount: 5,
                    times_completed: 2,
                    in_progress: true,
                    sort_order: 1,
                    reward_quest_id: 501,
                    chat_link: "|Hinitiative:71|h[Plant five seeds]|h".into(),
                    requirements_list: vec![CriteriaRequirement {
                        completed: true,
                        requirement_text: "Own a plot".into(),
                    }],
                    criteria_list: vec![CriteriaRequiredValue {
                        criteria_id: 91,
                        required_value: 5,
                    }],
                    ..Default::default()
                }],
                milestones: vec![InitiativeMilestoneInfo {
                    milestone_order_index: 1,
                    required_contribution_amount: 100,
                    rewards: vec![InitiativeMilestoneRewardInfo {
                        title: "Garden bench".into(),
                        decor_id: 123,
                        decor_quantity: 2,
                        favor: 25,
                        money: 10000,
                        ..Default::default()
                    }],
                }],
                ..Default::default()
            },
        );
        model.activity_logs.insert(
            "Neighborhood-A".into(),
            InitiativeActivityLogInfo {
                next_update_time: 123456,
                task_activity: vec![InitiativeActivityLogEntry {
                    task_id: 71,
                    player_name: "Alessio".into(),
                    task_name: "Plant five seeds".into(),
                    completion_time: 123400,
                    amount: 5,
                }],
                ..Default::default()
            },
        );
    }
    env.exec(r#"
        local N = C_NeighborhoodInitiative
        N.SetViewingNeighborhood('Neighborhood-A')
        N.SetActiveNeighborhood('Neighborhood-A')
        N.AddTrackedInitiativeTask(71)
        assert(N.PlayerHasInitiativeAccess())
        assert(N.IsPlayerInNeighborhoodGroup())
        A_Admin.SetPlayerLevel(39)
        assert(N.GetRequiredLevel() == 40 and not N.PlayerMeetsRequiredLevel())
        assert(N.PlayerHasInitiativeAccess(), 'entitlement must not become a level gate')
        A_Admin.SetPlayerLevel(40)
        assert(N.PlayerMeetsRequiredLevel())
        assert(N.GetInitiativeTaskChatLink(71) == '|Hinitiative:71|h[Plant five seeds]|h')
        assert(N.GetInitiativeTaskChatLink(999) == '')
        N.RequestNeighborhoodInitiativeInfo()
        N.RequestInitiativeActivityLog()
        local f = CreateFrame('Frame')
        dataEvents = 0
        concreteEventsFrame = f
        f:RegisterEvent('NEIGHBORHOOD_INITIATIVE_UPDATED')
        f:RegisterEvent('INITIATIVE_ACTIVITY_LOG_UPDATED')
        f:SetScript('OnEvent', function(_, event)
            if event == 'NEIGHBORHOOD_INITIATIVE_UPDATED' then
                local info = N.GetNeighborhoodInitiativeInfo()
                assert(info.isLoaded and info.neighborhoodGUID == 'Neighborhood-A')
                assert(info.initiativeID == 17 and info.currentCycleID == 3)
                assert(info.currentProgress == 37 and info.progressRequired == 100)
                assert(info.playerTotalContribution == 12 and info.duration == 3600)
                assert(info.title == 'Build the garden' and info.description == 'Plant seeds')
                local task = info.tasks[1]
                assert(task.ID == 71 and task.tracked and task.timesCompleted == 2)
                assert(task.inProgress and task.rewardQuestID == 501)
                assert(task.requirementsList[1].completed)
                assert(task.criteriaList[1].criteriaID == 91)
                assert(task.criteriaList[1].requiredValue == 5)
                assert(info.milestones[1].requiredContributionAmount == 100)
                assert(info.milestones[1].rewards[1].decorID == 123)
                info.tasks[1].taskName = 'not persisted'
                assert(N.GetNeighborhoodInitiativeInfo().tasks[1].taskName == 'Plant five seeds')
                N.SetActiveNeighborhood('Neighborhood-A') -- same-write reentry
            else
                local log = N.GetInitiativeActivityLogInfo()
                assert(log.isLoaded and log.neighborhoodGUID == 'Neighborhood-A')
                assert(log.nextUpdateTime == 123456 and log.taskActivity[1].taskID == 71)
                assert(log.taskActivity[1].playerName == 'Alessio')
                assert(log.taskActivity[1].completionTime == 123400 and log.taskActivity[1].amount == 5)
            end
            dataEvents = dataEvents + 1
        end)
    "#).unwrap();
    env.process_timers().unwrap();
    env.exec("assert(dataEvents == 2); concreteEventsFrame:UnregisterAllEvents()")
        .unwrap();
    env.state()
        .borrow_mut()
        .housing
        .initiative
        .player_has_access = false;
    env.exec(
        r#"
        assert(not C_NeighborhoodInitiative.PlayerHasInitiativeAccess())
        C_NeighborhoodInitiative.SetActiveNeighborhood('Neighborhood-B')
        assert(not C_NeighborhoodInitiative.IsPlayerInNeighborhoodGroup())
    "#,
    )
    .unwrap();
    let other = WowLuaEnv::new().unwrap();
    other
        .exec(
            r#"
        local N = C_NeighborhoodInitiative
        assert(N.GetActiveNeighborhood() == '')
        assert(not N.PlayerHasInitiativeAccess() and not N.IsPlayerInNeighborhoodGroup())
        assert(N.GetNeighborhoodInitiativeInfo() == nil and N.GetInitiativeActivityLogInfo() == nil)
    "#,
        )
        .unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn neighborhood_initiative_reply_keeps_requested_neighborhood_identity() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local N = C_NeighborhoodInitiative
        N.SetViewingNeighborhood('Neighborhood-A')
        N.RequestNeighborhoodInitiativeInfo()
        N.RequestInitiativeActivityLog()
        N.SetViewingNeighborhood('Neighborhood-B')
        assert(not N.GetNeighborhoodInitiativeInfo().isLoaded)
        replyEvents = 0
        local f = CreateFrame('Frame')
        f:RegisterEvent('NEIGHBORHOOD_INITIATIVE_UPDATED')
        f:RegisterEvent('INITIATIVE_ACTIVITY_LOG_UPDATED')
        f:SetScript('OnEvent', function() replyEvents = replyEvents + 1 end)
    "#,
    )
    .unwrap();
    env.process_timers().unwrap();
    env.exec(
        r#"
        local N = C_NeighborhoodInitiative
        assert(replyEvents == 0)
        assert(not N.GetNeighborhoodInitiativeInfo().isLoaded)
        assert(N.GetInitiativeActivityLogInfo() == nil)
        N.SetViewingNeighborhood('Neighborhood-A')
        assert(N.GetNeighborhoodInitiativeInfo().isLoaded)
        assert(N.GetInitiativeActivityLogInfo().isLoaded)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn housing_market_shop_policy_and_plot_selection_use_host_state() {
    let env = WowLuaEnv::new().unwrap();
    assert!(
        !env.eval::<bool>("return C_Housing.IsHousingMarketShopEnabled()")
            .unwrap()
    );
    env.state().borrow_mut().housing.market_shop_enabled = true;
    assert!(
        env.eval::<bool>("return C_Housing.IsHousingMarketShopEnabled()")
            .unwrap()
    );
    env.exec("C_Housing.OnHouseFinderClickPlot(42)").unwrap();
    assert_eq!(
        env.state().borrow().housing.house_finder_selected_plot,
        Some(42)
    );
    env.exec("C_Housing.OnHouseFinderClickPlot(7)").unwrap();
    assert_eq!(
        env.state().borrow().housing.house_finder_selected_plot,
        Some(7)
    );
    let other = WowLuaEnv::new().unwrap();
    assert!(
        !other
            .eval::<bool>("return C_Housing.IsHousingMarketShopEnabled()")
            .unwrap()
    );
    assert_eq!(
        other.state().borrow().housing.house_finder_selected_plot,
        None
    );
}

fn probe(env: &WowLuaEnv) -> bool {
    env.eval(r#"return C_Housing.IsHousingServiceEnabled()"#)
        .unwrap()
}

#[cfg(feature = "retail-12-0-0")]
mod freeplace_tests {
    use super::WowLuaEnv;

    #[test]
    fn housing_freeplace_explicit_and_repeated_toggles() {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            r#"
            for index, enabled in ipairs({true, false, false, true, true, false}) do
                C_HousingBasicMode.SetFreePlaceEnabled(enabled)
                assert(C_HousingBasicMode.IsFreePlaceEnabled() == enabled,
                    "free-place state differs after explicit write " .. index)
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn housing_freeplace_getter_and_setter_return_arity() {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            r#"
            for _, enabled in ipairs({true, false}) do
                assert(select('#', C_HousingBasicMode.SetFreePlaceEnabled(enabled)) == 0,
                    "free-place setter must return zero values")
                assert(select('#', C_HousingBasicMode.IsFreePlaceEnabled()) == 1,
                    "free-place getter must return one value")
                assert(type(C_HousingBasicMode.IsFreePlaceEnabled()) == "boolean",
                    "free-place getter must return a boolean")
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn housing_freeplace_isolates_lua_environments() {
        let first = WowLuaEnv::new().unwrap();
        first
            .exec("C_HousingBasicMode.SetFreePlaceEnabled(false)")
            .unwrap();
        let second = WowLuaEnv::new().unwrap();
        second
            .exec("C_HousingBasicMode.SetFreePlaceEnabled(true)")
            .unwrap();

        assert!(
            !first
                .eval::<bool>("return C_HousingBasicMode.IsFreePlaceEnabled()")
                .unwrap()
        );
        assert!(
            second
                .eval::<bool>("return C_HousingBasicMode.IsFreePlaceEnabled()")
                .unwrap()
        );

        first
            .exec("C_HousingBasicMode.SetFreePlaceEnabled(true)")
            .unwrap();
        second
            .exec("C_HousingBasicMode.SetFreePlaceEnabled(false)")
            .unwrap();
        assert!(
            first
                .eval::<bool>("return C_HousingBasicMode.IsFreePlaceEnabled()")
                .unwrap()
        );
        assert!(
            !second
                .eval::<bool>("return C_HousingBasicMode.IsFreePlaceEnabled()")
                .unwrap()
        );
    }

    #[test]
    fn housing_freeplace_preserves_housing_service_state() {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            r#"
            for _, serviceEnabled in ipairs({false, true}) do
                A_Admin.SetHousingServiceEnabled(serviceEnabled)
                for _, freePlaceEnabled in ipairs({true, false}) do
                    C_HousingBasicMode.SetFreePlaceEnabled(freePlaceEnabled)
                    assert(C_Housing.IsHousingServiceEnabled() == serviceEnabled,
                        "free-place write changed housing service availability")
                end
            end
            "#,
        )
        .unwrap();
    }
}

#[test]
fn defaults_to_true() {
    let env = WowLuaEnv::new().unwrap();
    assert!(probe(&env));
}

#[test]
fn admin_set_enables_and_disables() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetHousingServiceEnabled(false)").unwrap();
    assert!(!probe(&env));
    env.exec("A_Admin.SetHousingServiceEnabled(true)").unwrap();
    assert!(probe(&env));
}

#[test]
fn admin_no_arg_defaults_to_true() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetHousingServiceEnabled()").unwrap();
    assert!(probe(&env));
}

#[test]
fn other_c_housing_members_still_resolve_via_metamethod_fallback() {
    // Unimplemented C_Housing.* should return the stub-namespace no-op
    // function (which returns nil), not crash with "attempt to call a nil
    // value".
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local fn = C_Housing.SomeUnimplementedMember
            if type(fn) ~= "function" then return "missing_function" end
            if fn() ~= nil then return "non_nil_return" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn dashboard_bootstrap_members_have_safe_defaults() {
    let env = WowLuaEnv::new().unwrap();
    let (max_level, cooldown_is_nil): (i32, bool) = env
        .eval(
            r#"
            return C_Housing.GetMaxHouseLevel(), C_Housing.GetVisitCooldownInfo() == nil
            "#,
        )
        .unwrap();
    assert_eq!(max_level, 0);
    assert!(
        cooldown_is_nil,
        "GetVisitCooldownInfo should default to nil when no cooldown is active"
    );
}
