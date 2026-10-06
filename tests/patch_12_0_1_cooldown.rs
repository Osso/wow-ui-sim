//! Retained hotfix contracts driven by cast and explicit host charge transitions.
#![cfg(feature = "client-retail")]

use std::time::{Duration, Instant};
use wow_ui_sim::c_api::charge_state::SpellChargeState;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_12_0_1_cooldown_charge_formula_and_zero_span() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().action_bars.insert(17, 19750);
    // INFERRED: charge progression is a host input, not autonomous spell simulation.
    for (max, current, start, duration, active) in [
        (3, 1, 12.0, 40.0, true),
        (3, 3, 12.0, 40.0, false),
        (1, 0, 12.0, 40.0, false),
        (3, 1, 0.0, 40.0, false),
        (3, 1, 12.0, 0.0, false),
    ] {
        {
            let mut state = env.state().borrow_mut();
            state.start_time = Instant::now() - Duration::from_secs(20);
            state.cooldowns_restricted = true;
            state.spell_charges.insert(19750, SpellChargeState {
                current_charges: current, max_charges: max,
                recharge_start: start, recharge_duration: duration, charge_mod_rate: 2.0,
            });
        }
        env.exec(&format!(r#"
            for _, query in ipairs({{
                function() return C_Spell.GetSpellCharges(19750) end,
                function() return C_ActionBar.GetActionCharges(17) end,
                function() return C_SpellBook.GetSpellBookItemCharges(5, 0) end,
            }}) do
                local info = query()
                assert(type(info.isActive) == 'boolean' and info.isActive == {active})
                assert(not issecretvalue(info.isActive) and not issecretvalue(info.maxCharges))
                assert(info.maxCharges == {max})
                assert(issecretvalue(info.currentCharges) and issecretvalue(info.cooldownStartTime))
                assert(issecretvalue(info.cooldownDuration) and issecretvalue(info.chargeModRate))
            end
            for _, query in ipairs({{
                function() return C_Spell.GetSpellChargeDuration(19750) end,
                function() return C_ActionBar.GetActionChargeDuration(17) end,
                function() return C_SpellBook.GetSpellBookItemChargeDuration(5, 0) end,
            }}) do
                local object = query()
                assert(object ~= nil and object:IsZero() == {inactive})
                assert(object:GetTotalDuration() == {span})
            end
        "#, inactive = !active, span = if active {duration / 2.0} else {0.0})).unwrap();
    }
}

#[test]
fn patch_12_0_1_cooldown_nonpositive_intervals_are_inactive() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().action_bars.insert(17, 19750);
    for (start, duration) in [(0.0, 100.0), (-1.0, 100.0), (20.0, 0.0)] {
        env.state().borrow_mut().spell_cooldowns.insert(19750,
            wow_ui_sim::lua_api::state::SpellCooldownState { start, duration });
        env.exec(r#"
            for _, info in ipairs({C_Spell.GetSpellCooldown(19750), C_ActionBar.GetActionCooldown(17)}) do
                assert(info.isActive == false, 'nonpositive interval must not render')
            end
            for _, object in ipairs({C_Spell.GetSpellCooldownDuration(19750), C_ActionBar.GetActionCooldownDuration(17)}) do
                assert(object:IsZero() and object:GetTotalDuration() == 0)
            end
        "#).unwrap();
    }
}

#[test]
fn patch_12_0_1_cooldown_cast_expiry_transitions() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().action_bars.insert(17, 642);
    env.exec(r#"
        local function check(active)
            local spell = C_Spell.GetSpellCooldown(642)
            local action = C_ActionBar.GetActionCooldown(17)
            for _, info in ipairs({spell, action}) do
                assert(not issecretvalue(info.isEnabled) and not issecretvalue(info.isActive))
                assert(info.isActive == (info.isEnabled and info.startTime > 0 and info.duration > 0))
                assert(info.isActive == active)
            end
            for _, d in ipairs({C_Spell.GetSpellCooldownDuration(642), C_ActionBar.GetActionCooldownDuration(17)}) do
                assert(d:IsZero() == not active)
                if not active then assert(d:GetTotalDuration() == 0) end
            end
        end
        CheckCastCooldown = check
        check(false)
        CastSpellByID(642)
        check(true)
        CastSnapshot = C_Spell.GetSpellCooldown(642)
    "#).unwrap();
    env.state().borrow_mut().start_time -= Duration::from_secs(301);
    env.exec("CheckCastCooldown(false); assert(CastSnapshot.isActive == true and CastSnapshot.duration == 300)").unwrap();
}
