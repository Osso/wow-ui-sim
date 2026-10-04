//! 12.1.0 unit secrecy: identity-restricted unit APIs return secrets for
//! secret identities, UnitIsCharmed/UnitIsPossessed follow aura secrecy except
//! for player-controlled tokens, UnitName stops being secret for players in an
//! active PvP match, and GetGuildInfo rejects compound unit tokens.
#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

const NPC_GUID: &str = "Creature-0-0-0-0-4242-0000000001";

/// Two active party members, an NPC target, and Lua helpers that classify
/// every return of a call as secret or public.
fn unit_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create unit secrecy environment");
    {
        let mut state = env.state().borrow_mut();
        assert!(state.party_members.len() >= 2, "fixture needs two members");
        state.party_members.truncate(2);
        state.party_group_active = true;
    }
    env.exec(
        r#"
        A_Admin.SetTarget('Arena Dummy', 80, 1, true)
        Party1GUID = UnitGUID('party1')
        assert(Party1GUID ~= nil and UnitGUID('party2') ~= nil)

        -- Returns (count, secret count, non-nil count) for one call.
        function ClassifyReturns(...)
            local count, secret, present = select('#', ...), 0, 0
            for index = 1, count do
                local value = select(index, ...)
                -- Comparing a secret is an addon-caller error; secrets are present.
                if issecretvalue(value) then
                    secret, present = secret + 1, present + 1
                elseif value ~= nil then
                    present = present + 1
                end
            end
            return count, secret, present
        end
        "#,
    )
    .expect("seed party and target identities");
    env.state()
        .borrow_mut()
        .current_target
        .as_mut()
        .expect("target fixture")
        .guid = NPC_GUID.to_string();
    env
}

fn classify_secret(env: &WowLuaEnv, guid: &str) {
    env.state()
        .borrow_mut()
        .identity_secret_guids
        .insert(guid.to_string());
}

fn party1_guid(env: &WowLuaEnv) -> String {
    env.eval("return Party1GUID").expect("party1 GUID")
}

/// Every API named by the 12.1.0 identity-secrecy rows, called with `U`.
const IDENTITY_CALLS: &str = r#"
IdentityCalls = {
    UnitClass = function(u) return UnitClass(u) end,
    UnitClassBase = function(u) return UnitClassBase(u) end,
    UnitIsOwnerOrControllerOfUnit = function(u) return UnitIsOwnerOrControllerOfUnit('player', u) end,
    UnitIsOwnerOrControllerOfUnitControlled = function(u) return UnitIsOwnerOrControllerOfUnit(u, 'player') end,
    UnitSex = function(u) return UnitSex(u) end,
    UnitSexBase = function(u) return UnitSexBase(u) end,
    UnitPhaseReason = function(u) return UnitPhaseReason(u) end,
    UnitGroupRolesAssigned = function(u) return UnitGroupRolesAssigned(u) end,
    UnitGroupRolesAssignedEnum = function(u) return UnitGroupRolesAssignedEnum(u) end,
    UnitIsRaidOfficer = function(u) return UnitIsRaidOfficer(u) end,
    UnitInRaid = function(u) return UnitInRaid(u) end,
    UnitIsPVP = function(u) return UnitIsPVP(u) end,
    UnitRace = function(u) return UnitRace(u) end,
    UnitIsGroupLeader = function(u) return UnitIsGroupLeader(u) end,
    UnitIsGroupAssistant = function(u) return UnitIsGroupAssistant(u) end,
    UnitLeadsAnyGroup = function(u) return UnitLeadsAnyGroup(u) end,
    UnitGetAvailableRoles = function(u) return UnitGetAvailableRoles(u) end,
    GetInspectSpecialization = function(u) return GetInspectSpecialization(u) end,
}
-- UnitPhaseReason has no modeled reason: its single return stays nil.
NilOnly = { UnitPhaseReason = true }
"#;

/// For (api, unit, expected) rows, assert every non-nil return is secret
/// exactly when expected, from both the secure top level and an addon closure.
fn assert_identity_rows(env: &WowLuaEnv, rows: &[(&str, bool)]) {
    for (unit, expected) in rows {
        let check = format!(
            r#"
            local unit, expected = '{unit}', {expected}
            local function check(label)
                for api, call in pairs(IdentityCalls) do
                    local count, secret, present = ClassifyReturns(call(unit))
                    assert(count == BaselineArity[api],
                        label .. ' ' .. api .. ' arity ' .. count)
                    if not NilOnly[api] then
                        assert(present > 0, label .. ' ' .. api .. ' returned nothing')
                    end
                    assert(secret == (expected and present or 0),
                        label .. ' ' .. api .. '(' .. unit .. ') secret=' .. secret)
                end
            end
            check('secure')
            local function addonCaller()
                assert(debug.getstacktaint() == 'UnitSecrecyFixtureAddon')
                check('addon')
            end
            debug.setobjecttaint(addonCaller, 'UnitSecrecyFixtureAddon')
            addonCaller()
            "#
        );
        env.exec(&check)
            .unwrap_or_else(|err| panic!("identity row {unit}={expected}: {err}"));
    }
}

