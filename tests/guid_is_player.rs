//! Exact current-model GUID player classification and unchanged ObjectAPI consumers.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::loader::{discover_blizzard_addon_closure_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::SEEDED_LOCAL_CHARACTER_GUID;
use wow_ui_sim::screen::ScreenKind;

// Deliberately contradict GUID prefix heuristics. Kind comes from current records.
const PLAYER_TARGET: &str = "Creature-0-0-0-0-990001-00000001";
const NPC_FOCUS: &str = "Player-990001-00000002";
const PLAYER_FOCUS: &str = "Creature-0-0-0-0-990001-00000003";
const NPC_TARGET: &str = "Player-990001-00000004";
const UNKNOWN: &str = "Player-990001-00000099";

fn fresh_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("fresh GUID classification environment");
    {
        let mut state = env.state().borrow_mut();
        state.current_target = None;
        state.current_focus = None;
        state.party_members.clear();
        state.party_group_active = false;
    }
    env
}

fn supply_target_and_focus(env: &WowLuaEnv) {
    // Use existing public constructors; overwrite only modeled identity/kind fields.
    env.exec(
        "A_Admin.SetTarget('GUID player fixture', 80, 2, false); \
         A_Admin.SetFocus('GUID NPC fixture', 80, 1, true)",
    )
    .expect("existing target/focus constructors");
    let mut state = env.state().borrow_mut();
    let target = state.current_target.as_mut().expect("constructed target");
    target.guid = PLAYER_TARGET.into();
    target.is_player = true;
    let focus = state.current_focus.as_mut().expect("constructed focus");
    focus.guid = NPC_FOCUS.into();
    focus.is_player = false;
}

fn assert_guid(env: &WowLuaEnv, guid: &str, expected: bool) {
    env.exec(&format!(
        r#"
        local query = rawget(C_PlayerInfo, 'GUIDIsPlayer')
        assert(type(query) == 'function', 'missing raw C_PlayerInfo.GUIDIsPlayer')
        local function check(...)
            assert(select('#', ...) == 1, 'GUID classification arity must be one')
            local result = ...
            assert(type(result) == 'boolean', 'GUID classification must be boolean')
            assert(result == {expected}, 'wrong current kind for ' .. {guid:?})
        end
        check(query({guid:?}))
        "#
    ))
    .expect("exact represented GUID classification");
}

#[test]
fn guid_is_player_exact_local_identity_and_closed_world_misses() {
    let env = fresh_env();
    assert_guid(&env, SEEDED_LOCAL_CHARACTER_GUID, true);
    for guid in [
        UNKNOWN,
        "Player-1-00000002",
        "Player-0000-00000002", // absent party1, not a seeded GUID catalog
        "Creature-0-0-0-0-990001-00000099",
        "BNet-0-990001",
        "arbitrary GUID fixture",
        "",
        "player",
        "target",
        " Player-1-00000001",
        "Player-1-00000001 ",
        "player-1-00000001",
    ] {
        assert_guid(&env, guid, false);
    }
}

#[test]
fn guid_is_player_target_and_focus_follow_explicit_kind_not_prefix() {
    let env = fresh_env();
    supply_target_and_focus(&env);
    assert_guid(&env, PLAYER_TARGET, true);
    assert_guid(&env, NPC_FOCUS, false);
    {
        let mut state = env.state().borrow_mut();
        let mut target = state.current_target.as_ref().expect("target").clone();
        target.guid = NPC_TARGET.into();
        target.is_player = false;
        state.current_target = Some(target);
        let mut focus = state.current_focus.as_ref().expect("focus").clone();
        focus.guid = PLAYER_FOCUS.into();
        focus.is_player = true;
        state.current_focus = Some(focus);
    }
    assert_guid(&env, PLAYER_TARGET, false);
    assert_guid(&env, NPC_FOCUS, false);
    assert_guid(&env, NPC_TARGET, false);
    assert_guid(&env, PLAYER_FOCUS, true);
}

