//! Batch51 rows229/243: INFERRED bounded effective direct-spell slot queries.
//! Cached declarations establish slot/bool shapes, not native security or misses.
#![cfg(feature = "retail-12-0-5")]

use std::collections::HashMap;

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string, wrap_secret};
use wow_ui_sim::lua_api::WowLuaEnv;

const LINK: &str = "|cff71d5ff|Hspell:7001|h[Fixture Action]|h|r";

#[derive(Debug, PartialEq)]
struct Inputs {
    bars: HashMap<u32, u32>,
    macros: HashMap<u32, u32>,
    outfits: HashMap<u32, i64>,
    buttons: Vec<(u64, u32)>,
    aliases: HashMap<String, u32>,
}

fn inputs(env: &WowLuaEnv) -> Inputs {
    let state = env.state().borrow();
    Inputs {
        bars: state.action_bars.clone(),
        macros: state.action_macros.clone(),
        outfits: state.action_outfits.clone(),
        buttons: state.action_ui_buttons.clone(),
        aliases: state.spell_id_aliases.clone(),
    }
}

fn empty_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("action slot environment");
    {
        let mut state = env.state().borrow_mut();
        // Replace the seeded Protection Paladin bar; never infer empty defaults.
        state.action_bars.clear();
        state.action_macros.clear();
        state.action_outfits.clear();
        state.action_ui_buttons.clear();
        state.spell_id_aliases.clear();
    }
    env.exec(
        r#"
        function CheckActionSlots(identifier, expected)
            local slots
            local function findResult(...)
                assert(select('#', ...) == 1, 'one table, including chosen empty miss')
                slots = ...
                assert(type(slots) == 'table' and not issecretvalue(slots), 'public slot table')
                assert(#slots == #expected, 'exact slot count')
                local wanted, seen, count = {}, {}, 0
                for _, slot in ipairs(expected) do wanted[slot] = true end
                for key, slot in pairs(slots) do
                    assert(type(key) == 'number' and key % 1 == 0 and key >= 1 and key <= #expected,
                        'dense Lua array keys, not slot-keyed map')
                    assert(type(slot) == 'number' and not issecretvalue(slot) and slot > 0 and slot % 1 == 0,
                        'public positive slot index, not frame')
                    assert(wanted[slot] and not seen[slot], 'exact members without duplicates')
                    seen[slot], count = true, count + 1
                end
                assert(count == #expected, 'no sparse holes or metadata')
            end
            local function hasResult(...)
                assert(select('#', ...) == 1, 'exactly one boolean')
                local value = ...
                assert(type(value) == 'boolean' and not issecretvalue(value), 'public bool')
                assert(value == (#expected > 0), 'Has agrees with effective slots')
            end
            findResult(C_ActionBar.FindSpellActionButtons(identifier))
            hasResult(C_ActionBar.HasSpellActionButtons(identifier))
            return slots
        end
        function RejectActionIdentifier(...)
            for _, name in ipairs({'FindSpellActionButtons', 'HasSpellActionButtons'}) do
                local query = C_ActionBar[name]
                assert(type(query) == 'function', 'real registered paired API required')
                local ok, err = pcall(query, ...)
                assert(not ok and type(err) == 'string' and #err > 0, name .. ' rejects identifier')
            end
        end
        "#,
    )
    .expect("assertion helpers only; no API replacement");
    env
}

fn seeded_env() -> WowLuaEnv {
    let env = empty_env();
    {
        let mut state = env.state().borrow_mut();
        state
            .action_bars
            .extend([(3, 7001), (101, 7001), (5, 7002)]);
        state.action_macros.insert(8, 77);
        state.spell_id_aliases.insert("fixture action".into(), 7001);
        state.spell_id_aliases.insert(LINK.to_lowercase(), 7002);
    }
    env.exec(
        r#"
        assert(select(1, GetActionInfo(3)) == 'spell' and select(2, GetActionInfo(3)) == 7001)
        assert(select(1, GetActionInfo(101)) == 'spell' and select(2, GetActionInfo(101)) == 7001)
        assert(select(1, GetActionInfo(5)) == 'spell' and select(2, GetActionInfo(5)) == 7002)
        assert(select(1, GetActionInfo(8)) == 'macro' and select(2, GetActionInfo(8)) == 77)
        "#,
    )
    .expect("actual GetActionInfo fixture preconditions before paired queries");
    env
}

fn install_host_secrets(env: &WowLuaEnv) {
    env.exec("ActionWrongObject = CreateFrame('Frame'); assert(ActionWrongObject:GetObjectType() == 'Frame')")
        .expect("actual frame, no assumption about VM variant");
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("VM security helpers");
    for (name, text) in [
        ("ActionSecretName", "fixture action"),
        ("ActionSecretLink", LINK),
        ("ActionSecretUnknown", "unknown"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("globally root actual secret STRING");
    }
    for (name, number) in [
        ("ActionSecretNumber", 7001.0),
        ("ActionSecretMissing", 7003.0),
    ] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("globally root actual secret NUMBER");
    }
    let frame = lua.get_global_val("ActionWrongObject");
    assert!(!frame.is_nil());
    lua.state_mut().push(frame);
    let wrapped = wrap_secret(lua.state_mut(), frame).expect("wrap actual frame");
    lua.state_mut().push(wrapped);
    let inserted = lua.set_global_val("ActionSecretObject", wrapped);
    lua.state_mut().pop();
    lua.state_mut().pop();
    inserted.expect("root frame wrapper and original");
}

#[test]
fn replaced_default_bar_returns_one_empty_table_and_false() {
    let env = empty_env();
    assert!(env.state().borrow().action_bars.is_empty());
    env.exec("CheckActionSlots(19750, {}); CheckActionSlots(7001, {})")
        .expect("chosen empty-map miss policy");
}

#[test]
fn two_direct_assignments_return_exact_real_slots_not_spell_ids() {
    seeded_env()
        .exec("CheckActionSlots(7001, {3, 101})")
        .unwrap();
}

#[test]
fn unrelated_spell_is_excluded_from_singleton_query() {
    seeded_env().exec("CheckActionSlots(7002, {5})").unwrap();
}

#[test]
fn unknown_and_known_unslotted_spells_and_unseeded_strings_miss() {
    let env = seeded_env();
    env.state().borrow_mut().known_spells.insert(7004);
    assert!(env.state().borrow().known_spells.contains(&7004));
    env.exec("CheckActionSlots(7003, {}); CheckActionSlots(7004, {}); CheckActionSlots(77, {}); CheckActionSlots('unknown', {}); CheckActionSlots('', {}); CheckActionSlots('7001', {}); CheckActionSlots('|Hspell:7001|h[Fixture Action]|h', {})")
        .expect("neither catalog membership, macro ID nor string parsing creates assignment");
}

#[test]
fn explicit_uppercase_name_alias_resolves_shared_key() {
    seeded_env()
        .exec("CheckActionSlots('FIXTURE ACTION', {3, 101})")
        .unwrap();
}

#[test]
fn full_colored_link_alias_overrides_embedded_spell_id() {
    seeded_env()
        .exec(&format!("CheckActionSlots('{LINK}', {{5}})"))
        .unwrap();
}

#[test]
fn numeric_alias_precedes_identity_and_numeric_string_requires_seed() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("7001".into(), 7002);
    env.exec("CheckActionSlots(7001, {5}); CheckActionSlots('7001', {5})")
        .unwrap();
    env.state().borrow_mut().spell_id_aliases.remove("7001");
    env.exec("CheckActionSlots(7001, {3, 101}); CheckActionSlots('7001', {})")
        .unwrap();
}

#[test]
fn alias_replace_and_remove_change_both_queries_immediately() {
    let env = seeded_env();
    env.exec("CheckActionSlots('fixture action', {3, 101})")
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture action".into(), 7002);
    env.exec("CheckActionSlots('fixture action', {5})").unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .remove("fixture action");
    env.exec("CheckActionSlots('fixture action', {})").unwrap();
}

#[test]
fn macro_and_outfit_priority_hide_same_spell_assignments() {
    let env = seeded_env();
    {
        let mut state = env.state().borrow_mut();
        state.action_macros.insert(3, 78);
        state.action_outfits.insert(101, 900);
        state.action_macros.insert(101, 79);
    }
    env.exec("assert(select(1, GetActionInfo(3)) == 'macro'); assert(select(1, GetActionInfo(101)) == 'outfit')")
        .expect("non-spell effective kinds before query, outfit wins macro");
    env.exec("CheckActionSlots(7001, {}); CheckActionSlots(7002, {5})")
        .unwrap();
    env.state().borrow_mut().action_macros.remove(&3);
    env.exec("assert(select(1, GetActionInfo(3)) == 'spell'); CheckActionSlots(7001, {3})")
        .unwrap();
}

#[test]
fn actual_registered_ui_frame_does_not_create_a_spell_assignment() {
    let env = seeded_env();
    env.exec("ActionUIFrame = CreateFrame('Frame'); assert(ActionUIFrame:GetObjectType() == 'Frame'); C_ActionBar.RegisterActionUIButton(ActionUIFrame, 7003)")
        .expect("existing real UI registration API");
    assert_eq!(env.state().borrow().action_ui_buttons.len(), 1);
    assert_eq!(env.state().borrow().action_ui_buttons[0].1, 7003);
    env.exec("assert(GetActionInfo(7003) == nil); CheckActionSlots(7003, {}); CheckActionSlots(7001, {3, 101})").unwrap();
}

#[test]
fn public_move_then_host_clear_and_replace_are_live() {
    let env = seeded_env();
    env.exec("CheckActionSlots(7001, {3, 101}); assert(C_ActionBar.PutActionInSlot(101, 12)); assert(GetActionInfo(101) == nil); assert(select(2, GetActionInfo(12)) == 7001); CheckActionSlots(7001, {3, 12})")
        .expect("actual source,target order");
    env.state().borrow_mut().action_bars.remove(&3);
    env.exec("CheckActionSlots(7001, {12})").unwrap();
    env.state().borrow_mut().action_bars.insert(12, 7002);
    env.exec("CheckActionSlots(7001, {}); CheckActionSlots(7002, {5, 12})")
        .unwrap();
    env.state().borrow_mut().action_bars.clear();
    env.exec("CheckActionSlots(7002, {})").unwrap();
    assert_eq!(env.state().borrow().action_macros.get(&8), Some(&77));
}

#[test]
fn identifier_u32_endpoints_are_valid_but_slot_zero_is_not_a_result() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .action_bars
        .extend([(20, 0), (21, u32::MAX), (0, 7001)]);
    env.exec("CheckActionSlots(0, {20}); CheckActionSlots(4294967295, {21}); CheckActionSlots(7001, {3, 101})").unwrap();
}

#[test]
fn invalid_required_values_reject_before_alias_coercion() {
    let env = seeded_env();
    for key in ["0", "7001", "4294967295", "\u{fffd}"] {
        env.state()
            .borrow_mut()
            .spell_id_aliases
            .insert(key.into(), 7002);
    }
    let before = inputs(&env);
    env.exec(
        r#"
        RejectActionIdentifier(); RejectActionIdentifier(nil)
        local frame = CreateFrame('Frame')
        assert(frame:GetObjectType() == 'Frame')
        for _, value in ipairs({false, true, {}, function() end, coroutine.create(function() end),
            frame, 0/0, math.huge, -math.huge, -1, 7001.5, 4294967296,
            string.char(255), string.char(192, 175)}) do
            RejectActionIdentifier(value)
        end
        CheckActionSlots(7002, {5})
        "#,
    )
    .expect("INFERRED strict public boundary before potentially coerced aliases");
    assert_eq!(inputs(&env), before);
}

#[test]
fn environments_isolate_existing_inputs_and_queries_are_read_only() {
    let first = seeded_env();
    let second = seeded_env();
    let before = inputs(&first);
    first.exec("CheckActionSlots(7001, {3, 101}); CheckActionSlots('fixture action', {3, 101}); CheckActionSlots(7003, {})").unwrap();
    assert_eq!(inputs(&first), before);
    assert_eq!(inputs(&second), before);
    first.state().borrow_mut().action_bars.remove(&101);
    first
        .state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture action".into(), 7002);
    first
        .exec("CheckActionSlots('fixture action', {5}); CheckActionSlots(7001, {3})")
        .unwrap();
    second
        .exec("CheckActionSlots('fixture action', {3, 101})")
        .unwrap();
    assert_eq!(inputs(&second), before);
}

#[test]
fn returned_tables_are_fresh_and_caller_mutation_never_changes_inputs() {
    let env = seeded_env();
    let before = inputs(&env);
    env.exec(
        r#"
        local caller = {identifier = 'fixture action', expected = {3, 101}, marker = 17}
        local a = CheckActionSlots(caller.identifier, caller.expected)
        local b = CheckActionSlots(caller.identifier, caller.expected)
        assert(not rawequal(a, b), 'fresh success tables')
        a[1], a[2], a.extra = 999, nil, true
        caller.expected[1], caller.identifier = 888, 'unknown'
        CheckActionSlots(7001, {3, 101})
        assert(b[1] ~= 999 and b.extra == nil and caller.marker == 17)
        local c = CheckActionSlots(caller.identifier, {})
        local d = CheckActionSlots(caller.identifier, {})
        assert(not rawequal(c, d), 'fresh empty tables')
        c[1] = 555
        assert(next(d) == nil)
        collectgarbage('collect')
        CheckActionSlots(7003, {}); CheckActionSlots('fixture action', {3, 101})
        "#,
    )
    .unwrap();
    assert_eq!(inputs(&env), before);
}

#[test]
fn ordinary_public_calls_preserve_secure_and_tainted_contexts() {
    let env = seeded_env();
    let before = inputs(&env);
    env.exec(&format!(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            CheckActionSlots(7001, {{3, 101}}); CheckActionSlots('fixture action', {{3, 101}})
            CheckActionSlots('{LINK}', {{5}}); CheckActionSlots('unknown', {{}})
            assert(debug.getstacktaint() == before)
        end
        assert(issecure()); probe(); assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'ActionPublicFixture')
            probe()
            assert(debug.getstacktaint() == 'ActionPublicFixture')
        end
        debug.setobjecttaint(addon, 'ActionPublicFixture')
        addon(); assert(issecure())
        "#,
    ))
    .unwrap();
    assert_eq!(inputs(&env), before);
}

#[test]
fn actual_vm_secrets_reject_after_gc_without_mutation_and_public_calls_recover() {
    let env = seeded_env();
    install_host_secrets(&env);
    let before = inputs(&env);
    env.exec(
        r#"
        ActionSecretRoots = {ActionSecretNumber, ActionSecretMissing, ActionSecretName,
            ActionSecretLink, ActionSecretUnknown, ActionSecretObject}
        local rooted = ActionSecretRoots
        local number, object = rooted[1], rooted[6]
        collectgarbage('collect')
        local function probe()
            local before = debug.getstacktaint()
            for index, value in ipairs(rooted) do
                assert(issecretvalue(value), 'actual VM secret')
                assert(rawequal(value, ActionSecretRoots[index]), 'rooted identity before rejection')
                RejectActionIdentifier(value)
                assert(issecretvalue(value) and rawequal(value, rooted[index]), 'no declassification/replacement')
                assert(debug.getstacktaint() == before)
            end
            assert(rawequal(number, ActionSecretNumber) and rawequal(object, ActionSecretObject))
            assert(rawequal(rooted[2], ActionSecretMissing) and rawequal(rooted[3], ActionSecretName))
            assert(rawequal(rooted[4], ActionSecretLink) and rawequal(rooted[5], ActionSecretUnknown))
            CheckActionSlots(7001, {3, 101}); CheckActionSlots('fixture action', {3, 101})
            CheckActionSlots(7003, {})
            assert(debug.getstacktaint() == before, 'public recovery preserves context')
        end
        assert(issecure()); probe(); assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'ActionSecretFixture')
            probe()
            assert(debug.getstacktaint() == 'ActionSecretFixture')
        end
        debug.setobjecttaint(addon, 'ActionSecretFixture')
        addon(); assert(issecure())
        "#,
    ).expect("INFERRED conservative rejection even secure; native permissions unknown");
    assert_eq!(inputs(&env), before);
}
