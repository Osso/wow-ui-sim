//! Inferred Forever junk policy: sellable poor-quality stack units in carried slots.

use crate::c_api::item_spell::{
    BAG_SLOT_FLAG_EXCLUDE_JUNK_SELL, bag_slot_flag_is_set, container_slot_count,
};
use crate::items::{self, ItemInfo};
use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Count known sellable junk units in one slot without changing inventory.
/// Unknown catalog entries cannot establish eligibility; positive cases are
/// tested with concrete metadata fixtures until the live catalog contains junk.
pub fn junk_stack_quantity(
    (bag, slot): (i32, i32),
    quantity: i32,
    item: Option<&ItemInfo>,
    excluded: bool,
) -> u64 {
    if excluded
        || !(0..=4).contains(&bag)
        || !(1..=container_slot_count(bag)).contains(&slot)
        || quantity <= 0
    {
        return 0;
    }
    match item {
        Some(item) if item.quality == 0 && item.sell_price > 0 => quantity as u64,
        _ => 0,
    }
}

pub(super) fn get_num_junk_items(state: &mut LuaState) -> LuaResult<u32> {
    let excluded: [bool; 5] = std::array::from_fn(|bag| {
        bag_slot_flag_is_set(state, bag as i32, BAG_SLOT_FLAG_EXCLUDE_JUNK_SELL)
    });
    let count = {
        let sim = borrow_state(state)?;
        sim.bag_items
            .iter()
            .map(|(&(bag, slot), stack)| {
                let bag_excluded = (0..=4).contains(&bag) && excluded[bag as usize];
                junk_stack_quantity(
                    (bag, slot),
                    stack.stack_count,
                    items::get_item(stack.item_id),
                    bag_excluded,
                )
            })
            .sum::<u64>()
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}
