//! Explicit retail inputs. Defaults and numeric interpretations are simulator guesses.
//! See docs/specs/retail-missing-stat-inputs.md; Forever handlers remain unchanged.

use crate::c_api::c_secrets::push_stat_number;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn get_mastery(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.player.stats.mastery_pct();
    push_stat_number(state, value)?;
    Ok(1)
}

fn get_override_ap_by_spell_power(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .player
        .stats
        .spell_power_to_attack_power;
    push_stat_number(state, value)?;
    Ok(1)
}

fn get_override_spell_power_by_ap(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .player
        .stats
        .attack_power_to_spell_power;
    push_stat_number(state, value)?;
    Ok(1)
}

fn get_spell_penetration(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.player.stats.spell_penetration;
    push_stat_number(state, value)?;
    Ok(1)
}

fn get_sturdiness(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.player.stats.sturdiness_pct;
    push_stat_number(state, value)?;
    Ok(1)
}

fn push_power_regen(state: &mut LuaState, power_type: i32) -> LuaResult<u32> {
    let regen = borrow_state(state)?
        .player
        .power_regen
        .get(&power_type)
        .copied()
        .unwrap_or_default();
    push_stat_number(state, regen.base)?;
    push_stat_number(state, regen.casting)?;
    Ok(2)
}

fn get_power_regen(state: &mut LuaState) -> LuaResult<u32> {
    let power_type = borrow_state(state)?.player.power_type;
    push_power_regen(state, power_type)
}

fn get_power_regen_for_power_type(state: &mut LuaState) -> LuaResult<u32> {
    // Only this documented AllowedWhenUntainted selector is decoded by the VM.
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(number) = value else {
        return Err(rilua::runtime_error(
            "GetPowerRegenForPowerType requires a power type number",
        ));
    };
    let power_type = number as i32;
    if f64::from(power_type) != number {
        return Err(rilua::runtime_error(
            "GetPowerRegenForPowerType requires an i32 power type",
        ));
    }
    push_power_regen(state, power_type)
}

fn player_effective_attack_power(state: &mut LuaState) -> LuaResult<u32> {
    let Some(power) = borrow_state(state)?.player.effective_attack_power else {
        return Ok(0);
    };
    for value in [
        power.main_hand,
        power.off_hand,
        power.ranged,
        power.base,
        power.base_ranged,
    ] {
        push_stat_number(state, value)?;
    }
    Ok(5)
}

pub(super) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetMastery", get_mastery)?;
    LuaApiMut::register_function(
        lua,
        "GetOverrideAPBySpellPower",
        get_override_ap_by_spell_power,
    )?;
    LuaApiMut::register_function(
        lua,
        "GetOverrideSpellPowerByAP",
        get_override_spell_power_by_ap,
    )?;
    LuaApiMut::register_function(lua, "GetSpellPenetration", get_spell_penetration)?;
    LuaApiMut::register_function(lua, "GetSturdiness", get_sturdiness)?;
    LuaApiMut::register_function(lua, "GetPowerRegen", get_power_regen)?;
    LuaApiMut::register_function(
        lua,
        "GetPowerRegenForPowerType",
        get_power_regen_for_power_type,
    )?;
    LuaApiMut::register_function(
        lua,
        "PlayerEffectiveAttackPower",
        player_effective_attack_power,
    )?;
    Ok(())
}