#[test]
fn guid_is_player_same_env_kind_updates_and_unit_removal_are_live() {
    let env = fresh_env();
    supply_target_and_focus(&env);
    assert_guid(&env, PLAYER_TARGET, true);
    assert_guid(&env, NPC_FOCUS, false);
    {
        let mut state = env.state().borrow_mut();
        state.current_target.as_mut().expect("target").is_player = false;
        state.current_focus.as_mut().expect("focus").is_player = true;
    }
    assert_guid(&env, PLAYER_TARGET, false);
    assert_guid(&env, NPC_FOCUS, true);
    {
        let mut state = env.state().borrow_mut();
        state.current_target.as_mut().expect("target").is_player = true;
        state.current_focus.as_mut().expect("focus").is_player = false;
    }
    assert_guid(&env, PLAYER_TARGET, true);
    assert_guid(&env, NPC_FOCUS, false);
    env.exec("A_Admin.ClearTarget(); A_Admin.ClearFocus()")
        .expect("existing unit removal helpers");
    assert_guid(&env, PLAYER_TARGET, false);
    assert_guid(&env, NPC_FOCUS, false);
    assert_guid(&env, SEEDED_LOCAL_CHARACTER_GUID, true);
}

#[test]
fn guid_is_player_active_populated_party_lifecycle() {
    let env = fresh_env();
    env.exec("A_Admin.SetPartySize(2)")
        .expect("existing roster constructor");
    let first: String = env
        .eval("return UnitGUID('party1')")
        .expect("live party1 GUID");
    let second: String = env
        .eval("return UnitGUID('party2')")
        .expect("live party2 GUID");
    assert_ne!(first, second, "party slots have distinct exact identities");
    assert_guid(&env, &first, true);
    assert_guid(&env, &second, true);
    env.state().borrow_mut().party_group_active = false;
    assert_guid(&env, &first, false);
    assert_guid(&env, &second, false);
    env.state().borrow_mut().party_group_active = true;
    assert_guid(&env, &first, true);
    assert_guid(&env, &second, true);
    env.exec("A_Admin.SetPartySize(1)")
        .expect("shrink live roster");
    assert_guid(&env, &first, true);
    assert_guid(&env, &second, false);
    // Active flag alone does not populate a slot.
    env.state().borrow_mut().party_members.clear();
    assert!(env.state().borrow().party_group_active);
    assert_guid(&env, &first, false);
    assert_guid(&env, &second, false);
    env.exec("A_Admin.SetPartySize(2)")
        .expect("repopulate live roster");
    assert_guid(&env, &first, true);
    assert_guid(&env, &second, true);
    env.exec("A_Admin.SetPartySize(0)")
        .expect("remove live roster");
    assert_guid(&env, &first, false);
    assert_guid(&env, &second, false);
}

#[test]
fn guid_is_player_does_not_leak_records_between_environments() {
    let first = fresh_env();
    supply_target_and_focus(&first);
    first.exec("A_Admin.SetPartySize(1)").expect("first roster");
    let party: String = first
        .eval("return UnitGUID('party1')")
        .expect("first party GUID");
    assert_guid(&first, PLAYER_TARGET, true);
    assert_guid(&first, &party, true);
    let second = fresh_env();
    assert_guid(&second, PLAYER_TARGET, false);
    assert_guid(&second, &party, false);
    supply_target_and_focus(&second);
    second
        .state()
        .borrow_mut()
        .current_target
        .as_mut()
        .expect("second target")
        .is_player = false;
    assert_guid(&second, PLAYER_TARGET, false);
    assert_guid(&first, PLAYER_TARGET, true);
    first.state().borrow_mut().current_target = None;
    assert_guid(&first, PLAYER_TARGET, false);
    second
        .state()
        .borrow_mut()
        .current_target
        .as_mut()
        .expect("second target")
        .is_player = true;
    assert_guid(&second, PLAYER_TARGET, true);
    assert_guid(&first, PLAYER_TARGET, false);
}

