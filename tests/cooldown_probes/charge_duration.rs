//! Bounded 12.0.5 charge queries: no-data policy is simulator inference.
//! The patch's max-charge zero-span rule is firm, but needs explicit charge input.
//! Concrete fixtures use spell 19750, action 17, player book slot 5.
//! No charge state is inferred from the ordinary cooldown seeded below.

use super::{SpellCooldownState, WowLuaEnv, env};
use std::time::{Duration, Instant};
use wow_ui_sim::c_api::charge_state::SpellChargeState;

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

#[test]
fn charge_zero_span_core_is_fully_elapsed_even_before_start_and_after_reset() {
    let env = env();
    env.exec(
        r#"
        local clock = C_DurationUtil.CreateManualClock(20)
        local d = C_DurationUtil.CreateDuration()
        local function zero()
            assert(d:IsZero() and d:HasExpired(), 'zero span is fully elapsed')
            assert(not d:HasStarted() and not d:IsActive())
            assert(d:GetElapsedPercent() == 1 and d:GetRemainingPercent() == 0)
            assert(d:GetElapsedPercent(1) == 1 and d:GetRemainingPercent(1) == 0)
            assert(d:GetElapsedDuration() == 0 and d:GetRemainingDuration() == 0)
        end
        zero()
        d:SetClock(clock)
        d:SetTimeFromStart(100, 0, 2)
        zero()
        clock:SetTime(200)
        zero()
        d:SetTimeFromStart(190, 40, 2)
        assert(not d:HasExpired() and d:GetElapsedPercent() == 0.5)
        d:Reset()
        zero()
        d:SetToDefaults()
        zero()
        "#,
    )
    .expect(
        "fully elapsed includes expired and elapsed fraction one, an explicit simulator inference",
    );
}

#[test]
fn charge_zero_max_input_is_not_a_configured_charge_spell() {
    let env = env();
    seed_charge(&env, 0);
    env.state()
        .borrow_mut()
        .spell_charges
        .get_mut(&19750)
        .expect("fixture charge row")
        .max_charges = 0;
    env.exec(
        r#"
        assert(C_Spell.GetSpellCharges(19750) == nil)
        assert(C_ActionBar.GetActionCharges(17) == nil)
        assert(C_Spell.GetSpellChargeDuration(19750) == nil)
        assert(C_ActionBar.GetActionChargeDuration(17) == nil)
        assert(C_SpellBook.GetSpellBookItemChargeDuration(5, 0) == nil)
        "#,
    )
    .expect("zero maximum does not represent a configured charge spell");
}

fn seed_charge(env: &WowLuaEnv, current_charges: u32) {
    seed_noncharge_spell_with_cooldown(env);
    env.state().borrow_mut().spell_charges.insert(
        19750,
        SpellChargeState {
            current_charges,
            max_charges: 2,
            recharge_start: 12.0,
            recharge_duration: 40.0,
            charge_mod_rate: 2.0,
        },
    );
}

#[test]
fn charge_duration_active_recharge_uses_shared_interval_rate_and_runtime_clock() {
    let env = env();
    seed_charge(&env, 1);
    env.exec(
        r#"
        local producers = {
            function() return C_Spell.GetSpellChargeDuration('FIXTURE HEAL') end,
            function() return C_ActionBar.GetActionChargeDuration(17) end,
            function() return C_SpellBook.GetSpellBookItemChargeDuration(5, 0) end,
        }
        for _, produce in ipairs(producers) do
            local d = produce()
            assert(d ~= nil, 'configured recharge must have a duration')
            assert(d:GetStartTime() == 12)
            assert(d:GetModRate() == 2)
            assert(d:GetTotalDuration() == 20)
            assert(d:GetTotalDuration(1) == 40)
            assert(d:GetEndTime() == 32)
            assert(not d:IsZero() and not d:HasExpired())
            local before = GetTime()
            local elapsed, remaining = d:GetElapsedDuration(), d:GetRemainingDuration()
            local after = GetTime()
            assert(elapsed >= before - 12 - 1e-6 and elapsed <= after - 12 + 1e-6)
            assert(remaining >= 32 - after - 1e-6 and remaining <= 32 - before + 1e-6)
        end
        assert(C_SpellBook.GetSpellBookItemChargeDuration(5, 1) == nil)
        assert(C_SpellBook.GetSpellBookItemChargeDuration(5, 99) == nil)
        assert(C_ActionBar.GetActionChargeDuration(99) == nil)
        "#,
    )
    .expect("all producers use explicit charge interval and existing duration rate semantics");
}

