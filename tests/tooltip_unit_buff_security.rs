//! B79 exact342: inferred, bounded GetUnitBuff lookup/input contract only.
//! Inputs only; main owns compiled RED/GREEN and acceptance accounting.
#![cfg(feature = "retail-12-0-5")]

use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string, wrap_secret,
};
use rilua::{LuaApiMut, Val};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{AuraInfo, PartyMember};

const SECRET_NAMES: [&str; 9] = [
    "UBUnit",
    "UBOne",
    "UBTwo",
    "UBFilter",
    "UBNil",
    "UBBool",
    "UBPrivate",
    "UBSecretTable",
    "UBSecretFrame",
];

fn aura(id: i32, name: &str, helpful: bool, player_source: bool) -> AuraInfo {
    AuraInfo {
        name: name.into(),
        spell_id: 19750,
        icon: 135987,
        duration: 3600.0,
        expiration_time: 3600.0,
        applications: 2,
        source_unit: if player_source { "player" } else { "party2" }.into(),
        is_helpful: helpful,
        is_raid: true,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: player_source,
        dispel_type: None,
        aura_instance_id: id,
    }
}

fn member(name: &str, buffs: Vec<AuraInfo>) -> PartyMember {
    PartyMember {
        name: name.into(),
        connected: true,
        class_index: 2,
        level: 80,
        health: 100,
        health_max: 100,
        power: 100,
        power_max: 100,
        power_type: 0,
        power_type_name: "MANA".into(),
        is_leader: false,
        dead_since: None,
        buffs,
        debuffs: vec![aura(901, "Party harmful sentinel", false, true)],
    }
}

const ASSERTIONS: &str = r#"
    UBQuery = C_TooltipInfo.GetUnitBuff
    assert(type(UBQuery) == 'function', 'real registered API')
    UBFrame = CreateFrame('Frame')
    UBFrame:SetAlpha(0.625)
    UBFrame.marker = 342
    UBTable = {marker = 'PRIVATE-UnitBuff', count = 37}
    function UBProperties()
        assert(UBFrame:GetObjectType() == 'Frame')
        assert(UBFrame:GetAlpha() == 0.625 and UBFrame.marker == 342)
        assert(UBTable.marker == 'PRIVATE-UnitBuff' and UBTable.count == 37)
    end
    function UBPublic(value)
        assert(not issecretvalue(value), 'current public DTO convention only')
        if type(value) == 'table' then
            for key, child in pairs(value) do
                assert(not issecretvalue(key))
                UBPublic(child)
            end
        end
    end
    function UBData(...)
        assert(select('#', ...) == 1, 'one result, including misses')
        local data = ...
        assert(type(data) == 'table' and type(data.lines) == 'table')
        assert(data.type == Enum.TooltipDataType.UnitAura)
        local count = 0
        for key in pairs(data) do
            assert(key == 'type' or key == 'lines', 'current emptyTooltip shape')
            count = count + 1
        end
        assert(count == 2)
        UBPublic(data)
        return data
    end
    function UBCheck(name, ...)
        local data = UBData(UBQuery(...))
        if not name then
            assert(next(data.lines) == nil, 'true line-empty miss')
            return data
        end
        assert(#data.lines == 3, 'name/duration/description payload')
        assert(data.lines[1].type == Enum.TooltipDataLineType.SpellName)
        assert(data.lines[1].leftText == name, 'concrete host aura identity')
        assert(data.lines[2].type == Enum.TooltipDataLineType.SpellName)
        assert(data.lines[2].leftText == '1 hr', '3600s fixture; existing builder limitation')
        assert(data.lines[3].type == Enum.TooltipDataLineType.SpellDescription)
        assert(type(data.lines[3].leftText) == 'string' and #data.lines[3].leftText > 0)
        assert(data.lines[3].wrapText == true)
        for i=1,2 do
            local color = data.lines[i].leftColor
            assert(type(color.GetRGBA) == 'function')
            local function channels(...)
                assert(select('#', ...) == 4)
                for _, value in ipairs({...}) do
                    assert(type(value) == 'number' and not issecretvalue(value))
                end
                assert(select(4, ...) == 1)
            end
            channels(color:GetRGBA())
        end
        return data
    end
    function UBRecovery()
        UBCheck('Player outside source', 'player', 1)
        UBCheck('Party one outside source', 'party1', 1)
        UBCheck('Party two player source', 'party2', 1)
        UBProperties()
    end
    function UBTainted(probe)
        assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'UnitBuffFixture')
            probe()
            assert(debug.getstacktaint() == 'UnitBuffFixture', 'caller taint preserved')
        end
        debug.setobjecttaint(addon, 'UnitBuffFixture')
        addon()
        assert(issecure(), 'outer context restored')
    end
    function UBReject(denial, ...)
        local before = debug.getstacktaint()
        local ok, err = pcall(UBQuery, ...)
        assert(not ok, 'expected rejection')
        assert(type(err) == 'string' and #err > 0)
        assert(string.find(err, 'C_TooltipInfo.GetUnitBuff', 1, true), 'API error context')
        assert(not string.find(err, 'PRIVATE-UnitBuff', 1, true), 'no private payload leak')
        local gate = string.find(err, 'requires an untainted caller', 1, true)
        if denial then assert(gate, 'authentication before parsing or lookup')
        else assert(not gate, 'ordinary type validation after secure authentication') end
        assert(debug.getstacktaint() == before)
        UBRecovery()
        assert(debug.getstacktaint() == before, 'recovery preserves caller taint')
    end
"#;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("real tooltip namespace");
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![
            aura(101, "Player harmful sentinel", false, true),
            aura(102, "Player outside source", true, false),
            aura(103, "Player own source", true, true),
        ];
        state.party_members = vec![
            member(
                "Buff fixture one",
                vec![
                    aura(201, "Party one outside source", true, false),
                    aura(202, "Party one player source", true, true),
                ],
            ),
            member(
                "Buff fixture two",
                vec![aura(201, "Party two player source", true, true)],
            ),
        ];
    }
    env.exec(ASSERTIONS).expect("assertions, no query override");
    env
}