#[test]
fn guid_is_player_requires_string_without_numeric_coercion() {
    let env = fresh_env();
    env.exec(
        r#"
        local query = rawget(C_PlayerInfo, 'GUIDIsPlayer')
        assert(type(query) == 'function', 'missing raw C_PlayerInfo.GUIDIsPlayer')
        local function stringError(...)
            local ok, message = pcall(query, ...)
            assert(not ok and type(message) == 'string', 'required string must error')
            assert(message:find('string', 1, true), 'string validation: ' .. message)
        end
        stringError()
        stringError(nil)
        for _, value in ipairs({0, 901001, false, true, {}, function() end,
            coroutine.create(function() end)}) do
            stringError(value)
        end
        "#,
    )
    .expect("required strict string identity");
}

#[test]
fn guid_is_player_authenticates_real_secrets_before_type_and_lookup() {
    let env = fresh_env();
    supply_target_and_focus(&env);
    env.exec(&format!(
        r#"
        debug.settaintmode(true)
        assert(debug.getstacktaint() == nil, 'untainted setup')
        local query = rawget(C_PlayerInfo, 'GUIDIsPlayer')
        assert(type(query) == 'function', 'missing raw C_PlayerInfo.GUIDIsPlayer')
        local hit = secretwrap({PLAYER_TARGET:?})
        local miss = secretwrap({UNKNOWN:?})
        local invalid = {{secretwrap(901001), secretwrap(false), secretwrap({{}}), secretwrap(nil)}}
        local function check(expected, ...)
            assert(select('#', ...) == 1, 'secret result arity')
            local result = ...
            assert(type(result) == 'boolean' and result == expected, 'secret result kind/value')
        end
        local function stringError(value)
            local ok, message = pcall(query, value)
            assert(not ok and type(message) == 'string', 'invalid string input must error')
            assert(message:find('string', 1, true), 'expected string validation: ' .. message)
            assert(not message:find('untainted caller', 1, true), 'secure input reaches type check')
        end
        assert(issecretvalue(hit) and issecretvalue(miss), 'authentic VM wrappers')
        check(true, query(hit))
        check(false, query(miss))
        for _, value in ipairs(invalid) do
            assert(issecretvalue(value), 'authentic malformed wrapper')
            stringError(value)
        end
        local function denied(value)
            assert(issecretvalue(value), 'secret retained entering tainted caller')
            local ok, message = pcall(query, value)
            assert(not ok and type(message) == 'string', 'tainted secret must error')
            assert(message:find('untainted caller', 1, true), 'auth before type/lookup: ' .. message)
            assert(not message:find('string', 1, true), 'denial must precede string validation')
            assert(issecretvalue(value), 'denial retains original wrapper')
            assert(not pcall(secretunwrap, value), 'tainted caller cannot unwrap')
            assert(debug.getstacktaint() == 'GUIDPlayerProbe', 'denial preserves taint')
        end
        local function addon()
            assert(debug.getstacktaint() == 'GUIDPlayerProbe', 'actual tainted closure')
            denied(hit)
            denied(miss)
            for _, value in ipairs(invalid) do
                denied(value)
            end
            check(true, query({PLAYER_TARGET:?}))
            check(false, query({UNKNOWN:?}))
            stringError(901001)
            assert(debug.getstacktaint() == 'GUIDPlayerProbe', 'ordinary calls preserve taint')
        end
        debug.setobjecttaint(addon, 'GUIDPlayerProbe')
        addon()
        assert(debug.getstacktaint() == nil, 'secure caller restored')
        assert(issecretvalue(hit) and issecretvalue(miss), 'wrapper round trip')
        check(true, query(hit))
        check(false, query(miss))
        "#
    ))
    .expect("AllowedWhenUntainted authentication before malformed parsing or lookup");
}

fn load_object_api_closure(env: &WowLuaEnv) {
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    let addons =
        discover_blizzard_addon_closure_for_screen(&ui, ScreenKind::Game, &["Blizzard_ObjectAPI"]);
    assert!(
        addons.iter().any(|(name, _)| name == "Blizzard_ObjectAPI"),
        "BLOCKER: ordinary Blizzard_ObjectAPI closure root unavailable at {}",
        ui.display()
    );
    for (name, toc) in addons {
        load_object_api_addon(env, &name, &toc);
    }
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "BLOCKER: ordinary ObjectAPI closure Lua errors: {:?}",
        env.state().borrow().lua_errors
    );
    assert_object_api_provenance(env);
}

