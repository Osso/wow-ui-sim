//! Exact category-map queries; public guards never unwrap secrets or clear taint.

use super::input::read_public_integer;
use super::snapshot;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn category_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_public_integer(Val::from_stack(state, 1)?, "categoryID")?;
    let record = borrow_state(state)?
        .housing
        .catalog
        .categories
        .get(&id)
        .cloned();
    match record {
        Some(record) => {
            snapshot::push_category(state, id, &record);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub(super) fn subcategory_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_public_integer(Val::from_stack(state, 1)?, "subcategoryID")?;
    let record = borrow_state(state)?
        .housing
        .catalog
        .subcategories
        .get(&id)
        .cloned();
    match record {
        Some(record) => {
            snapshot::push_subcategory(state, id, &record);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}