fn publish_secret(lua: &mut impl LuaApiMut, name: &str, value: Val) {
    lua.state_mut().push(value);
    let result = lua.set_global_val(name, value);
    lua.state_mut().pop();
    result.expect("root authentic host wrapper");
}

fn wrap_original(lua: &mut impl LuaApiMut, original: &str, name: &str) {
    let value = lua.get_global_val(original);
    let Val::Table(reference) = value else {
        panic!("original table/frame")
    };
    if original == "UBFrame" {
        assert!(
            lua.state_mut()
                .gc
                .tables
                .get(reference)
                .unwrap()
                .backing()
                .is_some()
        );
    }
    lua.state_mut().push(value);
    let secret = wrap_secret(lua.state_mut(), value).expect("wrap real original");
    publish_secret(lua, name, secret);
    lua.state_mut().pop();
}

fn secret_env() -> WowLuaEnv {
    let env = fixture_env();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).unwrap();
    for (name, text) in [
        ("UBUnit", "party1"),
        ("UBFilter", "HELPFUL|PLAYER"),
        ("UBPrivate", "PRIVATE-UnitBuff-Payload"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        publish_secret(&mut *lua, name, value);
    }
    for (name, number) in [("UBOne", 1.0), ("UBTwo", 2.0)] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        publish_secret(&mut *lua, name, value);
    }
    let value = wrap_secret(lua.state_mut(), Val::Nil).unwrap();
    publish_secret(&mut *lua, "UBNil", value);
    let value = wrap_host_secret_bool(lua.state_mut(), false);
    publish_secret(&mut *lua, "UBBool", value);
    wrap_original(&mut *lua, "UBTable", "UBSecretTable");
    wrap_original(&mut *lua, "UBFrame", "UBSecretFrame");
    drop(lua);
    env.exec("UBSecrets = {UBUnit, UBOne, UBTwo, UBFilter, UBNil, UBBool, UBPrivate, UBSecretTable, UBSecretFrame}; for _, value in ipairs(UBSecrets) do assert(issecretvalue(value)) end").unwrap();
    env
}