#[test]
fn identity_restricted_unit_apis_return_secrets_for_secret_identities() {
    let env = unit_env();
    env.exec(IDENTITY_CALLS).expect("install call table");
    env.exec(
        r#"
        BaselineArity = {}
        for api, call in pairs(IdentityCalls) do
            local count, secret = ClassifyReturns(call('party1'))
            assert(secret == 0, api .. ' is public before classification')
            BaselineArity[api] = count
        end
        "#,
    )
    .expect("public baseline for every API");
    assert_identity_rows(&env, &[("party1", false), ("player", false)]);

    let guid = party1_guid(&env);
    classify_secret(&env, &guid);
    assert_identity_rows(
        &env,
        &[("party1", true), ("party2", false), ("player", false)],
    );

    env.state().borrow_mut().identity_secret_guids.remove(&guid);
    assert_identity_rows(&env, &[("party1", false)]);
}

#[test]
fn secret_identity_values_keep_payload_and_block_addon_reads() {
    let env = unit_env();
    let guid = party1_guid(&env);
    env.exec("PublicClass = {UnitClass('party1')}; PublicRace = {UnitRace('party1')}")
        .expect("public payloads");
    classify_secret(&env, &guid);
    env.exec(
        r#"
        local name, file, id = UnitClass('party1')
        assert(secretunwrap(name) == PublicClass[1])
        assert(secretunwrap(file) == PublicClass[2])
        assert(secretunwrap(id) == PublicClass[3])
        local race = UnitRace('party1')
        assert(secretunwrap(race) == PublicRace[1])
        local function addonCaller()
            local _, classFile = UnitClass('party1')
            assert(not pcall(string.upper, classFile))
            assert(not pcall(function() if UnitInRaid('party1') then end end))
        end
        debug.setobjecttaint(addonCaller, 'UnitSecrecyFixtureAddon')
        addonCaller()
        "#,
    )
    .expect("secure callers read payloads; addon callers cannot use secrets");
}

#[test]
fn charm_and_possession_follow_aura_secrecy_except_player_controlled_tokens() {
    let env = unit_env();
    let rows = [
        ("player", false, false),
        ("pet", false, false),
        ("vehicle", false, false),
        ("target", false, false),
        ("party1", false, false),
        ("player", true, false),
        ("pet", true, false),
        ("vehicle", true, false),
        ("target", true, true),
        ("party1", true, true),
    ];
    for (unit, auras_secret, expected) in rows {
        env.state().borrow_mut().unit_auras_restricted = auras_secret;
        env.exec(&format!(
            r#"
            for _, api in ipairs({{UnitIsCharmed, UnitIsPossessed}}) do
                local count, secret, present = ClassifyReturns(api('{unit}'))
                assert(count == 1 and present == 1, 'single boolean return')
                assert(secret == ({expected} and 1 or 0), 'possession secrecy')
            end
            local function addonCaller()
                local count, secret = ClassifyReturns(UnitIsCharmed('{unit}'))
                assert(secret == ({expected} and 1 or 0))
            end
            debug.setobjecttaint(addonCaller, 'UnitSecrecyFixtureAddon')
            addonCaller()
            "#
        ))
        .unwrap_or_else(|err| panic!("{unit} auras_secret={auras_secret}: {err}"));
    }
}

#[test]
fn unit_name_is_public_for_players_in_active_pvp_match() {
    let env = unit_env();
    let guid = party1_guid(&env);
    classify_secret(&env, &guid);
    classify_secret(&env, NPC_GUID);
    // (pvp match active, unit, UnitName secret, UnitNameUnmodified secret)
    let rows = [
        (false, "party1", true, true),
        (false, "target", true, true),
        (false, "party2", false, false),
        (true, "party1", false, true),
        (true, "target", true, true),
        (true, "party2", false, false),
        (true, "player", false, false),
    ];
    for (pvp, unit, name_secret, unmodified_secret) in rows {
        env.state().borrow_mut().pvp_match_active = pvp;
        env.exec(&format!(
            r#"
            local name = UnitName('{unit}')
            assert(name ~= nil and select('#', UnitName('{unit}')) == 1)
            assert(issecretvalue(name) == {name_secret}, 'UnitName secrecy')
            assert(issecretvalue(UnitNameUnmodified('{unit}')) == {unmodified_secret},
                'UnitNameUnmodified keeps identity rules')
            if not {name_secret} then
                assert(name == UnitName('{unit}'))
                assert(type(name) == 'string')
            end
            "#
        ))
        .unwrap_or_else(|err| panic!("pvp={pvp} {unit}: {err}"));
    }
}

#[test]
fn get_guild_info_rejects_compound_unit_tokens() {
    let env = unit_env();
    {
        let mut state = env.state().borrow_mut();
        state.world.guild_name = Some("Sim Guild".into());
        state.world.guild_rank = Some("Officer".into());
    }
    env.exec(
        r#"
        for _, unit in ipairs({'player', 'target', 'party1'}) do
            local name, rank = GetGuildInfo(unit)
            assert(name == 'Sim Guild' and rank == 'Officer', unit .. ' accepted')
        end
        for _, unit in ipairs({'targettarget', 'party1target', 'focustarget',
                               'TargetTarget', 'mouseovertarget'}) do
            assert(select('#', GetGuildInfo(unit)) == 0, unit .. ' rejected')
        end
        "#,
    )
    .expect("compound tokens return nothing, simple tokens resolve");
}
