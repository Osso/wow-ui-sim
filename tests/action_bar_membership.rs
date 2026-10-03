//! Row245: INFERRED public direct-spell membership, not native special-bar parity.
//! Aliases below are explicit simulator registry policy, not native grammar.
#![cfg(feature = "retail-12-0-5")]

use std::collections::HashMap;

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

fn snapshot_inputs(env: &WowLuaEnv) -> Inputs {
    let state = env.state().borrow();
    Inputs {
        bars: state.action_bars.clone(),
        macros: state.action_macros.clone(),
        outfits: state.action_outfits.clone(),
        buttons: state.action_ui_buttons.clone(),
        aliases: state.spell_id_aliases.clone(),
    }
}

fn seed_membership_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("membership environment");
    {
        let mut state = env.state().borrow_mut();
        // Replace seeded bars explicitly; no assumption about default emptiness.
        state.action_bars.clear();
        state.action_macros.clear();
        state.action_outfits.clear();
        state.action_ui_buttons.clear();
        state.spell_id_aliases.clear();
        state
            .action_bars
            .extend([(3, 7001), (101, 7001), (5, 7002)]);
        state.spell_id_aliases.insert("fixture action".into(), 7001);
    }
    env.exec(
        r#"
        function CheckMembership(identifier, expected)
            local function checkResult(...)
                assert(select('#', ...) == 1, 'exactly one membership result')
                local value = ...
                assert(type(value) == 'boolean', 'membership boolean')
                assert(value == expected, 'effective direct-spell membership')
            end
            checkResult(C_ActionBar.IsOnBarOrSpecialBar(identifier))
        end
        assert(select(1, GetActionInfo(3)) == 'spell' and select(2, GetActionInfo(3)) == 7001)
        assert(select(1, GetActionInfo(101)) == 'spell' and select(2, GetActionInfo(101)) == 7001)
        assert(select(1, GetActionInfo(5)) == 'spell' and select(2, GetActionInfo(5)) == 7002)
        "#,
    )
    .expect("actual slot preconditions and assertion helper, no API replacement");
    env
}

#[test]
fn effective_direct_spell_assignment_matches() {
    seed_membership_env()
        .exec("CheckMembership(7001, true); CheckMembership(7002, true)")
        .unwrap();
}

#[test]
fn unassigned_spell_and_empty_bar_miss() {
    let env = seed_membership_env();
    env.exec("CheckMembership(7003, false)").unwrap();
    env.state().borrow_mut().action_bars.clear();
    env.exec("CheckMembership(7001, false); CheckMembership(7002, false)")
        .unwrap();
}

#[test]
fn duplicate_membership_survives_one_removal_then_disappears_after_last() {
    let env = seed_membership_env();
    env.exec("CheckMembership(7001, true)").unwrap();
    env.state().borrow_mut().action_bars.remove(&3);
    env.exec("assert(GetActionInfo(3) == nil); CheckMembership(7001, true)")
        .unwrap();
    env.state().borrow_mut().action_bars.remove(&101);
    env.exec("assert(GetActionInfo(101) == nil); CheckMembership(7001, false); CheckMembership(7002, true)")
        .unwrap();
}

#[test]
fn slot_zero_alone_does_not_create_effective_membership() {
    let env = seed_membership_env();
    {
        let mut state = env.state().borrow_mut();
        state.action_bars.clear();
        state.action_bars.insert(0, 7001);
    }
    env.exec("CheckMembership(7001, false)").unwrap();
}

#[test]
fn registered_lowercase_name_resolves_case_normalized_public_input() {
    seed_membership_env()
        .exec("CheckMembership('fixture action', true); CheckMembership('FIXTURE ACTION', true); CheckMembership('unknown', false)")
        .unwrap();
}

#[test]
fn name_registry_replace_and_remove_are_live() {
    let env = seed_membership_env();
    env.exec("CheckMembership('fixture action', true)").unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture action".into(), 7003);
    env.exec("CheckMembership('fixture action', false); CheckMembership(7001, true)")
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture action".into(), 7002);
    env.exec("CheckMembership('fixture action', true)").unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .remove("fixture action");
    env.exec("CheckMembership('fixture action', false)")
        .unwrap();
}

#[test]
fn registered_numeric_key_overrides_identity_and_string_requires_registry() {
    let env = seed_membership_env();
    env.exec("CheckMembership(7001, true); CheckMembership('7001', false)")
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("7001".into(), 7003);
    env.exec("CheckMembership(7001, false); CheckMembership('7001', false)")
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("7001".into(), 7002);
    env.exec("CheckMembership(7001, true); CheckMembership('7001', true)")
        .unwrap();
    env.state().borrow_mut().spell_id_aliases.remove("7001");
    env.exec("CheckMembership(7001, true); CheckMembership('7001', false)")
        .unwrap();
}

#[test]
fn full_link_is_only_an_explicit_registry_alias_not_embedded_id_grammar() {
    let env = seed_membership_env();
    env.exec(&format!("CheckMembership('{LINK}', false)"))
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert(LINK.to_lowercase(), 7003);
    env.exec(&format!("CheckMembership('{LINK}', false)"))
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert(LINK.to_lowercase(), 7002);
    env.exec(&format!("CheckMembership('{LINK}', true)"))
        .unwrap();
}

#[test]
fn macro_and_outfit_overrides_exclude_direct_spell_until_removed() {
    let env = seed_membership_env();
    {
        let mut state = env.state().borrow_mut();
        state.action_macros.insert(3, 78);
        state.action_outfits.insert(101, 900);
        state.action_macros.insert(101, 79);
    }
    env.exec(
        r#"
        assert(select(1, GetActionInfo(3)) == 'macro' and select(2, GetActionInfo(3)) == 78)
        assert(select(1, GetActionInfo(101)) == 'outfit' and select(2, GetActionInfo(101)) == 900)
        CheckMembership(7001, false)
        CheckMembership(7002, true)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().action_outfits.remove(&101);
    env.exec("assert(select(1, GetActionInfo(101)) == 'macro'); CheckMembership(7001, false)")
        .unwrap();
    env.state().borrow_mut().action_macros.remove(&101);
    env.exec("assert(select(1, GetActionInfo(101)) == 'spell'); CheckMembership(7001, true)")
        .unwrap();
}

#[test]
fn membership_queries_leave_existing_inputs_unchanged() {
    let env = seed_membership_env();
    {
        let mut state = env.state().borrow_mut();
        state.action_macros.insert(3, 78);
        state.action_outfits.insert(5, 900);
    }
    let before = snapshot_inputs(&env);
    env.exec("CheckMembership(7001, true); CheckMembership('FIXTURE ACTION', true); CheckMembership(7002, false); CheckMembership(7003, false); CheckMembership('unknown', false)")
        .unwrap();
    assert_eq!(snapshot_inputs(&env), before);
}

#[test]
fn membership_and_alias_mutations_are_environment_local() {
    let first = seed_membership_env();
    let second = seed_membership_env();
    let second_before = snapshot_inputs(&second);
    {
        let mut state = first.state().borrow_mut();
        state.action_bars.remove(&3);
        state.action_bars.remove(&101);
        state.spell_id_aliases.insert("fixture action".into(), 7003);
    }
    first
        .exec("CheckMembership(7001, false); CheckMembership('fixture action', false)")
        .unwrap();
    second
        .exec("CheckMembership(7001, true); CheckMembership('fixture action', true)")
        .unwrap();
    assert_eq!(snapshot_inputs(&second), second_before);
}
