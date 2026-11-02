//! Supplied ItemChildEquipment rows selected by parent item.

use crate::lua_api::methods::{borrow_state, create_table, table_set_num};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemChildEquipment {
    /// Zero-based slot selector from the native row.
    pub slot_index: i32,
    /// ChildItemID from the supplied row.
    pub child_item_id: u32,
}

pub(crate) fn default_child_equipment() -> std::collections::HashMap<u32, Vec<ItemChildEquipment>> {
    let mut result = std::collections::HashMap::<u32, Vec<ItemChildEquipment>>::new();
    for &(parent, child_item_id, slot_index) in sim_data::item_children::ITEM_CHILDREN {
        result.entry(parent).or_default().push(ItemChildEquipment {
            slot_index,
            child_item_id,
        });
    }
    result
}

pub(super) fn get_item_child_info(state: &mut LuaState) -> LuaResult<u32> {
    let item = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let slot = rilua::table_security::unwrap_secret(state, stack_val(state, 2))?;
    let id = super::parse_item_id_key(state, item)
        .ok_or_else(|| runtime_error("C_Item.GetItemChildInfo: expected itemInfo"))?;
    let slot = match slot {
        Val::Nil => None,
        Val::Num(n)
            if n.is_finite()
                && n.fract() == 0.0
                && n >= i32::MIN as f64
                && n <= i32::MAX as f64 =>
        {
            Some(i64::from(n as i32) - 1)
        }
        _ => return Err(runtime_error("C_Item.GetItemChildInfo: expected slotID")),
    };
    let values: Vec<i64> = {
        let sim = borrow_state(state)?;
        sim.item_children
            .get(&id)
            .into_iter()
            .flatten()
            .filter(|row| slot.is_none_or(|selected| i64::from(row.slot_index) == selected))
            .flat_map(|row| [i64::from(row.child_item_id), i64::from(row.slot_index)])
            .collect()
    };
    let Val::Table(table) = create_table(state) else {
        unreachable!("create_table returns a table")
    };
    for (index, value) in values.into_iter().enumerate() {
        table_set_num(state, table, (index + 1) as f64, Val::Num(value as f64));
    }
    state.push(Val::Table(table));
    Ok(1)
}
