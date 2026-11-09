//! Forever 1.60.1.70707 Game duration queries: native decisions in R17.
//! Pinned SpellDocumentation.lua:274,327; SpellBookDocumentation.lua:198.
//! Supplied intervals exercise the model; live spell records are not asserted.

use super::{SpellCooldownState, WowLuaEnv, env};
use wow_ui_sim::c_api::charge_state::SpellChargeState;

fn spellbook_fixture() -> (WowLuaEnv, u32) {
    let env = env();
    let spell_id = env
        .eval("return C_SpellBook.GetSpellBookItemInfo(5, 0).spellID")
        .expect("player spellbook slot 5 resolves the fixture spell");
    env.state().borrow_mut().gcd = None;
    (env, spell_id)
}

#[test]
fn forever_spell_cooldown_duration_distinguishes_missing_inactive_and_active() {
    let (env, spell_id) = spellbook_fixture();
    env.exec(
        r#"
        local query = C_Spell.GetSpellCooldownDuration
        assert(type(rawget(C_Spell, 'GetSpellCooldownDuration')) == 'function', 'native Game publication')
        assert(select('#', query(2147483647)) == 0, 'unknown spell returns zero values')
        assert(not pcall(query) and not pcall(query, nil), 'identifier is required')
        assert(not pcall(query, true) and not pcall(query, {}), 'identifier must have native type')
        "#,
    )
    .expect("registered cooldown query validates identifiers and preserves unknown arity");
    for duration in [8.0, 17.0] {
        env.state().borrow_mut().spell_cooldowns.insert(
            spell_id,
            SpellCooldownState {
                start: 90.0,
                duration: 100.0 + duration,
            },
        );
        let total: f64 = env
            .eval(&format!(
                "return C_Spell.GetSpellCooldownDuration({spell_id}, true):GetTotalDuration()"
            ))
            .expect("active cooldown returns a populated object");
        assert_eq!(total, 100.0 + duration);
    }
    env.state().borrow_mut().spell_cooldowns.remove(&spell_id);
    let zero: bool = env
        .eval(&format!(
            "return C_Spell.GetSpellCooldownDuration({spell_id}, true):IsZero()"
        ))
        .expect("supported spell without active cooldown returns a duration object");
    assert!(zero);
}

#[test]
fn forever_charge_duration_distinguishes_missing_active_and_full_charges() {
    let (env, spell_id) = spellbook_fixture();
    env.exec(&format!("ForeverDurationSpellID = {spell_id}"))
        .unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Spell, 'GetSpellChargeDuration')) == 'function', 'native Game spell publication')
        assert(type(rawget(C_SpellBook, 'GetSpellBookItemChargeDuration')) == 'function', 'native Game book publication')
        assert(select('#', C_Spell.GetSpellChargeDuration(ForeverDurationSpellID)) == 0, 'no charge system returns zero values')
        assert(select('#', C_Spell.GetSpellChargeDuration(2147483647)) == 0, 'unknown spell returns zero values')
        assert(select('#', C_SpellBook.GetSpellBookItemChargeDuration(0, 0)) == 0, 'slot zero has no entry')
        assert(select('#', C_SpellBook.GetSpellBookItemChargeDuration(99999, 0)) == 0, 'unknown book entry returns zero values')
        assert(not pcall(C_Spell.GetSpellChargeDuration), 'identifier is required')
        assert(not pcall(C_Spell.GetSpellChargeDuration, {}), 'identifier must have native type')
        assert(not pcall(C_SpellBook.GetSpellBookItemChargeDuration, 5), 'bank is required')
        assert(not pcall(C_SpellBook.GetSpellBookItemChargeDuration, 5, 99), 'unknown bank is an argument error')
        "#,
    )
    .expect("registered charge queries validate selectors and preserve no-system arity");
    for duration in [24.0, 48.0] {
        env.state().borrow_mut().spell_charges.insert(
            spell_id,
            SpellChargeState {
                current_charges: 0,
                max_charges: 2,
                recharge_start: 90.0,
                recharge_duration: duration,
                charge_mod_rate: 1.0,
            },
        );
        let totals: (f64, f64) = env
            .eval(
                "return C_Spell.GetSpellChargeDuration(ForeverDurationSpellID):GetTotalDuration(), C_SpellBook.GetSpellBookItemChargeDuration(5, 0):GetTotalDuration()",
            )
            .expect("both charge queries read the supplied active recharge");
        assert_eq!(totals, (duration, duration));
    }
    env.state()
        .borrow_mut()
        .spell_charges
        .get_mut(&spell_id)
        .unwrap()
        .current_charges = 2;
    env.exec(
        r#"
        assert(C_Spell.GetSpellChargeDuration(ForeverDurationSpellID):IsZero(), 'full charges return zero object')
        assert(C_SpellBook.GetSpellBookItemChargeDuration(5, 0):IsZero(), 'full book charges return zero object')
        "#,
    )
    .expect("supported charge system with no recovery returns zero duration, not nil");
}