fn load_object_api_addon(env: &WowLuaEnv, name: &str, toc: &std::path::Path) {
    let result = load_addon(&env.loader_env(), toc)
        .unwrap_or_else(|error| panic!("BLOCKER: ordinary ObjectAPI load {name}: {error}"));
    assert!(
        result.warnings.is_empty(),
        "BLOCKER: ObjectAPI readiness {name}: {:?}",
        result.warnings
    );
    if !result.missing_requirements.is_empty() {
        eprintln!(
            "ObjectAPI nil-probe observations {name}: {:?}",
            result.missing_requirements
        );
    }
}

fn assert_object_api_provenance(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_ObjectAPI'), 'BLOCKER: ObjectAPI not loaded')
        assert(type(CreateFromMixins) == 'function', 'BLOCKER: CreateFromMixins unavailable')
        assert(type(PlayerLocation) == 'table', 'BLOCKER: vendor PlayerLocation unavailable')
        assert(type(PlayerLocationMixin) == 'table', 'BLOCKER: vendor mixin unavailable')
        assert(type(PlayerLocation.CreateFromGUID) == 'function', 'BLOCKER: original CreateFromGUID unavailable')
        assert(type(PlayerLocationMixin.IsValid) == 'function', 'BLOCKER: original IsValid unavailable')
        for _, method in ipairs({PlayerLocation.CreateFromGUID, PlayerLocationMixin.IsValid}) do
            local source = debug.getinfo(method, 'S').source:gsub('\\', '/')
            assert(source:find('Blizzard_ObjectAPI/Mainline/PlayerLocation.lua', 1, true),
                'BLOCKER: wrong original vendor method provenance: ' .. source)
        end
        "#,
    )
    .expect("ordinary whole-addon constructor and IsValid source provenance");
}

#[test]
fn guid_is_player_original_object_api_consumer_accepts_represented_player() {
    let env = fresh_env();
    supply_target_and_focus(&env);
    load_object_api_closure(&env);
    env.exec(&format!(
        r#"
        local location = PlayerLocation:CreateFromGUID({PLAYER_TARGET:?})
        assert(location.IsValid == PlayerLocationMixin.IsValid, 'original full IsValid dispatch')
        assert(location:GetGUID() == {PLAYER_TARGET:?} and location:IsGUID(), 'original constructor')
        local ok, result = pcall(location.IsValid, location)
        assert(ok, 'original player consumer call failed: ' .. tostring(result))
        assert(result == true, 'represented player must return true, not merely truthy')
        "#
    ))
    .expect("unchanged whole vendor consumer must accept nonplayer-looking player GUID");
}

#[test]
fn account_guid_siblings_project_id_and_current_player_identity() {
    let env = fresh_env();
    env.exec(
        r#"
        local getID = C_AccountInfo.GetIDFromBattleNetAccountGUID
        local related = C_AccountInfo.IsGUIDRelatedToLocalAccount
        assert(select('#', getID('Player-1-2')) == 1 and getID('Player-1-2') == nil)
        assert(getID('BNetAccount-0-0000002A') == 42)
        assert(getID('BNetAccount-7-000000010000002A') == 42)
        assert(getID('BNetAccount-') == 0 and getID('BNetAccount-foo') == 0)
        assert(getID('BNetAccount-1X2') == 2)
        assert(related(UnitGUID('player')) == true)
        assert(related('Player-999-00000001') == false)
        assert(related('BNetAccount-0-0000002A') == false)
        for _, fn in ipairs({getID, related}) do
            local ok, err = pcall(fn, false)
            assert(not ok and err:find('Usage:', 1, true))
        end
        "#,
    )
    .unwrap();
}

