use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn restricted_actions_combat_transitions() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local api = C_RestrictedActions
        local types = Enum.AddOnRestrictionType
        local states = Enum.AddOnRestrictionState
        local events = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_RESTRICTION_STATE_CHANGED')
        listener:SetScript('OnEvent', function(_, _, kind, state)
            events[#events + 1] = {kind, state, api.GetAddOnRestrictionState(kind), api.IsAddOnRestrictionActive(kind)}
        end)
        for kind = types.Combat, types.Chat do
            assert(api.GetAddOnRestrictionState(kind) == states.Inactive)
            assert(not api.IsAddOnRestrictionActive(kind))
        end
        A_Admin.SetInCombat(true)
        assert(#events == 1, 'combat activation event')
        assert(events[1][1] == types.Combat and events[1][2] == states.Activating)
        assert(events[1][4] == false, 'activation predicate during event')
        assert(api.GetAddOnRestrictionState(types.Combat) == states.Active)
        assert(api.IsAddOnRestrictionActive(types.Combat))
        for kind = types.Encounter, types.Chat do
            assert(api.GetAddOnRestrictionState(kind) == states.Inactive)
            assert(not api.IsAddOnRestrictionActive(kind))
        end
        A_Admin.SetInCombat(false)
        assert(#events == 2, 'combat deactivation event')
        assert(events[2][1] == types.Combat and events[2][2] == states.Inactive)
        assert(events[2][3] == states.Inactive and events[2][4] == false)
        assert(api.GetAddOnRestrictionState(types.Combat) == states.Inactive)
        "#,
    )
    .unwrap();
}

#[test]
fn restricted_actions_reject_invalid_types() {
    let env = WowLuaEnv::new().unwrap();
    for api in ["GetAddOnRestrictionState", "IsAddOnRestrictionActive"] {
        for argument in ["", "nil", "-1", "6", "'bad'"] {
            let expression = format!("return C_RestrictedActions.{api}({argument})");
            assert!(env.eval::<rilua::Val>(&expression).is_err(), "{expression}");
        }
    }
}

#[test]
fn restricted_actions_supplied_types_are_independent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local api = C_RestrictedActions
        local events = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_RESTRICTION_STATE_CHANGED')
        listener:SetScript('OnEvent', function(_, _, kind, value)
            events[#events + 1] = {kind, value, api.GetAddOnRestrictionState(kind), api.IsAddOnRestrictionActive(kind)}
        end)
        for kind = 1, 5 do
            A_Admin.SetAddOnRestrictionActive(kind, true)
            assert(api.IsAddOnRestrictionActive(kind))
            assert(events[kind][1] == kind and events[kind][2] == 1)
            assert(events[kind][3] == 1 and events[kind][4] == false)
        end
        for kind = 1, 5 do
            A_Admin.SetAddOnRestrictionActive(kind, false)
            assert(not api.IsAddOnRestrictionActive(kind))
            assert(events[kind + 5][1] == kind and events[kind + 5][2] == 0)
            assert(events[kind + 5][3] == 0 and events[kind + 5][4] == false)
        end
    "#).unwrap();
}

#[test]
fn restricted_actions_follow_host_lifecycle_inputs() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local api = C_RestrictedActions
        FireEvent('ENCOUNTER_START')
        assert(api.IsAddOnRestrictionActive(1))
        FireEvent('ENCOUNTER_END')
        assert(not api.IsAddOnRestrictionActive(1))
        FireEvent('PVP_MATCH_ACTIVE')
        assert(api.IsAddOnRestrictionActive(3))
        FireEvent('PVP_MATCH_COMPLETE')
        assert(not api.IsAddOnRestrictionActive(3))
        A_Admin.StartChallengeRun({mapChallengeModeID=9001, eventMapID=7001, level=2, affixIDs={}, wasActiveKeystoneCharged=false, numDeaths=0, timeLost=0})
        assert(api.IsAddOnRestrictionActive(2))
        A_Admin.ClearChallengeRun()
        assert(not api.IsAddOnRestrictionActive(2))
    "#).unwrap();
}
