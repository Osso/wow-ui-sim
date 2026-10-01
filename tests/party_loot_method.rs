//! Bounded Retail 12.0.5 fixtures; validation, resolution and event policies are inferred.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;

fn loot_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create party loot environment");
    env.exec(
        r#"
        assert(type(C_PartyInfo.GetLootMethod) == 'function')
        assert(type(C_PartyInfo.SetLootMethod) == 'function')
        LootNotifications = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('PARTY_LOOT_METHOD_CHANGED')
        listener:SetScript('OnEvent', function()
            LootNotifications = LootNotifications + 1
            LootObservedNumeric = C_PartyInfo.GetLootMethod()
            LootObservedLegacy = GetLootMethod()
            LootObservedThreshold = GetMasterLooterThreshold()
        end)
        "#,
    )
    .expect("register public loot listener");
    env.state().borrow_mut().events.drain();
    env
}

fn seed_master(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.loot_method.method = "master".into();
    state.loot_method.party_master_index = 2;
    state.loot_method.raid_master_index = 5;
    state.loot_method.threshold = 4;
}

fn assert_seeded_master(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(select('#', C_PartyInfo.GetLootMethod()) == 3)
        local method, party, raid = C_PartyInfo.GetLootMethod()
        assert(method == 2 and party == 2 and raid == 5)
        local token, legacyParty, legacyRaid = GetLootMethod()
        assert(token == 'master' and legacyParty == 2 and legacyRaid == 5)
        assert(GetMasterLooterThreshold() == 4)
        "#,
    )
    .expect("rejected call preserves shared master state");
    let state = env.state();
    let state = state.borrow();
    assert_eq!(state.loot_method.method, "master");
    assert_eq!(state.loot_method.party_master_index, 2);
    assert_eq!(state.loot_method.raid_master_index, 5);
    assert_eq!(state.loot_method.threshold, 4);
}

fn assert_change_events(env: &WowLuaEnv, expected: usize) {
    let events = env.state().borrow_mut().events.drain();
    assert_eq!(
        events.len(),
        expected,
        "only requested loot changes queue events"
    );
    for event in events {
        assert_eq!(event.name, "PARTY_LOOT_METHOD_CHANGED");
        assert!(
            event.args.is_empty(),
            "bounded change event carries no payload"
        );
        env.fire_event(&event.name)
            .expect("dispatch queued loot change");
        env.exec(
            r#"
            assert(LootObservedNumeric == C_PartyInfo.GetLootMethod())
            assert(LootObservedLegacy == GetLootMethod())
            assert(LootObservedThreshold == GetMasterLooterThreshold())
            "#,
        )
        .expect("listener observes committed shared state");
    }
}

fn seed_party_identity(env: &WowLuaEnv) {
    env.exec("A_Admin.SetPartySize(2)")
        .expect("create modeled party members");
    {
        let mut state = env.state().borrow_mut();
        state.party_members[0].name = "LootPartnerOne".into();
        state.party_members[1].name = "LootPartnerTwo".into();
    }
    env.state().borrow_mut().events.drain();
}

#[test]
fn fresh_getters_share_personal_default_and_preserve_threshold() {
    let env = loot_env();
    env.exec(
        r#"
        assert(select('#', C_PartyInfo.GetLootMethod()) == 3)
        local method, party, raid = C_PartyInfo.GetLootMethod()
        assert(type(method) == 'number' and method == Enum.LootMethod.Personal)
        assert(party == nil and raid == nil)
        assert(select('#', GetLootMethod()) == 3)
        local token, legacyParty, legacyRaid = GetLootMethod()
        assert(token == 'personalloot' and legacyParty == 0 and legacyRaid == 0)
        assert(GetMasterLooterThreshold() == 2)
        "#,
    )
    .expect("numeric C getter joins unchanged legacy default");
    assert_change_events(&env, 0);
}

