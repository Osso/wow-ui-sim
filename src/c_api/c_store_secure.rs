//! Profile-specific group-list projection over the existing simulator Lua store table.
//! Shapes follow cached Store consumers; native declarations are unavailable.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{create_table, table_get_static};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_StoreSecure")?;
    table_set_rust_fn_static(state, namespace, "GetProductGroups", get_product_groups)
}

pub(crate) fn publish_simulator_members_to_secure(state: &mut LuaState) -> LuaResult<()> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return Ok(());
    }

    // INFERRED (R88g.3-q1 pending): secure Retail Store UI needs this
    // simulator-owned API surface.
    let ordinary = super::global_val(state, "C_StoreSecure");
    let Val::Table(ordinary) = ordinary else {
        return Err(rilua::runtime_error(
            "C_StoreSecure must be registered before secure publication",
        ));
    };
    let secure = super::ensure_secure_namespace(state, "C_StoreSecure")?;
    let members = state
        .gc
        .tables
        .get(ordinary)
        .ok_or_else(|| rilua::runtime_error("C_StoreSecure namespace is unavailable"))?
        .hash_entries();

    let strings = &state.gc.string_arena;
    let secure_table = state
        .gc
        .tables
        .get_mut(secure)
        .ok_or_else(|| rilua::runtime_error("secure C_StoreSecure namespace is unavailable"))?;
    for (key, value) in members {
        if matches!(key, Val::Str(_)) && matches!(value, Val::Function(_)) {
            secure_table.raw_set(key, value, strings)?;
        }
    }
    state.gc.barrier_back(secure);
    Ok(())
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

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(feature = "client-retail")]
    #[test]
    fn secure_store_ui_can_call_simulated_product_list() {
        if !crate::client_profile::is_active_retail_12_1_5() {
            return;
        }

        let env = WowLuaEnv::new().expect("Lua environment should initialize");
        env.exec_rilua_secure("SecureStoreProductListResult = C_StoreSecure.GetProductList()")
            .expect("secure Store UI can call the simulated product-list API");

        let result: bool = env
            .eval("return rawget(__secureenv, 'SecureStoreProductListResult') == true")
            .expect("secure product-list result should be readable");
        assert!(result);
    }
}
