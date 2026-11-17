//! Legacy non-Retail group-list projection; native behavior remains UNKNOWN there.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{create_table, table_get_static};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE == ClientProfile::Retail {
        super::c_store_secure_catalog::register(state)?;
        return super::c_store_secure_vas::register(state);
    }
    let namespace = super::ensure_namespace(state, "C_StoreSecure")?;
    table_set_rust_fn_static(state, namespace, "GetProductGroups", get_product_groups)
}

fn get_product_groups(state: &mut LuaState) -> LuaResult<u32> {
    let store = super::global_val(state, "__wow_store_state");
    let groups = table_get_static(state, store, "productGroups");
    let Val::Table(groups_ref) = groups else {
        return Err(rilua::runtime_error(
            "Store product group data is not initialized",
        ));
    };
    if ACTIVE != ClientProfile::Mists {
        state.push(groups);
        return Ok(1);
    }

    let ids = read_group_ids(state, groups_ref)?;
    let result = create_table(state);
    state.push(result);
    for (index, id) in ids.into_iter().enumerate() {
        super::helpers::set_table_array(state, result, index as i64 + 1, id);
    }
    Ok(1)
}

fn read_group_ids(state: &mut LuaState, groups: GcRef<Table>) -> LuaResult<Vec<Val>> {
    let rows: Vec<_> = {
        let table = state
            .gc
            .tables
            .get(groups)
            .ok_or_else(|| rilua::runtime_error("Store product group table is unavailable"))?;
        table
            .array_slice()
            .iter()
            .copied()
            .take_while(|row| *row != Val::Nil)
            .collect()
    };
    rows.into_iter()
        .map(|row| {
            let id = table_get_static(state, row, "groupID");
            if matches!(id, Val::Num(_)) {
                Ok(id)
            } else {
                Err(rilua::runtime_error(
                    "Store product group record requires a numeric groupID",
                ))
            }
        })
        .collect()
}