#[test]
fn getter_translates_all_six_existing_tokens_from_shared_input() {
    let env = loot_env();
    for (token, numeric) in [
        ("freeforall", 0),
        ("roundrobin", 1),
        ("master", 2),
        ("group", 3),
        ("needbeforegreed", 4),
        ("personalloot", 5),
    ] {
        env.state().borrow_mut().loot_method.method = token.into();
        env.exec(&format!(
            r#"
            assert(select('#', C_PartyInfo.GetLootMethod()) == 3)
            local method, party, raid = C_PartyInfo.GetLootMethod()
            assert(type(method) == 'number' and method == {numeric})
            assert(party == nil and raid == nil)
            assert(GetLootMethod() == '{token}')
            "#,
        ))
        .expect("translate concrete legacy token");
    }
    assert_change_events(&env, 0);
}

#[test]
fn getter_exposes_only_positive_existing_master_indices_without_changing_legacy() {
    let env = loot_env();
    seed_master(&env);
    assert_seeded_master(&env);
    for (party, raid) in [(0, 5), (2, 0), (-1, -3)] {
        {
            let mut state = env.state().borrow_mut();
            state.loot_method.party_master_index = party;
            state.loot_method.raid_master_index = raid;
        }
        let (method, party_result, raid_result): (u8, Option<i32>, Option<i32>) =
            env.eval("return C_PartyInfo.GetLootMethod()").unwrap();
        assert_eq!(method, 2);
        assert_eq!(party_result, (party > 0).then_some(party));
        assert_eq!(raid_result, (raid > 0).then_some(raid));
        let (_, legacy_party, legacy_raid): (String, i32, i32) =
            env.eval("return GetLootMethod()").unwrap();
        assert_eq!((legacy_party, legacy_raid), (party, raid));
    }
    assert_change_events(&env, 0);
}

#[test]
fn non_master_setters_roundtrip_clear_indices_and_leave_threshold_unchanged() {
    let env = loot_env();
    for (symbol, numeric, token) in [
        ("Freeforall", 0, "freeforall"),
        ("Roundrobin", 1, "roundrobin"),
        ("Group", 3, "group"),
        ("Needbeforegreed", 4, "needbeforegreed"),
        ("Personal", 5, "personalloot"),
    ] {
        seed_master(&env);
        env.exec(&format!(
            r#"
            assert(Enum.LootMethod.{symbol} == {numeric})
            local function capture(...) return select('#', ...), ... end
            local count, success = capture(C_PartyInfo.SetLootMethod(Enum.LootMethod.{symbol}))
            assert(count == 1 and type(success) == 'boolean' and success)
            local method, party, raid = C_PartyInfo.GetLootMethod()
            assert(method == {numeric} and party == nil and raid == nil)
            local legacy, legacyParty, legacyRaid = GetLootMethod()
            assert(legacy == '{token}' and legacyParty == 0 and legacyRaid == 0)
            assert(GetMasterLooterThreshold() == 4)
            "#,
        ))
        .expect("accepted non-master enum updates both public views");
        let state = env.state();
        let state = state.borrow();
        assert_eq!(state.loot_method.method, token);
        assert_eq!(state.loot_method.party_master_index, 0);
        assert_eq!(state.loot_method.raid_master_index, 0);
        assert_eq!(state.loot_method.threshold, 4);
        drop(state);
        assert_change_events(&env, 1);
    }
    assert_eq!(env.eval::<i32>("return LootNotifications").unwrap(), 5);
}

#[test]
fn repeated_unchanged_selection_succeeds_without_duplicate_change_event() {
    let env = loot_env();
    env.exec("assert(C_PartyInfo.SetLootMethod(3))").unwrap();
    assert_change_events(&env, 1);
    env.exec(
        r#"
        assert(select('#', C_PartyInfo.SetLootMethod(3, nil)) == 1)
        assert(C_PartyInfo.SetLootMethod(3) == true)
        assert(C_PartyInfo.GetLootMethod() == 3 and GetLootMethod() == 'group')
        assert(GetMasterLooterThreshold() == 2)
        assert(LootNotifications == 1)
        "#,
    )
    .expect("inferred idempotent setter policy");
    assert_change_events(&env, 0);
}

