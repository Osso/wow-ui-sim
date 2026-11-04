//! Bag-child item selection over the existing ItemChildEquipment snapshot.
//!
//! Retail/PTR/Forever TooltipInfoDocumentation.lua:154 declares the getter.
//! Selection composes modeled bag state with the generated child-equipment rows;
//! it does not claim native item-variant or exceptional-argument parity.

use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static, val_to_i32};
use rilua::LuaResult;
use rilua::table_security::unwrap_secret;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetBagItemChild", get_bag_item_child)
}

fn get_bag_item_child(state: &mut LuaState) -> LuaResult<u32> {
    let bag = authenticated_index(state, 1)?;
    let slot = authenticated_index(state, 2)?;
    let equip_slot = authenticated_index(state, 3)?;
    let child_id = {
        let sim = borrow_state(state)?;
        sim.get_bag_item(bag, slot)
            .and_then(|(parent, _)| sim.item_children.get(&parent))
            .and_then(|rows| {
                rows.iter()
                    .find(|row| i64::from(row.slot_index) == i64::from(equip_slot) - 1)
            })
            .map(|row| row.child_item_id)
    };
    let item = child_id.and_then(|id| crate::items::get_item(id).map(|item| (id, item)));
    let Some((id, item)) = item else {
        // EMULATED: MayReturnNothing permits a miss, but native result arity is unknown.
        return Ok(0);
    };
    let tooltip = tooltip_for_item_context(state, id, Some(item));
    state.push(tooltip);
    Ok(1)
}

fn authenticated_index(state: &LuaState, position: i32) -> LuaResult<i32> {
    let value = unwrap_secret(state, stack_val(state, position))?;
    val_to_i32(value, position)
}