fn snapshot_roots(env: &WowLuaEnv) -> Vec<(Val, u64)> {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let Val::Table(list) = lua.get_global_val("UBSecrets") else {
        panic!("root list")
    };
    SECRET_NAMES
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let value = lua.get_global_val(name);
            let Val::Userdata(reference) = value else {
                panic!("authentic wrapper {name}")
            };
            assert!(rilua::table_security::is_secret_value(
                lua.state_mut(),
                value
            ));
            assert_eq!(
                lua.state_mut()
                    .gc
                    .tables
                    .get(list)
                    .unwrap()
                    .get_int(index as i64 + 1),
                value,
                "host identity; no tainted secret BOOL equality"
            );
            let sequence = lua
                .state_mut()
                .gc
                .userdata
                .get(reference)
                .unwrap()
                .alloc_seq();
            (value, sequence)
        })
        .collect()
}

fn snapshot_auras(env: &WowLuaEnv) -> Vec<Vec<AuraInfo>> {
    let state = env.state().borrow();
    let mut rows = vec![state.player.buffs.clone()];
    for member in &state.party_members {
        rows.push(member.buffs.clone());
        rows.push(member.debuffs.clone());
    }
    rows
}

fn assert_auras_unchanged(actual: &[Vec<AuraInfo>], expected: &[Vec<AuraInfo>]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(actual.len(), expected.len());
        for (a, b) in actual.iter().zip(expected) {
            assert_eq!(
                (
                    &a.name,
                    a.spell_id,
                    a.icon,
                    a.duration,
                    a.expiration_time,
                    a.applications,
                    &a.source_unit,
                    a.aura_instance_id
                ),
                (
                    &b.name,
                    b.spell_id,
                    b.icon,
                    b.duration,
                    b.expiration_time,
                    b.applications,
                    &b.source_unit,
                    b.aura_instance_id
                )
            );
            assert_eq!(
                (
                    a.is_helpful,
                    a.is_raid,
                    a.is_nameplate_only,
                    a.is_stealable,
                    a.can_apply_aura,
                    a.is_from_player_or_player_pet,
                    &a.dispel_type
                ),
                (
                    b.is_helpful,
                    b.is_raid,
                    b.is_nameplate_only,
                    b.is_stealable,
                    b.can_apply_aura,
                    b.is_from_player_or_player_pet,
                    &b.dispel_type
                )
            );
        }
    }
}

fn public_probe(script: &str) {
    let env = fixture_env();
    let rows = snapshot_auras(&env);
    env.exec(script).expect("real GetUnitBuff public probe");
    assert_auras_unchanged(&snapshot_auras(&env), &rows);
}

fn secret_probe(script: &str) {
    let env = secret_env();
    let roots = snapshot_roots(&env);
    let rows = snapshot_auras(&env);
    env.exec(script).expect("real GetUnitBuff security probe");
    assert_eq!(snapshot_roots(&env), roots, "wrapper identity/secrecy");
    assert_auras_unchanged(&snapshot_auras(&env), &rows);
}

#[test]
fn distinct_player_party1_party2_content_uses_requested_live_unit() {
    public_probe("UBRecovery()");
}

#[test]
fn positive_one_based_index_addresses_filtered_not_raw_player_vector() {
    public_probe(
        "UBCheck('Player outside source', 'player', 1, 'HELPFUL'); \
         UBCheck('Player own source', 'player', 2, 'HELPFUL'); \
         UBCheck('Party one player source', 'party1', 2, 'HELPFUL')",
    );
}

#[test]
fn omitted_and_public_nil_filter_default_to_helpful() {
    public_probe(
        "UBCheck('Player outside source', 'player', 1); \
         UBCheck('Player outside source', 'player', 1, nil); \
         UBCheck('Party one outside source', 'party1', 1, nil)",
    );
}

#[test]
fn player_filter_excludes_other_sources_before_indexing() {
    public_probe(
        "UBCheck('Player own source', 'player', 1, 'HELPFUL|PLAYER'); \
         UBCheck(nil, 'player', 2, 'HELPFUL|PLAYER'); \
         UBCheck('Party one player source', 'party1', 1, 'HELPFUL|PLAYER'); \
         UBCheck(nil, 'party1', 2, 'HELPFUL|PLAYER'); \
         UBCheck('Party two player source', 'party2', 1, 'HELPFUL|PLAYER')",
    );
}

