#![cfg(feature = "retail-12-0-7")]
//! Authored only: all cases remain uncompiled/unrun until integration.

use wow_ui_sim::lua_api::WowLuaEnv;

fn audit_env(grouped: bool) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create audit environment");
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
    }
    env.exec("LeaveParty()").unwrap();
    if grouped {
        env.exec(
            "InviteToGroup('Ada-Realm'); InviteToGroup('Bea-Realm'); InviteToGroup('Cy-Realm')",
        )
        .unwrap();
    }
    env.state().borrow_mut().events.drain();
    env
}

#[test]
fn p1207_empty_defaults_and_exact_public_return_arities() {
    let env = audit_env(false);
    {
        let sim = env.state().borrow();
        assert!(sim.party_assistants.is_empty());
        assert!(sim.party_assistant_exclusions.is_empty());
        assert!(sim.party_category_guids.is_empty());
        assert!(!sim.party_operations_restricted);
        assert!(!sim.solo_follower_dungeon);
        assert!(sim.solo_group_guid.is_none());
        assert!(sim.solo_group_category.is_none());
    }
    env.exec(
        r#"
        assert(not UnitIsGroupAssistant('player'))
        assert(not C_PartyInfo.IsGUIDInGroup(UnitGUID('player')))
        assert(not C_PartyInfo.IsGUIDInGroup('Player-9-Missing', 2))
        assert(select('#', C_PartyInfo.IsGUIDInGroup('Player-9-Missing')) == 1)
        assert(select('#', C_PartyInfo.DemoteAssistant('player')) == 0)
        assert(select('#', C_PartyInfo.PromoteToAssistant('player')) == 0)
        assert(select('#', C_PartyInfo.PromoteToLeader('player')) == 0)
        assert(select('#', C_PartyInfo.UninviteUnit('player')) == 0)
        local function check(...)
            assert(select('#', ...) == 1)
            local updated = ...
            assert(updated == false and not issecretvalue(updated))
        end
        check(C_PartyInfo.SetEveryoneIsAssistant(true))
        assert(not IsEveryoneAssistant())
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().events.is_empty());
}

#[test]
fn p1207_assistant_roles_are_individual_and_read_host_changes_live() {
    let env = audit_env(true);
    env.exec(
        r#"
        C_PartyInfo.PromoteToAssistant('Ada', false)
        assert(UnitIsGroupAssistant('party1'))
        assert(UnitLeadsAnyGroup('party1'))
        assert(not UnitIsGroupAssistant('party2'))
        assert(not UnitIsGroupAssistant('player'))
        assert(not IsEveryoneAssistant())
        C_PartyInfo.DemoteAssistant('Ada-Realm', true)
        assert(not UnitIsGroupAssistant('party1'))
        C_PartyInfo.PromoteToAssistant('Ada', true)
        assert(not UnitIsGroupAssistant('party1'))
        C_PartyInfo.PromoteToAssistant('party1')
        assert(C_PartyInfo.SetEveryoneIsAssistant(true))
        assert(not C_PartyInfo.SetEveryoneIsAssistant(true))
        C_PartyInfo.DemoteAssistant('party2')
        assert(IsEveryoneAssistant())
        assert(not UnitIsGroupAssistant('party2'))
        assert(not UnitLeadsAnyGroup('party2'))
        assert(UnitIsGroupAssistant('party1') and UnitIsGroupAssistant('player'))
        assert(C_PartyInfo.SetEveryoneIsAssistant(false))
        assert(UnitIsGroupAssistant('party1'))
        assert(not UnitIsGroupAssistant('party3'))
    "#,
    )
    .unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.party_assistants.clear();
        sim.party_assistants.insert("Cy-Realm".into());
    }
    env.exec("assert(not UnitIsGroupAssistant('party1')); assert(UnitIsGroupAssistant('party3'))")
        .unwrap();
    env.exec("InviteToGroup('Ada-Other'); C_PartyInfo.PromoteToAssistant('Ada', false); assert(not UnitIsGroupAssistant('party1')); assert(not UnitIsGroupAssistant('party4'))")
        .unwrap();
}

