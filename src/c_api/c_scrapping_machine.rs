//! Closed scrapping machine with no pending items.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_ScrappingMachineUI")?;
    for name in [
        "CloseScrappingMachine",
        "RemoveAllScrapItems",
        "ValidateScrappingList",
    ] {
        table_set_rust_fn_static(state, namespace, name, closed_machine_action)?;
    }
    table_set_rust_fn_static(state, namespace, "HasScrappableItems", has_scrappable_items)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCurrentPendingScrapItemLocationByIndex",
        pending_item_location,
    )
}

fn closed_machine_action(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn has_scrappable_items(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn pending_item_location(state: &mut LuaState) -> LuaResult<u32> {
    f64::from_stack(state, 1)?;
    Ok(0)
}
