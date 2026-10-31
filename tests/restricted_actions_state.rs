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