#[test]
fn p1207_leader_changes_resolve_existing_members_and_notify_after_mutation() {
    let env = audit_env(true);
    env.exec(
        r#"
        leaderEvents = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('PARTY_LEADER_CHANGED')
        listener:SetScript('OnEvent', function(_, event, ...)
            assert(event == 'PARTY_LEADER_CHANGED' and select('#', ...) == 0)
            assert(UnitIsGroupLeader('party2'))
            leaderEvents = leaderEvents + 1
        end)
        C_PartyInfo.PromoteToLeader('Bea-Realm', true)
        assert(UnitIsGroupLeader('party2') and not UnitIsGroupLeader('player'))
        C_PartyInfo.PromoteToLeader('Nobody', true)
        C_PartyInfo.PromoteToLeader('party99')
        C_PartyInfo.PromoteToLeader('party2')
        assert(UnitIsGroupLeader('party2') and leaderEvents == 1)
        listener:UnregisterEvent('PARTY_LEADER_CHANGED')
    "#,
    )
    .unwrap();
    env.state().borrow_mut().party_leader_index = Some(2);
    env.exec("assert(UnitIsGroupLeader('party3')); assert(not UnitIsGroupLeader('party2')); C_PartyInfo.PromoteToLeader('player'); assert(UnitIsGroupLeader('player'))")
        .unwrap();
}

#[test]
fn p1207_uninvite_removes_one_member_rebases_leader_and_notifies_live_roster() {
    let env = audit_env(true);
    env.exec(
        r#"
        C_PartyInfo.PromoteToLeader('party3')
        C_PartyInfo.PromoteToAssistant('party1')
        rosterEvents = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('GROUP_ROSTER_UPDATE')
        listener:SetScript('OnEvent', function(_, event, ...)
            assert(event == 'GROUP_ROSTER_UPDATE' and select('#', ...) == 0)
            assert(GetNumGroupMembers() == 3)
            assert(UnitName('party1') == 'Bea-Realm')
            assert(UnitIsGroupLeader('party2'))
            assert(not UnitIsGroupAssistant('party1'))
            rosterEvents = rosterEvents + 1
        end)
        C_PartyInfo.UninviteUnit('Ada', 'fixture removal', false)
        assert(rosterEvents == 1)
        C_PartyInfo.UninviteUnit('Nobody', nil, true)
        C_PartyInfo.UninviteUnit('player')
        assert(GetNumGroupMembers() == 3 and rosterEvents == 1)
        listener:UnregisterEvent('GROUP_ROSTER_UPDATE')
        C_PartyInfo.UninviteUnit('Cy-Realm', nil, true)
        assert(UnitIsGroupLeader('player'))
        C_PartyInfo.PromoteToAssistant('party1')
        LeaveParty()
        assert(not UnitIsGroupAssistant('player'))
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().party_assistants.is_empty());
}