#[test]
fn blocked_filter_reindexes_and_same_instance_on_other_unit_remains_visible() {
    public_probe(
        r#"
        C_UnitAuras.AddBlockedAura('party1', 201)
        UBCheck('Party one player source', 'party1', 1)
        UBCheck(nil, 'party1', 2)
        UBCheck('Party two player source', 'party2', 1)
        UBCheck('Player outside source', 'player', 1)
        C_UnitAuras.AddBlockedAura('party1', 202)
        UBCheck(nil, 'party1', 1, 'HELPFUL|PLAYER')
        C_UnitAuras.AddBlockedAura('player', 102)
        UBCheck('Player own source', 'player', 1)
        UBCheck(nil, 'player', 2)
        "#,
    );
}

#[test]
fn missing_or_nonpositive_filtered_index_returns_current_empty_shape() {
    public_probe(
        r#"
        for _, index in ipairs({0, -1, -37, 3, 999}) do
            UBCheck(nil, 'player', index)
            UBCheck(nil, 'party1', index)
        end
        UBCheck(nil, 'party2', 2)
        "#,
    );
}

#[test]
fn unsupported_units_never_fall_back_to_player() {
    public_probe(
        r#"
        for _, unit in ipairs({'target', 'pet', 'focus', 'raid1', 'party3', 'missing', ''}) do
            UBCheck(nil, unit, 1)
        end
        UBRecovery()
        "#,
    );
}

#[test]
fn live_host_mutation_removal_and_environment_isolation() {
    let env = fixture_env();
    let other = fixture_env();
    env.exec("UBOld = UBCheck('Party one outside source', 'party1', 1)")
        .unwrap();
    env.state().borrow_mut().party_members[0].buffs[0].name = "Live party replacement".into();
    env.exec(
        "UBCheck('Live party replacement', 'party1', 1); \
         UBCheck('Party two player source', 'party2', 1); \
         UBCheck('Player outside source', 'player', 1); \
         assert(UBOld.lines[1].leftText == 'Party one outside source')",
    )
    .unwrap();
    env.state().borrow_mut().party_members[0].buffs.remove(0);
    env.exec("UBCheck('Party one player source', 'party1', 1); UBCheck(nil, 'party1', 2)")
        .unwrap();
    env.state().borrow_mut().party_members[0].buffs.clear();
    env.exec("UBCheck(nil, 'party1', 1); UBCheck('Party two player source', 'party2', 1)")
        .unwrap();
    env.state().borrow_mut().player.buffs[2].name = "Live player replacement".into();
    env.exec("UBCheck('Live player replacement', 'player', 2)")
        .unwrap();
    env.state().borrow_mut().player.buffs.clear();
    env.exec("UBCheck(nil, 'player', 1); UBCheck('Party two player source', 'party2', 1)")
        .unwrap();
    other.exec("UBRecovery(); UBCheck('Player own source', 'player', 2); UBCheck('Party one player source', 'party1', 2)").unwrap();
}

#[test]
fn fresh_payload_and_miss_tables_do_not_alias_host_or_other_results() {
    public_probe(
        r#"
        local first = UBCheck('Party one outside source', 'party1', 1)
        local second = UBCheck('Party one outside source', 'party1', 1)
        assert(not rawequal(first, second) and not rawequal(first.lines, second.lines))
        assert(not rawequal(first.lines[1], second.lines[1]))
        first.lines[1].leftText = 'mutated result'
        first.lines[2] = nil
        first.extra = true
        assert(second.lines[1].leftText == 'Party one outside source')
        UBRecovery()
        local miss = UBCheck(nil, 'party1', 99)
        local another = UBCheck(nil, 'party1', 99)
        assert(not rawequal(miss, another) and not rawequal(miss.lines, another.lines))
        miss.lines[1] = {leftText = 'not an aura'}
        assert(next(another.lines) == nil)
        UBCheck(nil, 'party1', 99)
        "#,
    );
}