#[test]
fn master_selection_resolves_modeled_party_and_player_roster_identity() {
    let env = loot_env();
    seed_party_identity(&env);
    env.state().borrow_mut().loot_method.threshold = 4;
    env.exec(
        r#"
        assert(IsInGroup() and not IsInRaid())
        assert(UnitExists('party2'))
        -- Ordinary literal matches explicit model input; do not declassify UnitName.
        local function capture(...) return select('#', ...), ... end
        local count, success = capture(C_PartyInfo.SetLootMethod(Enum.LootMethod.Masterlooter,
            'LootPartnerTwo', 2))
        assert(count == 1 and type(success) == 'boolean' and success)
        local method, party, raid = C_PartyInfo.GetLootMethod()
        assert(method == 2 and party == 2 and raid == nil)
        local token, legacyParty, legacyRaid = GetLootMethod()
        assert(token == 'master' and legacyParty == 2 and legacyRaid == 0)
        assert(GetMasterLooterThreshold() == 4, 'undocumented third arg is not threshold')
        "#,
    )
    .expect("master name resolves existing 1-based party member");
    assert_change_events(&env, 1);
    env.exec("assert(C_PartyInfo.SetLootMethod(2, 'LootPartnerTwo') == true)")
        .unwrap();
    assert_change_events(&env, 0);

    env.exec("A_Admin.SetPartySize(6)").unwrap();
    env.state().borrow_mut().player.name = "LootPlayer".into();
    env.state().borrow_mut().events.drain();
    env.exec(
        r#"
        assert(IsInRaid())
        -- Existing GetRaidRosterInfo model puts the player at roster index 1.
        assert(C_PartyInfo.SetLootMethod(2, 'LootPlayer') == true)
        local method, party, raid = C_PartyInfo.GetLootMethod()
        assert(method == 2 and party == nil and raid == 1)
        local token, legacyParty, legacyRaid = GetLootMethod()
        assert(token == 'master' and legacyParty == 0 and legacyRaid == 1)
        assert(GetMasterLooterThreshold() == 4)
        "#,
    )
    .expect("resolve existing player raid roster position without fabricating identity");
    assert_change_events(&env, 1);
}

#[test]
fn unresolved_missing_and_inactive_master_requests_return_false_atomically() {
    let env = loot_env();
    seed_party_identity(&env);
    seed_master(&env);
    env.exec(
        r#"
        local function unresolved(...)
            local function capture(...) return select('#', ...), ... end
            local count, success = capture(C_PartyInfo.SetLootMethod(2, ...))
            assert(count == 1 and type(success) == 'boolean' and success == false)
        end
        unresolved()
        unresolved(nil)
        unresolved('')
        unresolved('NotInModeledRoster')
        unresolved('LootPartnerTwo-UnmodeledRealm')
        "#,
    )
    .expect("inferred false policy does not fabricate master identity");
    assert_seeded_master(&env);
    assert_change_events(&env, 0);
    env.state().borrow_mut().party_group_active = false;
    env.exec("assert(C_PartyInfo.SetLootMethod(2, 'LootPartnerTwo') == false)")
        .unwrap();
    assert_seeded_master(&env);
    assert_change_events(&env, 0);
}

#[test]
fn public_setter_mutations_are_isolated_between_environments() {
    let first = loot_env();
    let second = loot_env();
    first.exec("assert(C_PartyInfo.SetLootMethod(1))").unwrap();
    second.exec("assert(C_PartyInfo.SetLootMethod(4))").unwrap();
    assert_eq!(
        first
            .eval::<u8>("return C_PartyInfo.GetLootMethod()")
            .unwrap(),
        1
    );
    assert_eq!(
        second.eval::<String>("return GetLootMethod()").unwrap(),
        "needbeforegreed"
    );
    assert_change_events(&first, 1);
    assert_change_events(&second, 1);
    first.exec("assert(C_PartyInfo.SetLootMethod(5))").unwrap();
    assert_eq!(
        second
            .eval::<u8>("return C_PartyInfo.GetLootMethod()")
            .unwrap(),
        4
    );
    assert_change_events(&first, 1);
    assert_change_events(&second, 0);
}