#[test]
fn p1207_guid_membership_reads_home_roster_and_explicit_category_live() {
    let env = audit_env(true);
    env.exec(
        r#"
        assert(C_PartyInfo.IsGUIDInGroup(UnitGUID('player')))
        assert(C_PartyInfo.IsGUIDInGroup(UnitGUID('party2'), 1))
        assert(not C_PartyInfo.IsGUIDInGroup('Player-77-ABC', 2))
        assert(not C_PartyInfo.IsGUIDInGroup(UnitGUID('player'), 2))
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .party_category_guids
        .entry(2)
        .or_default()
        .insert("Player-77-ABC".into());
    env.exec("assert(C_PartyInfo.IsGUIDInGroup('Player-77-ABC', 2)); assert(not C_PartyInfo.IsGUIDInGroup('Player-77-ABC', 1))")
        .unwrap();
    env.state().borrow_mut().party_category_guids.clear();
    env.state().borrow_mut().party_group_active = false;
    env.exec("assert(not C_PartyInfo.IsGUIDInGroup('Player-77-ABC', 2)); assert(not C_PartyInfo.IsGUIDInGroup(UnitGUID('player')))")
        .unwrap();
}

#[test]
fn p1207_state_is_environment_local_across_roles_membership_marker_and_solo_entry() {
    let first = audit_env(true);
    let second = audit_env(true);
    first.exec("C_PartyInfo.PromoteToAssistant('party2'); C_PartyInfo.PromoteToLeader('party2'); TargetUnit('player'); C_Macro.RunMacroText('/tm ~6')")
        .unwrap();
    first
        .state()
        .borrow_mut()
        .party_category_guids
        .entry(2)
        .or_default()
        .insert("Player-77-ABC".into());
    first.state().borrow_mut().party_operations_restricted = true;
    second
        .exec(
            r#"
        assert(not UnitIsGroupAssistant('party2'))
        assert(UnitIsGroupLeader('player'))
        assert(not C_PartyInfo.IsGUIDInGroup('Player-77-ABC', 2))
        assert(GetRaidTargetIndex('player') == nil)
        C_PartyInfo.PromoteToAssistant('party3')
    "#,
        )
        .unwrap();
    first
        .exec("assert(not UnitIsGroupAssistant('party3')); assert(UnitIsGroupAssistant('party2'))")
        .unwrap();
    assert!(!second.state().borrow().party_operations_restricted);
    first.state().borrow_mut().solo_group_guid = Some("Party-77-Solo".into());
    first.state().borrow_mut().solo_group_category = Some(2);
    assert!(second.state().borrow().solo_group_guid.is_none());
    assert!(second.state().borrow().solo_group_category.is_none());
}

#[test]
fn p1207_secure_callers_authenticate_declared_inputs_and_extras_without_unwrapping_roots() {
    let env = audit_env(true);
    env.exec(
        r#"
        SName = secretwrap('party2')
        SExact = secretwrap(true)
        SReason = secretwrap('fixture reason')
        SGuid = secretwrap(UnitGUID('party1'))
        SCategory = secretwrap(1)
        SEnabled = secretwrap(true)
        SExtra = secretwrap('ignored only after authentication')
        assert(C_PartyInfo.IsGUIDInGroup(SGuid, SCategory, SExtra))
        assert(select('#', C_PartyInfo.PromoteToAssistant(SName, SExact, SExtra)) == 0)
        assert(UnitIsGroupAssistant('party2'))
        C_PartyInfo.DemoteAssistant(SName, SExact, SExtra)
        assert(not UnitIsGroupAssistant('party2'))
        C_PartyInfo.PromoteToLeader(SName, SExact, SExtra)
        assert(UnitIsGroupLeader('party2'))
        local updated = C_PartyInfo.SetEveryoneIsAssistant(SEnabled, SExtra)
        assert(updated == true and not issecretvalue(updated))
        C_PartyInfo.UninviteUnit(SName, SReason, SExact, SExtra)
        assert(GetNumGroupMembers() == 3)
        for _, value in ipairs({SName,SExact,SReason,SGuid,SCategory,SEnabled,SExtra}) do
            assert(issecretvalue(value))
        end
        assert(secretunwrap(SName) == 'party2')
        assert(secretunwrap(SExact) == true)
        assert(debug.getstacktaint() == nil)
    "#,
    )
    .unwrap();
}

#[test]
fn p1207_tainted_callers_reject_secrets_in_every_argument_and_extra_before_validation() {
    let env = audit_env(true);
    env.exec(
        r#"
        local cases = {
            {C_PartyInfo.DemoteAssistant, {'party2', false}},
            {C_PartyInfo.PromoteToAssistant, {'party2', false}},
            {C_PartyInfo.PromoteToLeader, {'party2', false}},
            {C_PartyInfo.UninviteUnit, {'party2', 'reason', false}},
            {C_PartyInfo.IsGUIDInGroup, {UnitGUID('party1'), 1}},
            {C_PartyInfo.SetEveryoneIsAssistant, {true}},
        }
        -- All secrets are created securely, before tainting any probe.
        for _, case in ipairs(cases) do
            local api, plain = case[1], case[2]
            for position = 1, #plain + 1 do
                local args = {}
                for index, value in ipairs(plain) do args[index] = value end
                if position <= #plain then
                    args[position] = secretwrap(plain[position])
                else
                    args[position] = secretwrap('extra')
                end
                local secret = args[position]
                local function probe()
                    assert(debug.getstacktaint() == 'PartyAudit')
                    local ok, err = pcall(api, unpack(args))
                    assert(not ok and tostring(err):find('untainted', 1, true))
                    assert(issecretvalue(secret))
                    assert(not pcall(secretunwrap, secret))
                    assert(debug.getstacktaint() == 'PartyAudit')
                end
                debug.setobjecttaint(probe, 'PartyAudit')
                probe()
                assert(debug.getstacktaint() == nil and issecretvalue(secret))
            end
        end
        local secretExtra = secretwrap('extra')
        local function priority()
            for _, case in ipairs(cases) do
                local ok, err = pcall(case[1], {}, nil, nil, secretExtra)
                assert(not ok and tostring(err):find('untainted', 1, true))
            end
        end
        debug.setobjecttaint(priority, 'PartyAudit')
        priority()
        assert(GetNumGroupMembers() == 4)
        assert(UnitIsGroupLeader('player'))
        assert(not UnitIsGroupAssistant('party2'))
        assert(not IsEveryoneAssistant())
        local function publicProbe()
            C_PartyInfo.PromoteToAssistant('party2')
            assert(UnitIsGroupAssistant('party2'))
            assert(not issecretvalue(C_PartyInfo.IsGUIDInGroup(UnitGUID('party1'))))
            assert(debug.getstacktaint() == 'PartyAudit')
        end
        debug.setobjecttaint(publicProbe, 'PartyAudit')
        publicProbe()
    "#,
    )
    .unwrap();
}

#[test]
fn p1207_invalid_inputs_and_explicit_restriction_reject_atomically_then_recover() {
    let env = audit_env(true);
    env.exec(
        r#"
        assert(not pcall(C_PartyInfo.PromoteToAssistant, {}, false))
        assert(not pcall(C_PartyInfo.PromoteToAssistant, 'party2', 'bad'))
        assert(not pcall(C_PartyInfo.UninviteUnit, 'party2', {}, false))
        assert(not pcall(C_PartyInfo.UninviteUnit, 'party2', nil, 4))
        assert(not pcall(C_PartyInfo.SetEveryoneIsAssistant, 1))
        assert(not pcall(C_PartyInfo.IsGUIDInGroup, {}, 1))
        assert(not pcall(C_PartyInfo.IsGUIDInGroup, 'Player-9-Missing', 1.5))
        assert(not pcall(C_PartyInfo.IsGUIDInGroup, 'Player-9-Missing', 3))
        assert(GetNumGroupMembers() == 4 and not UnitIsGroupAssistant('party2'))
    "#,
    )
    .unwrap();
    env.state().borrow_mut().party_operations_restricted = true;
    env.exec(
        r#"
        assert(not pcall(C_PartyInfo.PromoteToAssistant, 'party2'))
        assert(not pcall(C_PartyInfo.DemoteAssistant, 'party2'))
        assert(not pcall(C_PartyInfo.PromoteToLeader, 'party2'))
        assert(not pcall(C_PartyInfo.UninviteUnit, 'party2'))
        assert(not pcall(C_PartyInfo.SetEveryoneIsAssistant, true))
        assert(C_PartyInfo.IsGUIDInGroup(UnitGUID('party1')))
        assert(GetNumGroupMembers() == 4 and UnitIsGroupLeader('player'))
        assert(not UnitIsGroupAssistant('party2') and not IsEveryoneAssistant())
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().events.is_empty());
    env.state().borrow_mut().party_operations_restricted = false;
    env.state().borrow_mut().player.in_combat = true;
    env.exec("C_PartyInfo.PromoteToAssistant('party2'); assert(UnitIsGroupAssistant('party2'))")
        .unwrap();
}

#[test]
fn p1207_solo_group_formed_requires_host_payload_and_observed_entry_and_reentry() {
    let env = audit_env(false);
    env.exec(
        r#"
        formations = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('GROUP_FORMED')
        listener:SetScript('OnEvent', function(_, event, ...)
            assert(event == 'GROUP_FORMED' and select('#', ...) == 2)
            local category, guid = ...
            assert(category == 2 and type(guid) == 'string')
            assert(not issecretvalue(category) and not issecretvalue(guid))
            assert(IsInInstance())
            assert(GetNumGroupMembers() == 0)
            table.insert(formations, guid)
        end)
    "#,
    )
    .unwrap();
    env.fire_on_update(0.0).unwrap();
    env.state().borrow_mut().has_active_delve = true;
    env.fire_on_update(0.0).unwrap();
    assert_eq!(env.eval::<i32>("return #formations").unwrap(), 0);
    {
        let mut sim = env.state().borrow_mut();
        sim.solo_group_category = Some(2);
        sim.solo_group_guid = Some("Party-77-Delve".into());
    }
    env.fire_on_update(0.0).unwrap();
    env.fire_on_update(0.0).unwrap();
    assert_eq!(
        env.eval::<String>("return table.concat(formations, ',')")
            .unwrap(),
        "Party-77-Delve"
    );
    env.state().borrow_mut().has_active_delve = false;
    env.fire_on_update(0.0).unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.world.in_instance = true;
        sim.solo_follower_dungeon = true;
        sim.solo_group_guid = Some("Party-77-Follower".into());
    }
    env.fire_on_update(0.0).unwrap();
    env.fire_on_update(0.0).unwrap();
    assert_eq!(env.eval::<i32>("return #formations").unwrap(), 2);
    env.state().borrow_mut().solo_group_guid = Some("Party-77-Follower-2".into());
    env.fire_on_update(0.0).unwrap();
    assert_eq!(env.eval::<i32>("return #formations").unwrap(), 3);
    env.state().borrow_mut().party_group_active = true;
    env.fire_on_update(0.0).unwrap();
    assert_eq!(env.eval::<i32>("return #formations").unwrap(), 3);
    env.state().borrow_mut().party_group_active = false;
    env.fire_on_update(0.0).unwrap();
    assert_eq!(env.eval::<i32>("return #formations").unwrap(), 4);
    env.state().borrow_mut().solo_follower_dungeon = false;
    env.fire_on_update(0.0).unwrap();
    assert_eq!(env.eval::<i32>("return #formations").unwrap(), 4);
    let other = audit_env(false);
    other.fire_on_update(0.0).unwrap();
    assert!(
        !other
            .state()
            .borrow()
            .events
            .pending()
            .iter()
            .any(|event| event.name == "GROUP_FORMED")
    );
}