#[test]
fn account_guid_predicate_accepts_permissive_native_inputs() {
    let env = fresh_env();
    env.exec(
        r#"
        local predicate = C_AccountInfo.IsGUIDBattleNetAccountType
        for _, guid in ipairs({'BNetAccount-1-0000002A', 'BNetAccount-', 'BNetAccount-foo', 'BNetAccount-1X2', 'bnetaccount-1-2'}) do
            assert(predicate(guid, {}) == true, guid)
        end
        for _, guid in ipairs({'BNetAccount', 'BNetAccount-288230376151711744-1', 'Player-1-2', 'junk', 42}) do
            assert(predicate(guid) == false, tostring(guid))
        end
        for _, value in ipairs({false, {}}) do
            local ok, err = pcall(predicate, value)
            assert(not ok and err:find('Usage: local isBNet = C_AccountInfo.IsGUIDBattleNetAccountType(guid)', 1, true))
        end
        local ok, err = pcall(predicate)
        assert(not ok and err:find('Usage: local isBNet = C_AccountInfo.IsGUIDBattleNetAccountType(guid)', 1, true))
        debug.settaintmode(true)
        local secret = secretwrap('BNetAccount-1-2')
        assert(predicate(secret) == true)
        local function addon()
            local accepted, message = pcall(predicate, secret)
            assert(not accepted and message:find('Secret values are only allowed during untainted execution', 1, true))
        end
        debug.setobjecttaint(addon, 'AccountGuidProbe')
        addon()
        "#,
    )
    .unwrap();
}

#[test]
fn guid_is_player_original_consumer_preserves_actual_account_predicate_for_misses() {
    let env = fresh_env();
    supply_target_and_focus(&env);
    load_object_api_closure(&env);
    // Direct classification and consumer/account diagnostics are separate evidence.
    assert_guid(&env, NPC_FOCUS, false);
    assert_guid(&env, UNKNOWN, false);
    let diagnostics: String = env.eval(&format!(
        r#"
        local diagnostics = {{}}
        local accountLocation = PlayerLocation:CreateFromGUID('BNetAccount-1-0000002A')
        assert(accountLocation:IsValid() == true, 'PlayerLocation accepts the native BNet GUID kind')
        local namespace = C_AccountInfo
        assert(type(namespace) == 'table', 'BLOCKER: ordinary C_AccountInfo unavailable')
        local account = namespace.IsGUIDBattleNetAccountType
        assert(type(account) == 'function', 'BLOCKER: ordinary account predicate unavailable')
        local info = debug.getinfo(account, 'S')
        diagnostics[#diagnostics + 1] = table.concat({{'GUID account-provider', info.what, info.source,
            'raw-provider', type(rawget(namespace, 'IsGUIDBattleNetAccountType'))}}, '\t')
        for _, guid in ipairs({{{NPC_FOCUS:?}, {UNKNOWN:?}, 'BNetAccount-1-0000002A'}}) do
            local accountOK, accountResult = pcall(account, guid)
            diagnostics[#diagnostics + 1] = table.concat({{'GUID account-return', guid,
                tostring(accountOK), type(accountResult), tostring(accountResult)}}, '\t')
            assert(accountOK, 'BLOCKER: ordinary account predicate failed for ' .. guid .. ': ' .. tostring(accountResult))
            local location = PlayerLocation:CreateFromGUID(guid)
            assert(location.IsValid == PlayerLocationMixin.IsValid, 'original full IsValid dispatch')
            local ok, result = pcall(location.IsValid, location)
            diagnostics[#diagnostics + 1] = table.concat({{'GUID consumer-return', guid,
                tostring(ok), type(result), tostring(result)}}, '\t')
            assert(ok, 'BLOCKER: original miss consumer failed: ' .. tostring(result))
            assert(result == accountResult, 'unchanged account operand must determine false-classification consumer result')
            if guid == 'BNetAccount-1-0000002A' then
                assert(accountResult == true, 'native BNet kind reaches PlayerLocation')
            else
                assert(accountResult == false, 'non-BNet misses remain false')
            end
            assert(namespace.IsGUIDBattleNetAccountType == account, 'account provider must remain unchanged')
            assert(debug.getinfo(account, 'S').source == info.source, 'account provenance retained')
        end
        return table.concat(diagnostics, '\n')
        "#
    ))
    .expect("original consumer account-path result is diagnostic, not native GUID classification proof");
    eprintln!("{diagnostics}");
}
