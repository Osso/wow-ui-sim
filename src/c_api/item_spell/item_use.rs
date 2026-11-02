//! Item usability from ownership, supplied eligibility, and spell power costs.

use crate::lua_api::SimState;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use sim_data::item_spell_metadata::SPELL_POWER;

fn owns_item(sim: &SimState, item: u32) -> bool {
    sim.player
        .equipped_items
        .values()
        .any(|entry| entry.item_id == item)
        || sim
            .bag_items
            .values()
            .any(|entry| entry.item_id == item && entry.stack_count > 0)
}

fn available_power(sim: &SimState, power_type: i32) -> (i32, i32) {
    if power_type == -2 {
        return (sim.player.health, sim.player.health_max);
    }
    if sim.player.power_type == power_type {
        return (sim.player.power, sim.player.power_max);
    }
    sim.player
        .secondary_powers
        .get(&power_type)
        .map_or((0, 0), |power| (power.current, power.max))
}

fn resources_sufficient(sim: &SimState, spell: u32) -> bool {
    let start = SPELL_POWER.partition_point(|row| row.0 < spell);
    let end = SPELL_POWER.partition_point(|row| row.0 <= spell);
    SPELL_POWER[start..end]
        .iter()
        .all(|&(_, _, _, power_type, flat, _, percent, _, bar, aura)| {
            if aura != 0
                && !sim
                    .player
                    .buffs
                    .iter()
                    .any(|buff| buff.spell_id == aura as i32)
            {
                return true;
            }
            if bar != 0 && sim.item_use_power_bar_id != Some(bar) {
                return false;
            }
            let (current, maximum) = available_power(sim, power_type);
            let cost = if flat > 0 {
                flat
            } else if percent > 0.0 {
                (maximum as f32 * percent / 100.0).round() as i32
            } else {
                0
            };
            if power_type == -2 {
                cost < current
            } else {
                cost <= current
            }
        })
}

pub(super) fn is_usable_item(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let item = super::parse_item_id_key(state, value)
        .ok_or_else(|| runtime_error("C_Item.IsUsableItem: expected itemInfo"))?;
    let owned = {
        let sim = borrow_state(state)?;
        sim.player.present && item != 0 && owns_item(&sim, item)
    };
    let (usable, insufficient) = if owned {
        match super::item_effects::selected_item_spell(state, item)? {
            Some((spell, _)) => {
                let sim = borrow_state(state)?;
                let eligible = sim
                    .item_use_eligibility
                    .get(&(item, spell))
                    .copied()
                    .ok_or_else(|| {
                        runtime_error("C_Item.IsUsableItem: eligibility has not been supplied")
                    })?;
                if eligible {
                    let enough = resources_sufficient(&sim, spell);
                    (enough, !enough)
                } else {
                    (false, false)
                }
            }
            None => (false, false),
        }
    } else {
        (false, false)
    };
    state.push(Val::Bool(usable));
    state.push(Val::Bool(insufficient));
    Ok(2)
}
