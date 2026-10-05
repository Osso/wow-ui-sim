//! Exact330/331: inferred finite item-level variants, not native acquisition/parity.
//! Scaffold only: main must establish compiled behavioral RED before production work.
#![cfg(feature = "retail-12-0-5")]

use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string, wrap_secret,
};
use rilua::{LuaApiMut, Val};
use wow_ui_sim::c_api::ItemTooltipContext;
use wow_ui_sim::items::{ItemInfo, get_item};
use wow_ui_sim::lua_api::WowLuaEnv;

const ITEM: u32 = 211995;
const SECRET_NAMES: [&str; 10] = [
    "ITItem",
    "ITOne",
    "ITTwo",
    "IT70",
    "IT80",
    "ITNil",
    "ITBool",
    "ITString",
    "ITSecretTable",
    "ITSecretFrame",
];

fn key(
    item_id: u32,
    item_context: Option<u32>,
    treasure_context_level: Option<u32>,
) -> ItemTooltipContext {
    ItemTooltipContext {
        item_id,
        item_context,
        treasure_context_level,
    }
}

const ASSERTIONS: &str = r#"
    ITAPI = 'C_TooltipInfo.GetItemByID'
    ITFrame = CreateFrame('Frame'); ITFrame:SetAlpha(0.625); ITFrame.marker = 330
    ITTable = {marker='PRIVATE-ItemContext', count=37}
    function ITProperties()
        assert(ITFrame:GetAlpha() == 0.625 and ITFrame.marker == 330)
        assert(ITFrame:GetObjectType() == 'Frame')
        assert(ITTable.marker == 'PRIVATE-ItemContext' and ITTable.count == 37)
    end
    function ITRGBA(value)
        if type(value) ~= 'table' and type(value) ~= 'userdata' then return nil end
        if type(value.GetRGBA) ~= 'function' then return nil end
        local function channels(...)
            assert(select('#', ...) == 4)
            local result = {...}
            for i=1,4 do assert(type(result[i]) == 'number') end
            return result
        end
        return channels(value:GetRGBA())
    end
    function ITEqual(a,b)
        local ac,bc = ITRGBA(a),ITRGBA(b)
        if ac or bc then
            assert(ac and bc)
            for i=1,4 do assert(ac[i] == bc[i], 'semantic RGBA equality') end
            return
        end
        assert(type(a) == type(b), 'DTO value type')
        if type(a) ~= 'table' then assert(a == b, 'DTO scalar'); return end
        for k,v in pairs(a) do ITEqual(v,b[k]) end
        for k in pairs(b) do assert(a[k] ~= nil, 'no extra DTO fields') end
    end
    function ITPublic(value)
        assert(not issecretvalue(value), 'chosen public output; no native output-policy credit')
        if ITRGBA(value) or type(value) ~= 'table' then return end
        for k,v in pairs(value) do assert(not issecretvalue(k)); ITPublic(v) end
    end
    function ITData(...)
        assert(select('#', ...) == 1, 'one result')
        local data = ...
        assert(type(data) == 'table' and data.type == Enum.TooltipDataType.Item)
        assert(type(data.lines) == 'table')
        ITPublic(data)
        return data
    end
    function ITMiss(...)
        local data = ITData(C_TooltipInfo.GetItemByID(...))
        assert(data.id == nil and next(data.lines) == nil, 'empty Item DTO, no invented name')
        local count = 0
        for k in pairs(data) do assert(k == 'type' or k == 'lines'); count=count+1 end
        assert(count == 2)
        return data
    end
    function ITCheck(level, ...)
        local data = ITData(C_TooltipInfo.GetItemByID(...))
        assert(data.id == 211995 and #data.lines == 8, 'meaningful catalog gear payload')
        assert(data.lines[1].type == 22 and data.lines[1].leftText == "Entombed Seraph's Sabatons")
        assert(data.lines[2].type == 31 and data.lines[2].leftText == 'Item Level '..level)
        assert(data.lines[3].type == 21 and data.lines[3].leftText == 'Feet')
        assert(data.lines[4].type == 20 and data.lines[4].leftText == ITEM_BIND_ON_PICKUP)
        local name = ITRGBA(data.lines[1].leftColor)
        assert(name)
        assert(name[1] == 0.64)
        assert(name[2] == 0.21)
        assert(name[3] == 0.93)
        assert(name[4] == 1)
        local specs = {
            {5259,1.2,ITEM_MOD_STRENGTH_SHORT}, {7889,1.8,ITEM_MOD_STAMINA_SHORT},
            {4799,0.75,ITEM_MOD_MASTERY_RATING_SHORT}, {2201,0.75,ITEM_MOD_VERSATILITY},
        }
        for i,spec in ipairs(specs) do
            local line = data.lines[i+4]
            local value = math.max(1,math.floor(level*2*spec[1]/10000*spec[2]+0.5))
            assert(line.type == 13 and line.leftText == '+'..value..' '..spec[3], 'existing budget heuristic')
            local rgba = ITRGBA(line.leftColor)
            assert(rgba and rgba[2] > rgba[1] and rgba[4] == 1, 'opaque green stat')
        end
        if ITBase then
            -- Compare every existing DTO field; only level/stat text may differ.
            local texts = {}
            for i,line in ipairs(data.lines) do texts[i] = line.leftText end
            data.lines[2].leftText = ITBase.lines[2].leftText
            for i=5,8 do data.lines[i].leftText = ITBase.lines[i].leftText end
            ITEqual(data, ITBase)
            for i,line in ipairs(data.lines) do line.leftText = texts[i] end
        end
        return data
    end
    function ITContexts(probe)
        assert(issecure()); probe(); assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'ItemContextFixture')
            probe()
            assert(debug.getstacktaint() == 'ItemContextFixture')
        end
        debug.setobjecttaint(addon, 'ItemContextFixture'); addon()
        assert(issecure(), 'outer trust restored')
    end
    function ITTainted(probe)
        assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'ItemContextFixture'); probe()
            assert(debug.getstacktaint() == 'ItemContextFixture')
        end
        debug.setobjecttaint(addon, 'ItemContextFixture'); addon(); assert(issecure())
    end
    function ITReject(denial, ...)
        local before = debug.getstacktaint()
        local ok,err = pcall(C_TooltipInfo.GetItemByID, ...)
        assert(not ok, 'argument rejection')
        assert(type(err) == 'string', 'public error string')
        assert(#err > 0, 'nonempty rejection')
        assert(string.find(err, ITAPI, 1, true), 'exact API namespace')
        assert(not string.find(err, 'PRIVATE-ItemContext', 1, true), 'no private leak')
        local gate = string.find(err, 'requires an untainted caller', 1, true)
        if denial then assert(gate, 'authentication before type/lookup')
        else assert(not gate, 'secure ordinary validation') end
        assert(debug.getstacktaint() == before)
        ITCheck(601,211995,nil,1,nil); ITProperties()
        assert(debug.getstacktaint() == before, 'public recovery preserves caller')
    end
"#;

fn catalog() -> ItemInfo {
    let item = get_item(ITEM)
        .expect("public finite catalog accessor")
        .clone();
    assert_eq!(item.name, "Entombed Seraph's Sabatons");
    assert_eq!(
        (
            item.item_level,
            item.quality,
            item.inventory_type,
            item.bonding
        ),
        (571, 4, 8, 1)
    );
    assert_eq!(&item.stat_percent_editor[..4], &[5259, 7889, 4799, 2201]);
    assert_eq!(&item.stat_modifier_bonus_stat[..4], &[74, 7, 49, 40]);
    item
}

fn empty_env() -> WowLuaEnv {
    let item = catalog();
    let env = WowLuaEnv::new().expect("real tooltip namespace");
    env.state().borrow_mut().player.class_index = 2;
    assert!(env.state().borrow().item_tooltip_levels.is_empty());
    env.exec(ASSERTIONS)
        .expect("assertions only; never replace actual query/callback");
    env.exec(&format!("ITBase = ITCheck({},211995)", item.item_level))
        .unwrap();
    env
}

fn seeded_env() -> WowLuaEnv {
    let env = empty_env();
    // Explicit TEST DATA, no native mapping or fabricated production default.
    env.state().borrow_mut().item_tooltip_levels.extend([
        (key(ITEM, Some(1), None), 601),
        (key(ITEM, Some(2), None), 602),
        (key(ITEM, Some(1), Some(70)), 603),
        (key(ITEM, Some(1), Some(80)), 604),
    ]);
    env
}

fn assert_catalog_unchanged(expected: &ItemInfo) {
    let actual = get_item(ITEM).unwrap();
    assert_eq!(actual.name, expected.name);
    assert_eq!(
        (
            actual.quality,
            actual.item_level,
            actual.required_level,
            actual.inventory_type
        ),
        (
            expected.quality,
            expected.item_level,
            expected.required_level,
            expected.inventory_type
        )
    );
    assert_eq!(
        (
            actual.sell_price,
            actual.stackable,
            actual.bonding,
            actual.expansion_id,
            actual.icon_file_data_id
        ),
        (
            expected.sell_price,
            expected.stackable,
            expected.bonding,
            expected.expansion_id,
            expected.icon_file_data_id
        )
    );
    assert_eq!(actual.stat_percent_editor, expected.stat_percent_editor);
    assert_eq!(
        actual.stat_modifier_bonus_stat,
        expected.stat_modifier_bonus_stat
    );
}

fn probe(env: &WowLuaEnv, script: &str) {
    let levels = env.state().borrow().item_tooltip_levels.clone();
    let item = catalog();
    let player_class = env.state().borrow().player.class_index;
    env.exec(script).expect("actual GetItemByID contract");
    assert_eq!(
        env.state().borrow().item_tooltip_levels,
        levels,
        "read-only host inputs"
    );
    assert_eq!(env.state().borrow().player.class_index, player_class);
    assert_catalog_unchanged(&item);
    env.exec("ITProperties()").unwrap();
}

fn publish(lua: &mut impl LuaApiMut, name: &str, value: Val) {
    lua.state_mut().push(value);
    let result = lua.set_global_val(name, value);
    lua.state_mut().pop();
    result.expect("root authentic VM wrapper");
}

fn secret_env() -> WowLuaEnv {
    let env = seeded_env();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).unwrap();
    for (name, number) in [
        ("ITItem", 211995.0),
        ("ITOne", 1.0),
        ("ITTwo", 2.0),
        ("IT70", 70.0),
        ("IT80", 80.0),
    ] {
        let wrapper = wrap_host_secret_number(lua.state_mut(), number);
        publish(&mut *lua, name, wrapper);
    }
    let nil = wrap_secret(lua.state_mut(), Val::Nil).unwrap();
    publish(&mut *lua, "ITNil", nil);
    let boolean = wrap_host_secret_bool(lua.state_mut(), false);
    publish(&mut *lua, "ITBool", boolean);
    let string = wrap_host_secret_string(lua.state_mut(), "PRIVATE-ItemContext");
    publish(&mut *lua, "ITString", string);
    for (original, name) in [("ITTable", "ITSecretTable"), ("ITFrame", "ITSecretFrame")] {
        let value = lua.get_global_val(original);
        let Val::Table(reference) = value else {
            panic!("actual frame/table")
        };
        if original == "ITFrame" {
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
        let wrapper = wrap_secret(lua.state_mut(), value).unwrap();
        publish(&mut *lua, name, wrapper);
        lua.state_mut().pop();
    }
    drop(lua);
    env.exec("ITSecrets = {ITItem,ITOne,ITTwo,IT70,IT80,ITNil,ITBool,ITString,ITSecretTable,ITSecretFrame}; for _,v in ipairs(ITSecrets) do assert(issecretvalue(v)) end").unwrap();
    env
}

fn roots(env: &WowLuaEnv) -> Vec<(Val, u64)> {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let Val::Table(list) = lua.get_global_val("ITSecrets") else {
        panic!("root list")
    };
    SECRET_NAMES
        .iter()
        .enumerate()
        .map(|(i, name)| {
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
                    .get_int(i as i64 + 1),
                value,
                "host metadata identity only, never tainted secret BOOL equality"
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

fn secret_probe(script: &str) {
    let env = secret_env();
    let before = roots(&env);
    probe(&env, script);
    assert_eq!(
        roots(&env),
        before,
        "root identity/allocation/secrecy retained"
    );
}

#[test]
fn tooltip_item_payload_survives_collection_during_color_callback() {
    let env = seeded_env();
    env.exec(
        r#"
        local createColor = CreateColor
        CreateColor = function(...)
            collectgarbage('collect')
            return createColor(...)
        end
        "#,
    )
    .unwrap();
    probe(&env, "ITCheck(571,211995); ITCheck(604,211995,nil,1,80)");
}

#[test]
fn default_no_map_is_meaningful_base_catalog_not_a_missing_provider() {
    probe(
        &empty_env(),
        "ITEqual(ITBase,ITCheck(571,211995)); ITEqual(ITBase,ITCheck(571,211995,nil,nil,nil)); ITMiss(211995,nil,1); ITMiss(211995,nil,nil,70)",
    );
}

#[test]
fn context_one_changes_level_and_derived_stats_preserving_every_other_value() {
    probe(
        &seeded_env(),
        "local a=ITCheck(601,211995,nil,1); assert(a.lines[5].leftText ~= ITBase.lines[5].leftText)",
    );
}

#[test]
fn context_two_independently_selects_same_item_distinct_variant() {
    probe(
        &seeded_env(),
        "local a=ITCheck(601,211995,nil,1); local b=ITCheck(602,211995,nil,2); assert(a.id == b.id and a.lines[2].leftText ~= b.lines[2].leftText and a.lines[6].leftText ~= b.lines[6].leftText)",
    );
}

#[test]
fn treasure_seventy_selects_exact_context_one_variant() {
    probe(
        &seeded_env(),
        "ITCheck(603,211995,nil,1,70); ITMiss(211995,nil,2,70)",
    );
}

#[test]
fn treasure_eighty_independently_changes_level_and_stats_for_same_context() {
    probe(
        &seeded_env(),
        "local a=ITCheck(603,211995,nil,1,70); local b=ITCheck(604,211995,nil,1,80); assert(a.lines[2].leftText ~= b.lines[2].leftText and a.lines[6].leftText ~= b.lines[6].leftText)",
    );
}

#[test]
fn explicit_nil_nil_override_replaces_only_default_item_level() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .item_tooltip_levels
        .insert(key(ITEM, None, None), 605);
    probe(
        &env,
        "ITCheck(605,211995); ITCheck(605,211995,nil,nil,nil); ITCheck(601,211995,nil,1)",
    );
}

#[test]
fn nil_and_zero_are_distinct_in_both_optional_positions() {
    let env = seeded_env();
    env.state().borrow_mut().item_tooltip_levels.extend([
        (key(ITEM, None, None), 605),
        (key(ITEM, Some(0), None), 606),
        (key(ITEM, None, Some(0)), 607),
        (key(ITEM, Some(0), Some(0)), 608),
    ]);
    probe(
        &env,
        "ITCheck(605,211995); ITCheck(606,211995,nil,0); ITCheck(607,211995,nil,nil,0); ITCheck(608,211995,nil,0,0)",
    );
}

#[test]
fn missing_combinations_never_default_cross_context_or_nearest_scale() {
    probe(
        &seeded_env(),
        "for _,c in ipairs({3,70,4294967295}) do ITMiss(211995,nil,c) end; ITMiss(211995,nil,1,71); ITMiss(211995,nil,2,80); ITMiss(211995,nil,nil,70); ITMiss(211995,nil,1,0); ITMiss(211995,nil,0,70)",
    );
}

#[test]
fn unknown_catalog_id_stays_empty_even_with_explicit_host_level() {
    let env = seeded_env();
    assert!(get_item(u32::MAX).is_none());
    env.state().borrow_mut().item_tooltip_levels.extend([
        (key(u32::MAX, None, None), 601),
        (key(u32::MAX, Some(1), Some(70)), 604),
    ]);
    probe(&env, "ITMiss(4294967295); ITMiss(4294967295,nil,1,70)");
}

#[test]
fn finite_integral_u32_context_boundaries_can_select_explicit_keys() {
    let env = seeded_env();
    env.state().borrow_mut().item_tooltip_levels.extend([
        (key(ITEM, Some(u32::MAX), Some(0)), 605),
        (key(ITEM, Some(0), Some(u32::MAX)), 606),
    ]);
    probe(
        &env,
        "ITCheck(605,211995,nil,4294967295,0); ITCheck(606,211995,nil,0,4294967295)",
    );
}

#[test]
fn public_context_domains_reject_wrong_types_nonfinite_fractional_and_overflow() {
    probe(
        &seeded_env(),
        r#"
        for _,v in ipairs({false,true,'1','PRIVATE-ItemContext',{},ITFrame,-1,1.5,4294967296,math.huge,-math.huge,0/0}) do
            ITReject(false,211995,nil,v,nil); ITReject(false,211995,nil,1,v)
        end
    "#,
    );
}

#[test]
fn item_id_retains_existing_u32_positive_and_zero_miss_behavior() {
    probe(
        &seeded_env(),
        "ITCheck(571,211995); ITMiss(0); ITMiss(0,nil,1,70)",
    );
}

#[test]
fn ordinary_quality_is_ignored_without_new_type_or_output_policy() {
    probe(
        &seeded_env(),
        r#"
        ITContexts(function()
            for _,q in ipairs({0,4,99,-1,1.5,false,'PRIVATE-ItemContext',{},ITFrame}) do
                ITEqual(ITBase,ITCheck(571,211995,q))
                ITEqual(ITCheck(604,211995,nil,1,80),ITCheck(604,211995,q,1,80))
            end
        end)
    "#,
    );
}

#[test]
fn repeated_variants_misses_and_base_reads_never_mutate_inputs() {
    probe(
        &seeded_env(),
        "for i=1,4 do ITCheck(601,211995,nil,1); ITCheck(604,211995,nil,1,80); ITMiss(211995,nil,3,80); ITCheck(571,211995) end",
    );
}

#[test]
fn returned_dtos_lines_and_colors_are_fresh_and_mutations_are_isolated() {
    probe(
        &seeded_env(),
        r#"
        local a,b = ITCheck(604,211995,nil,1,80),ITCheck(604,211995,nil,1,80)
        assert(not rawequal(a,b))
        assert(not rawequal(a.lines,b.lines))
        for i=1,8 do
            assert(not rawequal(a.lines[i],b.lines[i]))
            assert(not rawequal(a.lines[i].leftColor,b.lines[i].leftColor))
        end
        a.lines[1].leftColor.r = 0; a.lines[1].leftText = 'mutated'
        a.lines[6] = nil; a.id = 99; a.extra = true
        ITEqual(b,ITCheck(604,211995,nil,1,80))
        local m,n=ITMiss(211995,nil,99),ITMiss(211995,nil,99)
        assert(not rawequal(m,n))
        assert(not rawequal(m.lines,n.lines))
        m.lines[1]={leftText='invented'}; ITEqual(n,ITMiss(211995,nil,99))
    "#,
    );
}

#[test]
fn live_host_replace_and_clear_affect_new_reads_not_old_dtos() {
    let env = seeded_env();
    probe(&env, "ITOld=ITCheck(601,211995,nil,1)");
    env.state()
        .borrow_mut()
        .item_tooltip_levels
        .insert(key(ITEM, Some(1), None), 608);
    probe(
        &env,
        "ITCheck(608,211995,nil,1); assert(ITOld.lines[2].leftText == 'Item Level 601')",
    );
    env.state().borrow_mut().item_tooltip_levels.clear();
    probe(
        &env,
        "ITMiss(211995,nil,1); ITCheck(571,211995); assert(ITOld.lines[2].leftText == 'Item Level 601')",
    );
}

#[test]
fn independent_environments_do_not_share_variant_inputs() {
    let a = seeded_env();
    let b = empty_env();
    probe(&a, "ITCheck(604,211995,nil,1,80)");
    probe(&b, "ITMiss(211995,nil,1,80); ITCheck(571,211995)");
    b.state()
        .borrow_mut()
        .item_tooltip_levels
        .insert(key(ITEM, Some(1), Some(80)), 608);
    probe(&b, "ITCheck(608,211995,nil,1,80)");
    probe(&a, "ITCheck(604,211995,nil,1,80)");
}

#[test]
fn public_tainted_queries_retain_meaningful_context_and_caller_trust() {
    probe(
        &seeded_env(),
        "ITContexts(function() ITCheck(601,211995,nil,1); ITCheck(602,211995,nil,2); ITCheck(603,211995,nil,1,70); ITCheck(604,211995,nil,1,80); ITMiss(211995,nil,3) end)",
    );
}

#[test]
fn secure_authentic_secret_numbers_select_meaningful_item_context_and_treasure() {
    secret_probe(
        "assert(issecure()); ITCheck(601,ITItem,nil,1); ITCheck(602,211995,nil,ITTwo); ITCheck(603,211995,nil,1,IT70); ITCheck(604,ITItem,nil,ITOne,IT80); assert(issecure())",
    );
}

#[test]
fn secure_authentic_secret_nil_optionals_follow_nil_default_and_exact_keys() {
    secret_probe(
        "assert(issecure()); ITEqual(ITBase,ITCheck(571,211995,ITNil,ITNil,ITNil)); ITCheck(601,211995,ITNil,ITOne,ITNil); ITMiss(211995,nil,ITNil,IT70)",
    );
}

#[test]
fn secure_secret_quality_is_authenticated_but_ignored_regardless_of_payload() {
    secret_probe(
        r#"
        assert(issecure())
        for _,q in ipairs(ITSecrets) do ITCheck(604,211995,q,1,80) end
    "#,
    );
}

#[test]
fn secure_secret_wrong_type_contexts_reach_validation_without_private_leaks() {
    secret_probe(
        r#"
        assert(issecure())
        for _,v in ipairs({ITBool,ITString,ITSecretTable,ITSecretFrame}) do
            ITReject(false,211995,nil,v,nil); ITReject(false,211995,nil,1,v)
        end
    "#,
    );
}

#[test]
fn tainted_secrets_all_four_positions_gate_before_public_invalid_types_and_lookup() {
    secret_probe(
        r#"
        ITTainted(function()
            ITReject(true,ITItem,nil,1,80)
            for _,v in ipairs(ITSecrets) do
                ITReject(true,v,nil,1,80)
                ITReject(true,211995,v,1,80)
                ITReject(true,211995,nil,v,80)
                ITReject(true,211995,nil,1,v)
                ITReject(true,'PRIVATE-ItemContext-invalid-item',v,false,{})
                ITReject(true,'PRIVATE-ItemContext-invalid-item',nil,v,false)
                ITReject(true,'PRIVATE-ItemContext-invalid-item',nil,false,v)
            end
            ITReject(true,211995,nil,false,IT80)
            ITReject(true,4294967295,nil,99,IT80)
            ITReject(true,ITItem,nil,false,false)
        end)
    "#,
    );
}

#[test]
fn rooted_wrappers_survive_gc_denial_and_public_recovery_without_trust_change() {
    secret_probe(
        r#"
        collectgarbage('collect')
        ITCheck(604,ITItem,ITNil,ITOne,IT80)
        ITTainted(function()
            ITReject(true,211995,ITBool,1,80)
            collectgarbage('collect')
            ITReject(true,211995,nil,ITSecretFrame,IT80)
            ITCheck(603,211995,nil,1,70)
        end)
        collectgarbage('collect')
        ITCheck(601,ITItem,ITNil,ITOne,ITNil)
        for _,v in ipairs(ITSecrets) do assert(issecretvalue(v)) end
        ITProperties(); assert(issecure())
    "#,
    );
}