#[test]
fn charge_duration_at_max_is_zero_span_fully_elapsed_at_query_time() {
    let env = env();
    seed_charge(&env, 2);
    env.exec(
        r#"
        local producers = {
            function() return C_Spell.GetSpellChargeDuration(19750) end,
            function() return C_ActionBar.GetActionChargeDuration(17) end,
            function() return C_SpellBook.GetSpellBookItemChargeDuration(5, 0) end,
        }
        for _, produce in ipairs(producers) do
            local before = GetTime()
            local d = produce()
            local after = GetTime()
            assert(d ~= nil, 'configured maximum charges must return a duration')
            assert(d:GetStartTime() >= before and d:GetStartTime() <= after)
            assert(d:GetStartTime() == d:GetEndTime())
            assert(d:GetTotalDuration() == 0 and d:GetModRate() == 2)
            assert(d:IsZero() and d:HasExpired() and not d:IsActive())
            assert(d:GetElapsedDuration() == 0 and d:GetRemainingDuration() == 0)
        end
        "#,
    )
    .expect("maximum charge rule returns a fully elapsed zero-span snapshot");
}

#[test]
fn charge_queries_derive_all_five_fields_from_same_explicit_state() {
    let env = env();
    seed_charge(&env, 1);
    env.exec(
        r#"
        local spell = C_Spell.GetSpellCharges('fixture heal')
        local action = C_ActionBar.GetActionCharges(17)
        assert(spell ~= nil and action ~= nil, 'configured charge tables must exist')
        for _, info in ipairs({spell, action}) do
            assert(info.currentCharges == 1 and info.maxCharges == 2)
            assert(info.cooldownStartTime == 12 and info.cooldownDuration == 40)
            assert(info.chargeModRate == 2)
        end
        "#,
    )
    .expect("charge table fields are explicit input, not cooldown or display counts");
    seed_charge(&env, 2);
    env.exec(
        r#"
        local spell = C_Spell.GetSpellCharges(19750)
        local action = C_ActionBar.GetActionCharges(17)
        assert(spell ~= nil and action ~= nil, 'maximum charge tables must exist')
        for _, info in ipairs({spell, action}) do
            assert(info.currentCharges == 2 and info.maxCharges == 2)
            assert(info.cooldownStartTime == 12 and info.cooldownDuration == 40)
            assert(info.chargeModRate == 2)
        end
        "#,
    )
    .expect("maximum charges preserve table inputs despite duration's zero span");
}

#[test]
fn charge_duration_snapshots_survive_changed_state_alias_mapping_and_clock() {
    let env = env();
    seed_charge(&env, 1);
    env.exec(
        r#"
        oldSpellCharge = C_Spell.GetSpellChargeDuration('fixture heal')
        oldActionCharge = C_ActionBar.GetActionChargeDuration(17)
        oldBookCharge = C_SpellBook.GetSpellBookItemChargeDuration(5, 0)
        oldChargeInfo = C_Spell.GetSpellCharges(19750)
        assert(oldSpellCharge ~= nil and oldBookCharge ~= nil)
        "#,
    )
    .expect("explicit charge snapshots exist before fixture mutation");
    {
        let mut state = env.state().borrow_mut();
        state.start_time = Instant::now() - Duration::from_secs(50);
        state.spell_charges.insert(
            54321,
            SpellChargeState {
                current_charges: 3,
                max_charges: 4,
                recharge_start: 45.0,
                recharge_duration: 30.0,
                charge_mod_rate: 3.0,
            },
        );
        state.spell_id_aliases.insert("fixture heal".into(), 54321);
        state.action_bars.insert(17, 54321);
        state.spell_charges.remove(&19750);
    }
    env.exec(
        r#"
        for _, old in ipairs({oldSpellCharge, oldActionCharge, oldBookCharge}) do
            assert(old:GetStartTime() == 12 and old:GetEndTime() == 32)
            assert(old:GetModRate() == 2 and old:GetTotalDuration() == 20)
            assert(old:HasExpired() and old:GetElapsedDuration() == 20)
            assert(old:GetRemainingDuration() == 0)
        end
        assert(oldChargeInfo.currentCharges == 1 and oldChargeInfo.chargeModRate == 2)
        local spell = C_Spell.GetSpellChargeDuration('FIXTURE HEAL')
        local action = C_ActionBar.GetActionChargeDuration(17)
        assert(spell ~= nil and action ~= nil, 'remapped charge durations must exist')
        for _, d in ipairs({spell, action}) do
            assert(d:GetStartTime() == 45 and d:GetEndTime() == 55)
            assert(d:GetModRate() == 3 and d:GetTotalDuration() == 10)
            assert(not d:HasExpired())
            local before = GetTime()
            local elapsed = d:GetElapsedDuration()
            local after = GetTime()
            assert(elapsed >= before - 45 - 1e-6 and elapsed <= after - 45 + 1e-6)
        end
        local info = C_Spell.GetSpellCharges('fixture heal')
        assert(info.currentCharges == 3 and info.maxCharges == 4)
        assert(info.cooldownStartTime == 45 and info.cooldownDuration == 30 and info.chargeModRate == 3)
        assert(C_Spell.GetSpellChargeDuration(19750) == nil)
        assert(C_SpellBook.GetSpellBookItemChargeDuration(5, 0) == nil)
        "#,
    )
    .expect("new identity/input and clock affect new queries, not existing snapshots");
}
