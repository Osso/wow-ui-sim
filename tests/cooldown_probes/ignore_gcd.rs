//! Bounded 12.0.5 duration selection; ordinary values, not native secrecy proof.

use super::{SpellCooldownState, WowLuaEnv, env};
use std::time::{Duration, Instant};

fn seed_intervals(env: &WowLuaEnv, spell: bool, gcd: bool) {
    let mut state = env.state().borrow_mut();
    state.start_time = Instant::now() - Duration::from_secs(35);
    state.action_bars.insert(17, 19750);
    state.spell_id_aliases.insert("fixture heal".into(), 19750);
    state.spell_cooldowns.clear();
    if spell {
        state.spell_cooldowns.insert(
            19750,
            SpellCooldownState {
                start: 12.0,
                duration: 37.0,
            },
        );
    }
    state.gcd = gcd.then_some((30.0, 30.0));
}

const CHECK_QUERIES: &str = r#"
    local function check(duration, start, total)
        assert(duration ~= nil, 'expected duration object')
        assert(duration:GetStartTime() == start, 'wrong interval start')
        assert(duration:GetTotalDuration() == total, 'wrong interval duration')
        assert(duration:GetEndTime() == start + total, 'wrong end')
        assert(duration:GetModRate() == 1)
    end
    local queries = {
        function(...) return C_ActionBar.GetActionCooldownDuration(17, ...) end,
        function(...) return C_Spell.GetSpellCooldownDuration(19750, ...) end,
        function(...) return C_Spell.GetSpellCooldownDuration('FIXTURE HEAL', ...) end,
        function(...) return C_SpellBook.GetSpellBookItemCooldownDuration(5, 0, ...) end,
    }
    for _, query in ipairs(queries) do
        check(query(), expectedDefaultStart, expectedDefaultTotal)
        check(query(false), expectedDefaultStart, expectedDefaultTotal)
        check(query(true), expectedSpellStart, expectedSpellTotal)
    end
"#;

fn check_intervals(env: &WowLuaEnv, default: (f64, f64), spell: (f64, f64)) {
    env.exec(&format!(
        "expectedDefaultStart = {}; expectedDefaultTotal = {}; \
         expectedSpellStart = {}; expectedSpellTotal = {}; {}",
        default.0, default.1, spell.0, spell.1, CHECK_QUERIES
    ))
    .unwrap();
}

#[test]
fn ignore_gcd_overlap_selects_individual_interval_on_all_three_surfaces() {
    let env = env();
    seed_intervals(&env, true, true);
    // Player slot 5 is Flash of Light (19750), not spell ID 5.
    let spell_id: i64 = env
        .eval("return C_SpellBook.GetSpellBookItemInfo(5, 0).spellID")
        .unwrap();
    assert_eq!(spell_id, 19750);
    check_intervals(&env, (30.0, 30.0), (12.0, 37.0));
}

#[test]
fn ignore_gcd_only_gcd_returns_zero_individual_interval() {
    let env = env();
    seed_intervals(&env, false, true);
    check_intervals(&env, (30.0, 30.0), (0.0, 0.0));
}

#[test]
fn ignore_gcd_only_spell_and_empty_preserve_zero_object_contract() {
    let env = env();
    seed_intervals(&env, true, false);
    check_intervals(&env, (12.0, 37.0), (12.0, 37.0));
    seed_intervals(&env, false, false);
    check_intervals(&env, (0.0, 0.0), (0.0, 0.0));
}

#[test]
fn ignore_gcd_unknown_inputs_do_not_invent_slot_spell_or_bank_mappings() {
    let env = env();
    seed_intervals(&env, true, true);
    env.exec(
        r#"
        for _, ignore in ipairs({false, true}) do
            local empty = C_ActionBar.GetActionCooldownDuration(99999, ignore)
            assert(empty:GetStartTime() == 0 and empty:GetTotalDuration() == 0)
            local unknown = C_Spell.GetSpellCooldownDuration(99999, ignore)
            assert(unknown:GetStartTime() == (ignore and 0 or 30))
            assert(C_Spell.GetSpellCooldownDuration('unmapped name', ignore) == nil)
            assert(C_Spell.GetSpellCooldownDuration({}, ignore) == nil)
            for _, bank in ipairs({0, 1, 99}) do
                assert(C_SpellBook.GetSpellBookItemCooldownDuration(19750, bank, ignore) == nil)
                assert(C_SpellBook.GetSpellBookItemCooldownDuration(0, bank, ignore) == nil)
            end
            assert(C_SpellBook.GetSpellBookItemCooldownDuration(5, 1, ignore) == nil)
            assert(C_SpellBook.GetSpellBookItemCooldownDuration(5, 99, ignore) == nil)
        end
        assert(C_SpellBook.GetSpellBookItemCooldownDuration(5, nil, true) == nil)
        assert(C_SpellBook.GetSpellBookItemCooldownDuration(nil, 0, true) == nil)
        assert(C_SpellBook.GetSpellBookItemCooldownDuration(5.5, 0, true) == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn ignore_gcd_snapshots_keep_interval_and_runtime_clock_after_model_changes() {
    let env = env();
    seed_intervals(&env, true, true);
    env.exec(
        r#"
        snapshots = {
            C_ActionBar.GetActionCooldownDuration(17, true),
            C_Spell.GetSpellCooldownDuration('fixture heal', true),
            C_SpellBook.GetSpellBookItemCooldownDuration(5, 0, true),
        }
        for _, duration in ipairs(snapshots) do
            local before = GetTime()
            local clock = duration:GetClockTime()
            local after = GetTime()
            assert(clock >= before and clock <= after)
            assert(duration:GetRemainingDuration() > 0)
        end
        "#,
    )
    .unwrap();
    seed_intervals(&env, false, false);
    env.exec(
        r#"
        for _, duration in ipairs(snapshots) do
            assert(duration:GetStartTime() == 12)
            assert(duration:GetTotalDuration() == 37)
            assert(duration:GetEndTime() == 49)
        end
        "#,
    )
    .unwrap();
    check_intervals(&env, (0.0, 0.0), (0.0, 0.0));
}

#[test]
fn ignore_gcd_spell_ending_later_and_expired_spell() {
    let env = env();
    seed_intervals(&env, true, true);
    env.state().borrow_mut().gcd = Some((30.0, 10.0));
    check_intervals(&env, (12.0, 37.0), (12.0, 37.0));
    env.state().borrow_mut().spell_cooldowns.insert(
        19750,
        SpellCooldownState {
            start: 12.0,
            duration: 7.0,
        },
    );
    check_intervals(&env, (30.0, 10.0), (0.0, 0.0));
}
