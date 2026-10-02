//! Exact Retail12.0.5 rows339/340/344/345/349/350; chosen, inferred input policy.
//! Tests/spec only. Main owns compilation and genuine RED/GREEN accounting.
#![cfg(feature = "retail-12-0-5")]

use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string, wrap_secret,
};
use rilua::{LuaApiMut, Val};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

const SECRET_NAMES: [&str; 9] = [
    "ATUnit",
    "ATOne",
    "ATTwo",
    "ATFilter",
    "ATNil",
    "ATBool",
    "ATPrivate",
    "ATSecretTable",
    "ATSecretFrame",
];

fn aura_rows() -> Vec<AuraInfo> {
    let helpful = AuraInfo {
        name: "Flash of Light".into(),
        spell_id: 19750,
        icon: 135987,
        duration: 3600.0,
        expiration_time: 3600.0,
        applications: 0,
        source_unit: "player".into(),
        is_helpful: true,
        is_raid: true,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id: 1,
    };
    let harmful = AuraInfo {
        name: "Corruption".into(),
        spell_id: 172,
        icon: 136118,
        is_helpful: false,
        dispel_type: Some("Magic".into()),
        aura_instance_id: 2,
        ..helpful.clone()
    };
    vec![helpful, harmful]
}

const ASSERTIONS: &str = r#"
    ATFrame = CreateFrame('Frame')
    ATFrame:SetAlpha(0.625)
    ATFrame.marker = 339
    ATTable = {marker = 'PRIVATE-AuraTooltip', count = 37}
    function ATProperties()
        assert(ATFrame:GetObjectType() == 'Frame')
        assert(ATFrame:GetAlpha() == 0.625 and ATFrame.marker == 339)
        assert(ATTable.marker == 'PRIVATE-AuraTooltip' and ATTable.count == 37)
    end
    function ATPublic(value)
        assert(not issecretvalue(value), 'chosen public DTO, not result-secrecy credit')
        if type(value) ~= 'table' then return end
        for key, child in pairs(value) do
            assert(not issecretvalue(key), 'public key')
            ATPublic(child)
        end
    end
    function ATRGBA(color)
        if type(color) ~= 'table' and type(color) ~= 'userdata' then return nil end
        if type(color.GetRGBA) ~= 'function' then return nil end
        local function channels(...)
            assert(select('#', ...) == 4, 'RGBA arity')
            local result = {...}
            for i=1,4 do
                assert(type(result[i]) == 'number' and not issecretvalue(result[i]))
            end
            return result
        end
        return channels(color:GetRGBA())
    end
    function ATEqual(left, right)
        local lc, rc = ATRGBA(left), ATRGBA(right)
        if lc or rc then
            assert(lc and rc, 'both semantic colors')
            for i=1,4 do assert(lc[i] == rc[i], 'same RGBA component') end
            return
        end
        assert(type(left) == type(right), 'same public DTO type')
        if type(left) ~= 'table' then
            assert(left == right, 'same public scalar')
            return
        end
        for key, value in pairs(left) do ATEqual(value, right[key]) end
        for key, value in pairs(right) do
            assert(left[key] ~= nil, 'no additional DTO field')
        end
    end
    function ATData(...)
        assert(select('#', ...) == 1, 'one result')
        local data = ...
        assert(type(data) == 'table' and type(data.lines) == 'table')
        assert(data.type == Enum.TooltipDataType.UnitAura)
        ATPublic(data)
        local fields = 0
        for key in pairs(data) do
            assert(key == 'type' or key == 'lines', 'preserved top-level DTO; no ID/width change')
            fields = fields + 1
        end
        assert(fields == 2)
        return data
    end
    function ATCheck(id, name, ...)
        local data = ATData(ATQuery(...))
        if not name then
            assert(next(data.lines) == nil, 'fresh line-empty UnitAura miss')
            return data
        end
        assert(#data.lines == 3, 'existing name/duration/description payload')
        assert(data.lines[1].type == Enum.TooltipDataLineType.SpellName)
        assert(data.lines[1].leftText == name, 'concrete aura identity ' .. tostring(id))
        assert(data.lines[2].type == Enum.TooltipDataLineType.SpellName)
        assert(data.lines[2].leftText == '1 hr', 'preserved hardcoded duration limitation')
        assert(data.lines[3].type == Enum.TooltipDataLineType.SpellDescription)
        assert(type(data.lines[3].leftText) == 'string' and #data.lines[3].leftText > 0)
        assert(data.lines[3].wrapText == true)
        for i=1,2 do
            local rgba = ATRGBA(data.lines[i].leftColor)
            assert(rgba, 'public name/duration color')
            assert(rgba[4] == 1, 'opaque existing highlight')
        end
        return data
    end
    function ATExpected(id)
        if id == 1 and ATClass ~= 'Debuff' then return 'Flash of Light' end
        if id == 2 and ATClass ~= 'Buff' then return 'Corruption' end
        return nil
    end
    function ATRecovery()
        ATCheck(ATLiveID, ATLiveName, 'player', ATLiveID)
        ATProperties()
    end
    function ATContexts(probe)
        assert(issecure())
        probe()
        assert(issecure(), 'secure caller preserved')
        local function addon()
            assert(debug.getstacktaint() == 'AuraTooltipFixture')
            probe()
            assert(debug.getstacktaint() == 'AuraTooltipFixture', 'ordinary caller taint preserved')
        end
        debug.setobjecttaint(addon, 'AuraTooltipFixture')
        addon()
        assert(issecure(), 'outer secure context restored')
    end
    function ATTainted(probe)
        assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'AuraTooltipFixture')
            probe()
            assert(debug.getstacktaint() == 'AuraTooltipFixture')
        end
        debug.setobjecttaint(addon, 'AuraTooltipFixture')
        addon()
        assert(issecure(), 'secure context restored after denial')
    end
    function ATReject(denial, ...)
        local before = debug.getstacktaint()
        local ok, err = pcall(ATQuery, ...)
        assert(not ok, 'argument rejection')
        assert(type(err) == 'string' and #err > 0, 'nonempty rejection')
        assert(string.find(err, ATAPI, 1, true), 'exact namespace API context')
        assert(not string.find(err, 'PRIVATE-AuraTooltip', 1, true), 'no private payload leak')
        local gate = string.find(err, 'requires an untainted caller', 1, true)
        if denial then assert(gate, 'VM denial before type/model access')
        else assert(not gate, 'secure caller reaches ordinary type validation') end
        assert(debug.getstacktaint() == before, 'error preserves caller taint')
        ATRecovery()
        assert(debug.getstacktaint() == before, 'public recovery preserves caller taint')
    end
"#;

fn seed_env(class: &str) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("real tooltip namespace");
    env.state().borrow_mut().player.buffs = aura_rows();
    env.exec(ASSERTIONS)
        .expect("assertions; no query/callback replacement");
    env.exec(&format!(
        "ATClass = '{class}'; ATAPI = 'C_TooltipInfo.GetUnit{class}ByAuraInstanceID'; \
         ATQuery = C_TooltipInfo.GetUnit{class}ByAuraInstanceID; \
         assert(type(ATQuery) == 'function'); ATLiveID = {}; ATLiveName = '{}'",
        if class == "Debuff" { 2 } else { 1 },
        if class == "Debuff" {
            "Corruption"
        } else {
            "Flash of Light"
        }
    ))
    .expect("alias exact real namespace function");
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
        panic!("real original table/frame")
    };
    if original == "ATFrame" {
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
    let secret = wrap_secret(lua.state_mut(), value).expect("wrap actual table/frame");
    publish_secret(lua, name, secret);
    lua.state_mut().pop();
}

fn seed_secret_env(class: &str) -> WowLuaEnv {
    let env = seed_env(class);
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).unwrap();
    for (name, text) in [
        ("ATUnit", "player"),
        ("ATFilter", "HARMFUL"),
        ("ATPrivate", "PRIVATE-AuraTooltip-Payload"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        publish_secret(&mut *lua, name, value);
    }
    for (name, number) in [("ATOne", 1.0), ("ATTwo", 2.0)] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        publish_secret(&mut *lua, name, value);
    }
    let nil = wrap_secret(lua.state_mut(), Val::Nil).unwrap();
    publish_secret(&mut *lua, "ATNil", nil);
    let boolean = wrap_host_secret_bool(lua.state_mut(), false);
    publish_secret(&mut *lua, "ATBool", boolean);
    wrap_original(&mut *lua, "ATTable", "ATSecretTable");
    wrap_original(&mut *lua, "ATFrame", "ATSecretFrame");
    drop(lua);
    env.exec("ATSecrets = {ATUnit, ATOne, ATTwo, ATFilter, ATNil, ATBool, ATPrivate, ATSecretTable, ATSecretFrame}; for _, value in ipairs(ATSecrets) do assert(issecretvalue(value)) end").unwrap();
    env
}

