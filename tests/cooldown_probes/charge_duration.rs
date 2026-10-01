//! Bounded 12.0.5 charge queries: no-data policy is simulator inference.
//! The patch's max-charge zero-span rule is firm, but needs explicit charge input.
//! Proposed next fixtures: spell 19750, action 17, player book slot 5;
//! clock 20, (current, max, start, duration, rate) = (2, 2, 12, 40, 2)
//! for max charges, and (1, 2, 12, 40, 2) for active recharge.
//! No charge state is inferred from the ordinary cooldown seeded below.

use super::{SpellCooldownState, WowLuaEnv, env};
use std::time::{Duration, Instant};

fn seed_noncharge_spell_with_cooldown(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.start_time = Instant::now() - Duration::from_secs(20);
    state.action_bars.insert(17, 19750);
    state.spell_id_aliases.insert("fixture heal".into(), 19750);
    state.spell_cooldowns.insert(
        19750,
        SpellCooldownState {
            start: 12.0,
            duration: 40.0,
        },
    );
    state.gcd = Some((18.0, 30.0));
}

#[test]
fn charge_duration_noncharge_spell_does_not_fabricate_recharge_on_any_surface() {
    let env = env();
    seed_noncharge_spell_with_cooldown(&env);
    env.exec(
        r#"
        assert(C_SpellBook.GetSpellBookItemInfo(5, 0).spellID == 19750,
            'fixture must resolve the same spell through the book')
        assert(C_Spell.GetSpellChargeDuration(19750) == nil,
            'ordinary cooldown and GCD are not spell charge input')
        assert(C_ActionBar.GetActionChargeDuration(17) == nil,
            'action assignment must not invent charges')
        assert(C_SpellBook.GetSpellBookItemChargeDuration(5, 0) == nil,
            'spellbook membership must not invent charges')
        "#,
    )
    .expect("unmodeled charge state returns no recharge object on all three APIs");
}

#[test]
fn charge_duration_unknown_spell_and_alias_return_no_recharge() {
    let env = env();
    seed_noncharge_spell_with_cooldown(&env);
    env.exec(
        r#"
        assert(C_Spell.GetSpellChargeDuration(99999) == nil,
            'unknown numeric spell must not fabricate recharge')
        assert(C_Spell.GetSpellChargeDuration('unmapped charge spell') == nil,
            'unresolved alias must not fabricate recharge')
        assert(C_Spell.GetSpellChargeDuration('FIXTURE HEAL') == nil,
            'resolved alias without charge input still has no recharge')
        "#,
    )
    .expect("unknown spells and unmodeled aliases have no charge duration");
}

#[test]
fn charge_duration_empty_action_and_unresolved_book_have_no_recharge() {
    let env = env();
    seed_noncharge_spell_with_cooldown(&env);
    env.state().borrow_mut().action_bars.remove(&99);
    env.exec(
        r#"
        assert(C_ActionBar.GetActionChargeDuration(99) == nil,
            'empty action must not fabricate recharge')
        assert(C_SpellBook.GetSpellBookItemChargeDuration(99999, 0) == nil,
            'unresolved book slot must not fabricate recharge')
        assert(C_SpellBook.GetSpellBookItemChargeDuration(5, 1) == nil,
            'player slot must not resolve through the pet bank')
        assert(C_SpellBook.GetSpellBookItemChargeDuration(5, 99) == nil,
            'unresolved bank must not fabricate recharge')
        "#,
    )
    .expect("empty action and unresolved spellbook identity have no charge duration");
}

#[test]
fn charge_query_tables_do_not_fabricate_counts_without_charge_input() {
    let env = env();
    seed_noncharge_spell_with_cooldown(&env);
    env.state().borrow_mut().action_bars.remove(&99);
    env.exec(
        r#"
        assert(C_Spell.GetSpellCharges(19750) == nil,
            'spell table must not fabricate counts from ordinary cooldown')
        assert(C_ActionBar.GetActionCharges(17) == nil,
            'action table must agree with absent spell charge state')
        assert(C_Spell.GetSpellCharges('FIXTURE HEAL') == nil,
            'alias must agree with numeric spell charge state')
        assert(C_Spell.GetSpellCharges(99999) == nil,
            'unknown spell must not fabricate a charge table')
        assert(C_ActionBar.GetActionCharges(99) == nil,
            'empty action must not fabricate a charge table')
        "#,
    )
    .expect("spell and action charge tables agree on absent explicit charge input");
}
