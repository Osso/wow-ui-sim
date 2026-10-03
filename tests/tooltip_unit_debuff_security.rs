//! B81 exact347: inferred, bounded GetUnitDebuff lookup/input contract only.
//! Inputs only; main owns compiled RED/GREEN and acceptance accounting.
#![cfg(feature = "retail-12-0-5")]

use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string, wrap_secret,
};
use rilua::{LuaApiMut, Val};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{AuraInfo, PartyMember};

const SECRET_NAMES: [&str; 9] = [
    "UDUnit",
    "UDOne",
    "UDTwo",
    "UDFilter",
    "UDNil",
    "UDBool",
    "UDPrivate",
    "UDSecretTable",
    "UDSecretFrame",
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

fn member(name: &str, debuffs: Vec<AuraInfo>) -> PartyMember {
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
        buffs: vec![aura(901, "Wrong party buffs store", false, true)],
        debuffs,
    }
}

const ASSERTIONS: &str = r#"
    UDQuery = C_TooltipInfo.GetUnitDebuff
    assert(type(UDQuery) == 'function', 'real registered API')
    UDFrame = CreateFrame('Frame')
    UDFrame:SetAlpha(0.625)
    UDFrame.marker = 347
    UDTable = {marker = 'PRIVATE-UnitDebuff', count = 37}
    function UDProperties()
        assert(UDFrame:GetObjectType() == 'Frame')
        assert(UDFrame:GetAlpha() == 0.625 and UDFrame.marker == 347)
        assert(UDTable.marker == 'PRIVATE-UnitDebuff' and UDTable.count == 37)
    end
    function UDPublic(value)
        assert(not issecretvalue(value), 'current public DTO convention only')
        if type(value) == 'table' then
            for key, child in pairs(value) do
                assert(not issecretvalue(key))
                UDPublic(child)
            end
        end
    end
    function UDData(...)
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
        UDPublic(data)
        return data
    end
    function UDCheck(name, ...)
        local data = UDData(UDQuery(...))
        if not name then
            assert(next(data.lines) == nil, 'true line-empty miss')
            return data
        end
        assert(data.icon == nil, 'unchanged builder does not publish an icon')
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
    function UDRecovery()
        UDCheck('Player outside source', 'player', 1)
        UDCheck('Party one outside source', 'party1', 1)
        UDCheck('Party two player source', 'party2', 1)
        UDProperties()
    end
    function UDTainted(probe)
        assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'UnitDebuffFixture')
            probe()
            assert(debug.getstacktaint() == 'UnitDebuffFixture', 'caller taint preserved')
        end
        debug.setobjecttaint(addon, 'UnitDebuffFixture')
        addon()
        assert(issecure(), 'outer context restored')
    end
    function UDReject(denial, ...)
        local before = debug.getstacktaint()
        local ok, err = pcall(UDQuery, ...)
        assert(not ok, 'expected rejection')
        assert(type(err) == 'string' and #err > 0)
        assert(string.find(err, 'C_TooltipInfo.GetUnitDebuff', 1, true), 'API error context')
        assert(not string.find(err, 'PRIVATE-UnitDebuff', 1, true), 'no private payload leak')
        local gate = string.find(err, 'requires an untainted caller', 1, true)
        if denial then assert(gate, 'authentication before parsing or lookup')
        else assert(not gate, 'ordinary type validation after secure authentication') end
        assert(debug.getstacktaint() == before)
        UDRecovery()
        assert(debug.getstacktaint() == before, 'recovery preserves caller taint')
    end
"#;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("real tooltip namespace");
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![
            aura(101, "Player helpful sentinel", true, true),
            aura(102, "Player outside source", false, false),
            aura(103, "Player own source", false, true),
        ];
        state.party_members = vec![
            member(
                "Debuff fixture one",
                vec![
                    aura(200, "Party one helpful sentinel", true, true),
                    aura(201, "Party one outside source", false, false),
                    aura(202, "Party one player source", false, true),
                ],
            ),
            member(
                "Debuff fixture two",
                vec![
                    aura(200, "Party two helpful sentinel", true, true),
                    aura(201, "Party two player source", false, true),
                ],
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
    if original == "UDFrame" {
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
        ("UDUnit", "party1"),
        ("UDFilter", "HARMFUL|PLAYER"),
        ("UDPrivate", "PRIVATE-UnitDebuff-Payload"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        publish_secret(&mut *lua, name, value);
    }
    for (name, number) in [("UDOne", 1.0), ("UDTwo", 2.0)] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        publish_secret(&mut *lua, name, value);
    }
    let value = wrap_secret(lua.state_mut(), Val::Nil).unwrap();
    publish_secret(&mut *lua, "UDNil", value);
    let value = wrap_host_secret_bool(lua.state_mut(), false);
    publish_secret(&mut *lua, "UDBool", value);
    wrap_original(&mut *lua, "UDTable", "UDSecretTable");
    wrap_original(&mut *lua, "UDFrame", "UDSecretFrame");
    drop(lua);
    env.exec("UDSecrets = {UDUnit, UDOne, UDTwo, UDFilter, UDNil, UDBool, UDPrivate, UDSecretTable, UDSecretFrame}; for _, value in ipairs(UDSecrets) do assert(issecretvalue(value)) end").unwrap();
    env
}

fn snapshot_roots(env: &WowLuaEnv) -> (Val, Vec<(Val, u64)>) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let Val::Table(list) = lua.get_global_val("UDSecrets") else {
        panic!("root list")
    };
    let roots = SECRET_NAMES
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
        .collect();
    (Val::Table(list), roots)
}

fn snapshot_auras(env: &WowLuaEnv) -> Vec<Vec<AuraInfo>> {
    let state = env.state().borrow();
    std::iter::once(state.player.buffs.clone())
        .chain(
            state
                .party_members
                .iter()
                .flat_map(|member| [member.buffs.clone(), member.debuffs.clone()]),
        )
        .collect()
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
    env.exec(script).expect("real GetUnitDebuff public probe");
    assert_auras_unchanged(&snapshot_auras(&env), &rows);
}

fn secret_probe(script: &str) {
    let env = secret_env();
    let roots = snapshot_roots(&env);
    let rows = snapshot_auras(&env);
    env.exec(script).expect("real GetUnitDebuff security probe");
    assert_eq!(
        snapshot_roots(&env),
        roots,
        "root-list and wrapper identity/secrecy"
    );
    assert_auras_unchanged(&snapshot_auras(&env), &rows);
}

#[test]
fn distinct_live_units_select_harmful_rows_from_correct_stores() {
    public_probe(
        "UDRecovery(); UDCheck('Player own source', 'player', 2); UDCheck('Party one player source', 'party1', 2)",
    );
}

#[test]
fn omitted_and_nil_filter_default_to_harmful() {
    public_probe(
        "UDCheck('Player outside source', 'player', 1); UDCheck('Player outside source', 'player', 1, nil); UDCheck('Party one outside source', 'party1', 1, nil)",
    );
}

#[test]
fn player_filter_selects_source_before_one_based_index() {
    public_probe(
        r#"
        UDCheck('Player own source', 'player', 1, 'HARMFUL|PLAYER')
        UDCheck('Party one player source', 'party1', 1, 'HARMFUL|PLAYER')
        UDCheck('Party two player source', 'party2', 1, 'HARMFUL|PLAYER')
        UDCheck(nil, 'player', 2, 'HARMFUL|PLAYER')
        UDCheck(nil, 'party1', 2, 'HARMFUL|PLAYER')
        UDCheck(nil, 'party1', 201, 'HARMFUL')
    "#,
    );
}

#[test]
fn blocked_visibility_precedes_index_and_is_unit_local() {
    public_probe(
        r#"
        C_UnitAuras.AddBlockedAura('party1', 201)
        UDCheck('Party one player source', 'party1', 1)
        UDCheck(nil, 'party1', 2)
        UDCheck('Party two player source', 'party2', 1)
        C_UnitAuras.AddBlockedAura('party1', 202)
        UDCheck(nil, 'party1', 1, 'HARMFUL|PLAYER')
        C_UnitAuras.AddBlockedAura('player', 102)
        UDCheck('Player own source', 'player', 1)
        UDCheck(nil, 'player', 2)
        C_UnitAuras.AddBlockedAura('player', 103)
        UDCheck(nil, 'player', 1)
    "#,
    );
}

#[test]
fn helpful_and_unknown_filters_do_not_select_harmful_or_helpful_sentinels() {
    public_probe(
        r#"
        for _, filter in ipairs({'HELPFUL', 'HELPFUL|PLAYER', 'UNKNOWN', 'UNKNOWN|PLAYER'}) do
            UDCheck(nil, 'player', 1, filter)
            UDCheck(nil, 'party1', 1, filter)
            UDCheck(nil, 'party2', 1, filter)
        end
        UDRecovery()
    "#,
    );
}

#[test]
fn unsupported_units_and_nonpositive_or_missing_rows_return_one_empty_dto() {
    public_probe(
        r#"
        for _, index in ipairs({0, -1, -37, 3, 999}) do
            UDCheck(nil, 'player', index)
            UDCheck(nil, 'party1', index)
        end
        UDCheck(nil, 'party2', 2)
        for _, unit in ipairs({'target', 'pet', 'focus', 'raid1', 'party3', 'missing', ''}) do
            UDCheck(nil, unit, 1)
        end
        UDRecovery()
    "#,
    );
}

#[test]
fn live_mutation_removal_and_clear_refresh_snapshots_without_cross_unit_leaks() {
    let env = fixture_env();
    let other = fixture_env();
    env.exec("UDOld = UDCheck('Party one outside source', 'party1', 1)")
        .unwrap();
    env.state().borrow_mut().party_members[0].debuffs[1].name = "Live party replacement".into();
    env.exec("UDCheck('Live party replacement', 'party1', 1); UDCheck('Party two player source', 'party2', 1); UDCheck('Player outside source', 'player', 1); assert(UDOld.lines[1].leftText == 'Party one outside source')").unwrap();
    env.state().borrow_mut().party_members[0].debuffs.remove(1);
    env.exec("UDCheck('Party one player source', 'party1', 1); UDCheck(nil, 'party1', 2)")
        .unwrap();
    env.state().borrow_mut().party_members[0].debuffs.clear();
    env.exec("UDCheck(nil, 'party1', 1); UDCheck('Party two player source', 'party2', 1)")
        .unwrap();
    env.state().borrow_mut().player.buffs[2].name = "Live player replacement".into();
    env.exec("UDCheck('Live player replacement', 'player', 2)")
        .unwrap();
    env.state().borrow_mut().player.buffs.clear();
    env.exec("UDCheck(nil, 'player', 1); UDCheck('Party two player source', 'party2', 1)")
        .unwrap();
    other.exec("UDRecovery(); UDCheck('Player own source', 'player', 2); UDCheck('Party one player source', 'party1', 2)").unwrap();
}

#[test]
fn dto_snapshots_are_fresh_and_caller_mutation_cannot_change_host_content() {
    public_probe(
        r#"
        local first = UDCheck('Party one outside source', 'party1', 1)
        local second = UDCheck('Party one outside source', 'party1', 1)
        assert(not rawequal(first, second) and not rawequal(first.lines, second.lines))
        assert(not rawequal(first.lines[1], second.lines[1]))
        first.lines[1].leftText = 'mutated result'
        first.lines[2] = nil
        first.extra = true
        assert(second.lines[1].leftText == 'Party one outside source')
        UDRecovery()
        local miss = UDCheck(nil, 'party1', 99)
        local another = UDCheck(nil, 'party1', 99)
        assert(not rawequal(miss, another) and not rawequal(miss.lines, another.lines))
        miss.lines[1] = {leftText = 'not an aura'}
        assert(next(another.lines) == nil)
        UDCheck(nil, 'party1', 99)
    "#,
    );
}

#[test]
fn public_tainted_calls_preserve_content_source_selection_and_caller_taint() {
    public_probe(
        r#"
        UDTainted(function()
            UDRecovery()
            UDCheck('Player own source', 'player', 1, 'HARMFUL|PLAYER')
            UDCheck('Party one player source', 'party1', 1, 'HARMFUL|PLAYER')
            UDCheck(nil, 'missing', 1)
            UDCheck(nil, 'player', 0)
        end)
        UDRecovery()
    "#,
    );
}

#[test]
fn secure_original_secret_unit_index_filter_and_nil_drive_real_selection() {
    secret_probe(
        r#"
        assert(issecure())
        UDCheck('Party one outside source', UDUnit, 1)
        UDCheck('Player own source', 'player', UDTwo)
        UDCheck('Party one player source', 'party1', UDTwo)
        UDCheck('Player own source', 'player', 1, UDFilter)
        UDCheck('Party one outside source', 'party1', 1, UDNil)
        UDCheck('Party one player source', UDUnit, UDOne, UDFilter)
        UDCheck(nil, UDUnit, UDTwo, UDFilter)
        UDCheck('Party one player source', UDUnit, UDTwo, UDNil)
        assert(issecure())
        UDProperties()
    "#,
    );
}

#[test]
fn tainted_secret_each_position_authenticates_before_earlier_malformed_parse() {
    secret_probe(
        r#"
        UDTainted(function()
            UDReject(true, UDUnit, 1)
            UDReject(true, UDUnit, false)
            UDReject(true, 'party1', UDOne)
            UDReject(true, false, UDOne)
            UDReject(true, nil, UDTwo)
            UDReject(true, 'party1', 1, UDFilter)
            UDReject(true, false, false, UDFilter)
            UDReject(true, nil, nil, UDNil)
            UDReject(true, 'missing', 999, UDNil)
            UDReject(true, UDNil, 1)
            UDReject(true, false, UDNil)
        end)
        UDRecovery()
    "#,
    );
}

#[test]
fn tainted_malformed_secret_payloads_are_denied_before_type_or_miss() {
    secret_probe(
        r#"
        UDTainted(function()
            for _, value in ipairs({UDBool, UDSecretTable, UDSecretFrame, UDPrivate}) do
                UDReject(true, value, false)
                UDReject(true, false, value)
                UDReject(true, 'missing', 999, value)
                assert(issecretvalue(value))
            end
        end)
        UDProperties()
    "#,
    );
}

#[test]
fn secure_and_public_invalid_types_are_ordinary_errors_without_payload_leaks() {
    secret_probe(
        r#"
        for _, value in ipairs({UDBool, UDSecretTable, UDSecretFrame}) do
            UDReject(false, value, 1)
            UDReject(false, 'party1', value)
            UDReject(false, 'party1', 1, value)
        end
        UDReject(false, UDNil, 1)
        UDReject(false, 'party1', UDNil)
        UDReject(false, UDOne, 1)
        UDReject(false, 'party1', 1, UDOne)
        UDReject(false, 'party1', UDUnit)
        UDReject(false, 'party1', UDPrivate)
        UDTainted(function()
            for _, value in ipairs({false, UDTable, UDFrame}) do
                UDReject(false, value, 1)
                UDReject(false, 'party1', value)
                UDReject(false, 'party1', 1, value)
            end
            UDReject(false)
            UDReject(false, nil, 1)
            UDReject(false, 1, 1)
            UDReject(false, 'party1')
            UDReject(false, 'party1', nil)
            UDReject(false, 'party1', '1')
            UDReject(false, 'party1', 1, 1)
        end)
        UDProperties()
    "#,
    );
}

#[test]
fn fractional_nonfinite_and_out_of_i32_indexes_are_inferred_errors() {
    public_probe(
        r#"
        for _, index in ipairs({1.5, -0.5, math.huge, -math.huge, 0/0, 2147483648, -2147483649}) do
            UDReject(false, 'player', index)
            UDReject(false, 'missing', index)
        end
    "#,
    );
}

#[test]
fn allocation_and_gc_preserve_permanent_secret_roots_and_original_properties() {
    secret_probe(
        r#"
        collectgarbage('collect')
        UDCheck('Party one player source', UDUnit, UDOne, UDFilter)
        UDTainted(function()
            for _, value in ipairs(UDSecrets) do
                UDReject(true, 'missing', 999, value)
                local garbage = {}
                for i=1,128 do garbage[i] = {name = 'allocation-' .. i, data = {i}} end
                collectgarbage('collect')
                assert(issecretvalue(value))
                UDRecovery()
            end
        end)
        collectgarbage('collect')
        UDCheck('Party one player source', UDUnit, UDTwo, UDNil)
        for _, value in ipairs(UDSecrets) do assert(issecretvalue(value)) end
        assert(issecure())
        UDProperties()
    "#,
    );
}