fn snapshot_roots(env: &WowLuaEnv) -> Vec<(Val, u64)> {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let Val::Table(list) = lua.get_global_val("ATSecrets") else {
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
                "host identity only; no tainted secret BOOL equality"
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

fn assert_rows_unchanged(actual: &[AuraInfo], expected: &[AuraInfo]) {
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

fn exec_probe(class: &str, script: &str) {
    let env = seed_secret_env(class);
    let roots = snapshot_roots(&env);
    let rows = env.state().borrow().player.buffs.clone();
    env.exec(script)
        .expect("exact namespace payload/security probe");
    // Fresh VM entry after errors/tainted Lua; metadata, not BOOL payload equality.
    assert_eq!(
        snapshot_roots(&env),
        roots,
        "live wrapper identity/allocation/secrecy"
    );
    assert_rows_unchanged(&env.state().borrow().player.buffs, &rows);
}

fn assert_live_updates(class: &str) {
    let env = seed_env(class);
    let other = seed_env(class);
    env.exec(
        r#"
        ATOld = ATCheck(ATLiveID, ATLiveName, 'player', ATLiveID)
        local second = ATCheck(ATLiveID, ATLiveName, 'player', ATLiveID)
        assert(not rawequal(ATOld, second), 'fresh DTO')
        assert(not rawequal(ATOld.lines, second.lines), 'fresh lines')
        assert(not rawequal(ATOld.lines[1], second.lines[1]))
        ATEqual(ATOld, second)
        second.lines[1].leftText = 'mutated result'
        second.lines[2] = nil
        second.extra = true
        ATRecovery()
    "#,
    )
    .unwrap();
    assert_rows_unchanged(&env.state().borrow().player.buffs, &aura_rows());
    let live_index = if class == "Debuff" { 1 } else { 0 };
    let mut replacement = aura_rows()[live_index].clone();
    replacement.name = "Live replacement aura".into();
    env.state().borrow_mut().player.buffs = vec![replacement];
    env.exec("ATCheck(ATLiveID, 'Live replacement aura', 'player', ATLiveID); assert(ATOld.lines[1].leftText == ATLiveName)").unwrap();
    other.exec("ATRecovery()").unwrap();
    assert_rows_unchanged(&other.state().borrow().player.buffs, &aura_rows());
    env.state().borrow_mut().player.buffs.clear();
    env.exec(
        "ATCheck(ATLiveID, nil, 'player', ATLiveID); assert(ATOld.lines[1].leftText == ATLiveName)",
    )
    .unwrap();
}

// Eight independent tests per exact query: a failure in Aura cannot hide Buff/Debuff.
macro_rules! query_tests {
    ($module:ident, $class:literal) => {
        mod $module {
            use super::*;

            #[test]
            fn public_payload_classification_misses_and_ignored_filter() {
                exec_probe($class, r#"
                    ATContexts(function()
                        for id=1,2 do
                            local expected = ATExpected(id)
                            local baseline = ATCheck(id, expected, 'player', id)
                            for _, filter in ipairs({'HELPFUL', 'HARMFUL', 'PRIVATE-AuraTooltip-ignored'}) do
                                ATEqual(baseline, ATCheck(id, expected, 'player', id, filter))
                            end
                            ATEqual(baseline, ATCheck(id, expected, 'player', id, nil))
                        end
                        for _, unit in ipairs({'target', 'party1', 'missing'}) do
                            ATCheck(ATLiveID, nil, unit, ATLiveID)
                        end
                        local first = ATCheck(999, nil, 'player', 999)
                        local second = ATCheck(999, nil, 'player', 999)
                        assert(not rawequal(first, second), 'fresh miss DTO')
                        assert(not rawequal(first.lines, second.lines), 'fresh miss lines')
                        ATRecovery()
                    end)
                "#);
            }

            #[test]
            fn secure_authentic_unit_string_returns_meaningful_payload() {
                exec_probe($class, r#"
                    assert(issecure())
                    for id=1,2 do
                        local name = ATExpected(id)
                        local public = ATCheck(id, name, 'player', id)
                        ATEqual(public, ATCheck(id, name, ATUnit, id))
                    end
                    assert(issecure() and issecretvalue(ATUnit))
                    ATRecovery()
                "#);
            }

            #[test]
            fn secure_authentic_numeric_instance_ids_select_concrete_rows() {
                exec_probe($class, r#"
                    assert(issecure())
                    ATEqual(ATCheck(1, ATExpected(1), 'player', 1),
                        ATCheck(1, ATExpected(1), 'player', ATOne))
                    ATEqual(ATCheck(2, ATExpected(2), 'player', 2),
                        ATCheck(2, ATExpected(2), 'player', ATTwo))
                    ATRecovery()
                    assert(issecure() and issecretvalue(ATOne) and issecretvalue(ATTwo))
                "#);
            }

            #[test]
            fn secure_authentic_optional_string_and_nil_remain_ignored() {
                exec_probe($class, r#"
                    assert(issecure())
                    local baseline = ATCheck(ATLiveID, ATLiveName, 'player', ATLiveID)
                    ATEqual(baseline, ATCheck(ATLiveID, ATLiveName, 'player', ATLiveID, ATFilter))
                    ATEqual(baseline, ATCheck(ATLiveID, ATLiveName, 'player', ATLiveID, ATNil))
                    ATEqual(baseline, ATCheck(ATLiveID, ATLiveName, ATUnit,
                        ATLiveID == 1 and ATOne or ATTwo, ATFilter))
                    assert(issecure() and issecretvalue(ATFilter) and issecretvalue(ATNil))
                    ATRecovery()
                "#);
            }

            #[test]
            fn tainted_secret_gate_precedes_lookup_and_public_type_errors() {
                exec_probe($class, r#"
                    ATTainted(function()
                        ATReject(true, ATUnit, ATLiveID)
                        ATReject(true, ATUnit, 999)
                        ATReject(true, ATUnit, false)
                        for _, id in ipairs({ATOne, ATTwo}) do
                            ATReject(true, 'player', id)
                            ATReject(true, 'missing', id)
                            ATReject(true, false, id)
                        end
                        for _, filter in ipairs({ATFilter, ATNil}) do
                            ATReject(true, 'player', ATLiveID, filter)
                            ATReject(true, 'player', 999, filter)
                            ATReject(true, false, false, filter)
                        end
                        for _, value in ipairs({ATBool, ATSecretTable, ATSecretFrame, ATPrivate}) do
                            ATReject(true, value, false)
                            ATReject(true, false, value)
                            assert(issecretvalue(value))
                        end
                        ATRecovery()
                    end)
                    ATRecovery()
                "#);
            }

            #[test]
            fn secure_wrong_type_bool_table_frame_reject_with_context_and_recovery() {
                exec_probe($class, r#"
                    assert(issecure())
                    for _, value in ipairs({ATBool, ATSecretTable, ATSecretFrame}) do
                        ATReject(false, value, ATLiveID)
                        ATReject(false, 'player', value)
                        assert(issecretvalue(value))
                    end
                    ATContexts(function()
                        for _, value in ipairs({false, ATTable, ATFrame}) do
                            ATReject(false, value, ATLiveID)
                            ATReject(false, 'player', value)
                        end
                    end)
                    ATProperties()
                "#);
            }

            #[test]
            fn fresh_dtos_read_only_live_replacement_clear_and_environment_isolation() {
                assert_live_updates($class);
            }

            #[test]
            fn rooted_secret_gc_failure_recovery_retains_payload_and_trust() {
                exec_probe($class, r#"
                    collectgarbage('collect')
                    ATCheck(ATLiveID, ATLiveName, ATUnit,
                        ATLiveID == 1 and ATOne or ATTwo, ATNil)
                    ATTainted(function()
                        for _, secret in ipairs(ATSecrets) do
                            ATReject(true, 'player', ATLiveID, secret)
                            collectgarbage('collect')
                            assert(issecretvalue(secret))
                            ATRecovery()
                        end
                    end)
                    collectgarbage('collect')
                    ATCheck(ATLiveID, ATLiveName, ATUnit,
                        ATLiveID == 1 and ATOne or ATTwo, ATFilter)
                    assert(issecure())
                "#);
            }
        }
    };
}

query_tests!(aura, "Aura");
query_tests!(buff, "Buff");
query_tests!(debuff, "Debuff");
