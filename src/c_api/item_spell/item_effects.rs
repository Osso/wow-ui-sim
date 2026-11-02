//! Item-effect catalog selection for item spell queries.

use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use sim_data::item_effects::{ITEM_EFFECTS, ITEM_SPELL_NAMES};
use sim_data::item_enchants::ENCHANT_SPELLS;

fn use_spell_for_item(item: u32) -> Option<u32> {
    let start = ITEM_EFFECTS.partition_point(|row| row.0 < item);
    let end = ITEM_EFFECTS.partition_point(|row| row.0 <= item);
    let mut slots = [None; 5];
    for &(_, _, spell, trigger, slot, _, _) in &ITEM_EFFECTS[start..end] {
        let slot = usize::try_from(slot)
            .ok()
            .filter(|slot| *slot < slots.len())?;
        if slots[slot].is_some() {
            return None;
        }
        slots[slot] = Some((spell, trigger));
    }
    slots
        .into_iter()
        .flatten()
        .find_map(|(spell, trigger)| (trigger == 0 && spell != 0).then_some(spell))
}

fn equipped_enchant_spell(state: &LuaState, item: u32) -> LuaResult<Option<(u32, &'static str)>> {
    let sim = borrow_state(state)?;
    let mut equipment: Vec<_> = sim.player.equipped_items.iter().collect();
    equipment.sort_unstable_by_key(|(slot, _)| *slot);
    let Some((_, owned)) = equipment
        .into_iter()
        .find(|(_, owned)| owned.item_id == item)
    else {
        return Ok(None);
    };
    let Ok(index) = ENCHANT_SPELLS.binary_search_by_key(&owned.enchant_id, |entry| entry.0) else {
        return Ok(None);
    };
    let (_, spell, name) = ENCHANT_SPELLS[index];
    Ok(name.filter(|_| spell != 0).map(|name| (spell, name)))
}

pub(super) fn get_item_spell(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let item = super::parse_item_id_key(state, value)
        .ok_or_else(|| runtime_error("C_Item.GetItemSpell: expected itemInfo"))?;
    let selected = match use_spell_for_item(item) {
        Some(spell) => ITEM_SPELL_NAMES
            .binary_search_by_key(&spell, |entry| entry.0)
            .ok()
            .map(|index| (spell, ITEM_SPELL_NAMES[index].1)),
        None => equipped_enchant_spell(state, item)?,
    };
    let Some((spell, name)) = selected else {
        return Ok(0);
    };
    let name = create_string(state, name);
    state.push(name);
    state.push(Val::Num(spell as f64));
    Ok(2)
}