#[test]
fn public_tainted_calls_keep_content_filters_and_caller_taint() {
    public_probe(
        r#"
        UBTainted(function()
            UBRecovery()
            UBCheck('Player own source', 'player', 1, 'HELPFUL|PLAYER')
            UBCheck('Party one player source', 'party1', 1, 'HELPFUL|PLAYER')
            UBCheck(nil, 'missing', 1)
            UBCheck(nil, 'player', 0)
        end)
        UBRecovery()
        "#,
    );
}

#[test]
fn secure_secret_unit_selects_actual_party_not_player() {
    secret_probe(
        "assert(issecure()); UBCheck('Party one outside source', UBUnit, 1); \
         UBCheck('Party one player source', UBUnit, 2); assert(issecure())",
    );
}

#[test]
fn secure_secret_index_selects_concrete_filtered_row() {
    secret_probe(
        "UBCheck('Player outside source', 'player', UBOne); \
         UBCheck('Player own source', 'player', UBTwo); \
         UBCheck('Party one player source', 'party1', UBTwo); assert(issecure())",
    );
}

#[test]
fn secure_secret_filter_and_nil_apply_filter_and_default() {
    secret_probe(
        "UBCheck('Player own source', 'player', 1, UBFilter); \
         UBCheck('Party one player source', 'party1', 1, UBFilter); \
         UBCheck('Party one outside source', 'party1', 1, UBNil); assert(issecure())",
    );
}

#[test]
fn secure_combined_secret_selectors_return_meaningful_content_and_true_miss() {
    secret_probe(
        "UBCheck('Party one player source', UBUnit, UBOne, UBFilter); \
         UBCheck(nil, UBUnit, UBTwo, UBFilter); \
         UBCheck('Party one player source', UBUnit, UBTwo, UBNil); assert(issecure())",
    );
}

#[test]
fn tainted_secret_each_position_denied_before_lookup_or_public_parse() {
    secret_probe(
        r#"
        UBTainted(function()
            UBReject(true, UBUnit, 1)
            UBReject(true, UBUnit, 999)
            UBReject(true, UBUnit, false)
            UBReject(true, 'party1', UBOne)
            UBReject(true, 'missing', UBOne)
            UBReject(true, false, UBOne)
            UBReject(true, 'party1', 1, UBFilter)
            UBReject(true, 'missing', 999, UBFilter)
            UBReject(true, false, false, UBNil)
        end)
        UBRecovery()
        "#,
    );
}

#[test]
fn tainted_malformed_secret_payload_denied_before_type_or_state_short_circuit() {
    secret_probe(
        r#"
        UBTainted(function()
            for _, value in ipairs({UBBool, UBSecretTable, UBSecretFrame, UBPrivate}) do
                UBReject(true, value, false)
                UBReject(true, false, value)
                UBReject(true, 'missing', 999, value)
                assert(issecretvalue(value))
            end
        end)
        UBProperties()
        "#,
    );
}

#[test]
fn secure_wrong_types_and_public_wrong_types_reject_with_recovery() {
    secret_probe(
        r#"
        for _, value in ipairs({UBBool, UBSecretTable, UBSecretFrame}) do
            UBReject(false, value, 1)
            UBReject(false, 'party1', value)
            UBReject(false, 'party1', 1, value)
        end
        UBReject(false, 'party1', UBPrivate)
        UBTainted(function()
            for _, value in ipairs({false, UBTable, UBFrame}) do
                UBReject(false, value, 1)
                UBReject(false, 'party1', value)
                UBReject(false, 'party1', 1, value)
            end
        end)
        UBProperties()
        "#,
    );
}

#[test]
fn rooted_host_secret_gc_preserves_identity_secrecy_trust_and_fixture_state() {
    secret_probe(
        r#"
        collectgarbage('collect')
        UBCheck('Party one player source', UBUnit, UBOne, UBFilter)
        UBTainted(function()
            for _, value in ipairs(UBSecrets) do
                UBReject(true, 'missing', 999, value)
                collectgarbage('collect')
                assert(issecretvalue(value))
                UBRecovery()
            end
        end)
        collectgarbage('collect')
        UBCheck('Party one player source', UBUnit, UBTwo, UBNil)
        for _, value in ipairs(UBSecrets) do assert(issecretvalue(value)) end
        assert(issecure())
        UBProperties()
        "#,
    );
}