#[test]
fn p1207_conditional_tm_and_cached_secure_action_observe_live_marker_state() {
    let env = audit_env(false);
    env.exec(
        r#"
        TargetUnit('player')
        markerEvents = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('RAID_TARGET_UPDATE')
        listener:SetScript('OnEvent', function() markerEvents = markerEvents + 1 end)
        C_Macro.RunMacroText('/tm ~3')
        assert(GetRaidTargetIndex('target') == 3 and markerEvents == 1)
        C_Macro.RunMacroText('/tm ~5')
        C_Macro.RunMacroText('/tm ~0')
        assert(GetRaidTargetIndex('target') == 3 and markerEvents == 1)
        SetRaidTarget('target', 0)
        C_Macro.RunMacroText('/tm ~9\n/tm ~~2\n/tm !2\n/tm ~bad')
        assert(GetRaidTargetIndex('target') == nil and markerEvents == 2)
        C_Macro.RunMacroText('/tm ~0')
        assert(GetRaidTargetIndex('target') == nil and markerEvents == 3)
        C_Macro.RunMacroText('/tm [@player,exists] ~7')
        assert(GetRaidTargetIndex('player') == 7 and markerEvents == 4)
        SetRaidTarget('player', 0)
        MacroSecret = secretwrap('/tm ~4')
        ExtraSecret = secretwrap('extra')
        C_Macro.RunMacroText(MacroSecret, secretwrap('LeftButton'), ExtraSecret)
        assert(GetRaidTargetIndex('player') == 4 and issecretvalue(MacroSecret))
        local function denied()
            local ok, err = pcall(C_Macro.RunMacroText, {}, nil, ExtraSecret)
            assert(not ok and tostring(err):find('untainted', 1, true))
            assert(debug.getstacktaint() == 'MarkerAudit')
        end
        debug.setobjecttaint(denied, 'MarkerAudit')
        denied()
        local macroArgs = {'/tm ~8', 'LeftButton', 'extra'}
        for position = 1, 3 do
            local args = {unpack(macroArgs)}
            args[position] = secretwrap(args[position])
            local function positionalDenied()
                local ok, err = pcall(C_Macro.RunMacroText, unpack(args))
                assert(not ok and tostring(err):find('untainted', 1, true))
                assert(issecretvalue(args[position]))
                assert(debug.getstacktaint() == 'MarkerAudit')
            end
            debug.setobjecttaint(positionalDenied, 'MarkerAudit')
            positionalDenied()
        end
        assert(GetRaidTargetIndex('player') == 4)
        SetRaidTarget('player', 0)
    "#,
    )
    .unwrap();
    let home = std::env::var_os("HOME").expect("HOME for authenticated retail cache");
    let path = std::path::PathBuf::from(home)
        .join(".cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_FrameXML/SecureTemplates.lua");
    let vendor =
        std::fs::read_to_string(&path).expect("cached SecureTemplates.lua required; no substitute");
    env.exec(&vendor)
        .expect("execute entire unmodified cached consumer");
    env.exec(
        r#"
        local button = CreateFrame('Button')
        button:SetAttribute('type', 'raidtarget')
        button:SetAttribute('unit', 'player')
        button:SetAttribute('useOnKeyDown', false)
        button:SetAttribute('action', 'set-unmarked')
        button:SetAttribute('marker', 5)
        assert(SecureActionButton_OnClick(button, 'LeftButton', false, false, true))
        assert(GetRaidTargetIndex('player') == 5)
        local before = markerEvents
        button:SetAttribute('marker', 8)
        assert(SecureActionButton_OnClick(button, 'LeftButton', false, false, true))
        assert(GetRaidTargetIndex('player') == 5 and markerEvents == before)
    "#,
    )
    .unwrap();
}