#[test]
fn lockdown_blocks_before_effects_while_combat_alone_allows_changes_and_reads() {
    let env = loot_env();
    for (combat, lockdown) in [(false, false), (false, true), (true, false), (true, true)] {
        seed_master(&env);
        {
            let mut state = env.state().borrow_mut();
            state.player.in_combat = combat;
            state.chat_messaging_lockdown = lockdown;
        }
        assert_seeded_master(&env);
        env.exec(&format!(
            r#"
            local ok, result = pcall(C_PartyInfo.SetLootMethod, 0)
            assert(ok == {allowed}, 'only explicit chat lockdown blocks')
            if ok then
                assert(result == true)
                assert(C_PartyInfo.GetLootMethod() == 0 and GetLootMethod() == 'freeforall')
                assert(GetMasterLooterThreshold() == 4)
            else
                assert(type(result) == 'string' and #result > 0)
            end
            "#,
            allowed = !lockdown,
        ))
        .expect("lockdown/combat matrix");
        if lockdown {
            assert_seeded_master(&env);
        }
        assert_change_events(&env, usize::from(!lockdown));
        let state = env.state();
        let state = state.borrow();
        assert_eq!(state.player.in_combat, combat);
        assert_eq!(state.chat_messaging_lockdown, lockdown);
    }
}

#[test]
fn malformed_enum_requests_reject_without_coercion_mutation_or_event() {
    let env = loot_env();
    seed_master(&env);
    env.exec(
        r#"
        local function reject(...)
            local ok, err = pcall(C_PartyInfo.SetLootMethod, ...)
            assert(not ok and type(err) == 'string' and #err > 0)
            local method, party, raid = C_PartyInfo.GetLootMethod()
            assert(method == 2 and party == 2 and raid == 5)
        end
        reject()
        reject(nil)
        for _, value in ipairs({-1, 6, 0.5, math.huge, -math.huge, '0', true,
                false, {}, function() end}) do
            reject(value)
        end
        reject(0/0)
        "#,
    )
    .expect("strict ordinary enum policy");
    assert_seeded_master(&env);
    assert_change_events(&env, 0);
}

#[test]
fn optional_master_requires_ordinary_string_even_for_non_master_methods() {
    let env = loot_env();
    seed_master(&env);
    env.exec(
        r#"
        for _, value in ipairs({0, true, false, {}, function() end}) do
            local ok, err = pcall(C_PartyInfo.SetLootMethod, 0, value)
            assert(not ok and type(err) == 'string' and #err > 0)
            assert(GetLootMethod() == 'master')
        end
        "#,
    )
    .expect("validate optional master before effects regardless of selected enum");
    assert_seeded_master(&env);
    assert_change_events(&env, 0);
    env.exec("assert(C_PartyInfo.SetLootMethod(0, 'UnusedOrdinaryName') == true)")
        .unwrap();
    assert_change_events(&env, 1);
}

#[test]
fn actual_secret_enum_and_master_reject_for_secure_and_tainted_callers() {
    let env = loot_env();
    seed_master(&env);
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let number = wrap_host_secret_number(lua.state_mut(), 0.0);
        lua.state_mut().push(number);
        let inserted = lua.set_global_val("SecretLootMethod", number);
        lua.state_mut().pop();
        inserted.expect("root actual secret enum");
        let string = wrap_host_secret_string(lua.state_mut(), "LootPartnerTwo");
        lua.state_mut().push(string);
        let inserted = lua.set_global_val("SecretLootMaster", string);
        lua.state_mut().pop();
        inserted.expect("root actual secret master string");
    }
    env.exec(
        r#"
        collectgarbage('collect')
        local function reject(...)
            local ok, err = pcall(C_PartyInfo.SetLootMethod, ...)
            assert(not ok and type(err) == 'string' and #err > 0)
            assert(issecretvalue(SecretLootMethod) and issecretvalue(SecretLootMaster))
            local method, party, raid = C_PartyInfo.GetLootMethod()
            assert(method == 2 and party == 2 and raid == 5)
            assert(GetLootMethod() == 'master' and GetMasterLooterThreshold() == 4)
        end
        local function probe()
            reject(SecretLootMethod)
            reject(2, SecretLootMaster)
            reject(0, SecretLootMaster)
        end
        assert(issecure())
        probe()
        assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'PartyLootFixture')
            probe()
            assert(debug.getstacktaint() == 'PartyLootFixture')
        end
        debug.setobjecttaint(addon, 'PartyLootFixture')
        addon()
        assert(issecure())
        "#,
    )
    .expect("conservative secret rejection preserves values and caller taint");
    assert_seeded_master(&env);
    assert_change_events(&env, 0);
}
