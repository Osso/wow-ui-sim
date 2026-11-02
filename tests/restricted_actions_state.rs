use wow_ui_sim::lua_api::WowLuaEnv;

/// No original consumer: searched the pinned Retail AddOns tree for both
/// C_RestrictedActions getters and all six addon*RestrictionsForced CVars;
/// only Blizzard_APIDocumentationGenerated declares the getters.
#[test]
fn forced_restrictions_follow_cvars_and_preserve_supplied_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local names = {
            'addonCombatRestrictionsForced', 'addonEncounterRestrictionsForced',
            'addonChallengeModeRestrictionsForced', 'addonPvPMatchRestrictionsForced',
            'addonMapRestrictionsForced', 'addonChatRestrictionsForced',
        }
        local api, events = C_RestrictedActions, {}
        for _, name in ipairs(names) do SetCVar(name, '0') end
        local f = CreateFrame('Frame')
        f:RegisterEvent('ADDON_RESTRICTION_STATE_CHANGED')
        f:SetScript('OnEvent', function(_, _, kind, value)
            events[#events + 1] = {kind, value, api.GetAddOnRestrictionState(kind)}
        end)
        for kind = 0, 5 do
            assert(not api.IsAddOnRestrictionActive(kind))
            assert(SetCVar(names[kind + 1], '1'))
            assert(api.IsAddOnRestrictionActive(kind), names[kind + 1] .. ' did not activate')
            assert(events[#events][1] == kind and events[#events][2] == 1 and events[#events][3] == 1)
            local count = #events
            SetCVar(names[kind + 1], '1')
            assert(#events == count, 'unchanged CVar generated a transition')
            SetCVar(names[kind + 1], '0')
            assert(not api.IsAddOnRestrictionActive(kind))
            assert(events[#events][1] == kind and events[#events][2] == 0 and events[#events][3] == 0)
        end
        assert(#events == 12, 'only activation and deactivation events are emitted')
        A_Admin.SetAddOnRestrictionActive(1, true)
        SetCVar(names[2], '1')
        A_Admin.SetAddOnRestrictionActive(1, false)
        assert(api.IsAddOnRestrictionActive(1), 'forced CVar must retain restriction')
        SetCVar(names[2], '0')
        assert(not api.IsAddOnRestrictionActive(1))
    "#).unwrap();
}

/// No original consumer: searched pinned Retail AddOns for the restriction
/// getters, forced CVars and restriction-change event; only API docs matched.
#[test]
fn supplied_restriction_context_controls_encounter_pvp_map_and_chat() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local api = C_RestrictedActions
        local function active(kind) return api.IsAddOnRestrictionActive(kind) end
        SetCVar('addonChatRestrictionsEnabled', '1')
        -- status, encounter count, PvP state, map classification, map flags, chat client flag
        A_Admin.SetRestrictionContext(false, 0, 0, 1, 0, true)
        assert(active(4) and not active(5))
        A_Admin.SetInCombat(true)
        assert(active(0) and not active(5), 'combat alone does not activate chat')
        A_Admin.SetRestrictionContext(false, 1, 0, 1, 0, true)
        assert(active(1) and active(5), 'encounter counter enables chat')
        A_Admin.SetRestrictionContext(false, 0, 1, 1, 0, true)
        assert(not active(1) and active(3) and active(5), 'waiting PvP enables chat')
        A_Admin.SetRestrictionContext(false, 0, 4, 1, 0, true)
        assert(active(3) and active(5), 'post-round remains active')
        A_Admin.SetRestrictionContext(false, 0, 5, 1, 0, true)
        assert(not active(3) and not active(5), 'complete clears PvP and chat')
        A_Admin.SetRestrictionContext(true, 0, 0, 1, 512, true)
        assert(active(1) and not active(4) and not active(5), 'map flag blocks map and chat')
        A_Admin.SetRestrictionContext(true, 0, 0, 1, 0, false)
        assert(active(1) and active(4) and not active(5), 'client flag gates chat')
        A_Admin.SetRestrictionContext(true, 0, 0, 1, 0, true)
        assert(active(5))
        SetCVar('addonChatRestrictionsEnabled', '0')
        assert(not active(5), 'chat policy gates activation')
    "#,
    )
    .unwrap();
}

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
fn restricted_actions_predicate_is_false_for_every_type_during_dispatch() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local api = C_RestrictedActions
        local types = Enum.AddOnRestrictionType
        A_Admin.SetInCombat(true)
        assert(api.IsAddOnRestrictionActive(types.Combat))
        local seen = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_RESTRICTION_STATE_CHANGED')
        listener:SetScript('OnEvent', function(_, _, kind)
            seen[#seen + 1] = {kind, api.IsAddOnRestrictionActive(types.Combat), api.GetAddOnRestrictionState(types.Combat)}
        end)
        A_Admin.SetAddOnRestrictionActive(types.Encounter, true)
        assert(#seen == 1 and seen[1][1] == types.Encounter)
        assert(seen[1][2] == false, 'other active type reported active during dispatch')
        assert(seen[1][3] == Enum.AddOnRestrictionState.Active, 'state getter is unaffected')
        assert(api.IsAddOnRestrictionActive(types.Combat))
        assert(api.IsAddOnRestrictionActive(types.Encounter))
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
