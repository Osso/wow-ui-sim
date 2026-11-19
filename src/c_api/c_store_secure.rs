//! Retail registers the profile-specific catalog and VAS models for `C_StoreSecure`.
//! Non-Retail retains the legacy group-list projection over simulator state;
//! native non-Retail behavior is UNKNOWN. R88y supplies publication flags only. Evidence: pr46-49-decomp-answers.md, sections R88y.

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

#[cfg(test)]
mod tests {
    #[cfg(feature = "client-retail")]
    use crate::lua_api::WowLuaEnv;
    #[cfg(feature = "client-retail")]
    use crate::lua_api::methods::table_get_static;
    #[cfg(feature = "client-retail")]
    use rilua::{LuaApiMut, Val};

    #[cfg(feature = "client-retail")]
    #[test]
    fn secure_store_ui_can_call_simulated_product_list() {
        if !crate::client_profile::is_active_retail_12_1_5() {
            return;
        }
        let env = WowLuaEnv::new().expect("Lua environment should initialize");
        assert_store_secure_members(&env);
        assert_secure_product_list_request(&env);
    }

    #[cfg(feature = "client-retail")]
    fn collect_store_secure_member_status(
        state: &mut rilua::vm::state::LuaState,
    ) -> (usize, Vec<String>) {
        let ordinary = crate::c_api::global_val(state, "C_StoreSecure");
        let secure_env = crate::lua_api::methods::registry_get(state, "__secureenv");
        let secure_namespace = table_get_static(state, secure_env, "C_StoreSecure");
        let statuses: Vec<_> = super::super::secure_publication_data::VERIFIED_MEMBERS
            .iter()
            .filter_map(|&(namespace, member, flag, _)| {
                (namespace == "C_StoreSecure" && flag % 4 == 3).then_some(member)
            })
            .map(|member| store_secure_member_status(state, ordinary, secure_namespace, member))
            .collect();
        let implemented = statuses.iter().map(|(count, _)| count).sum();
        let missing_members = statuses
            .into_iter()
            .filter_map(|(_, missing)| missing)
            .collect();
        (implemented, missing_members)
    }

    #[cfg(feature = "client-retail")]
    fn assert_store_secure_members(env: &WowLuaEnv) {
        let (implemented, missing_members) = {
            let mut lua = env.rilua_mut();
            collect_store_secure_member_status(lua.state_mut())
        };
        assert!(
            implemented > 1,
            "the verified Store UI inventory is not vacuous"
        );
        assert!(
            missing_members.is_empty(),
            "implemented flag-3 members are missing from secure C_StoreSecure: {missing_members:?}"
        );
    }

    #[cfg(feature = "client-retail")]
    fn assert_secure_product_list_request(env: &WowLuaEnv) {
        let request_count_before = store_product_list_request_count(env);
        env.exec_rilua_secure(
            "SecureStoreProductListResultCount = select('#', C_StoreSecure.GetProductList())",
        )
        .expect("secure Store UI can call the simulated product-list API");
        let returns_zero_values: bool = env
            .eval("return rawget(__secureenv, 'SecureStoreProductListResultCount') == 0")
            .expect("secure product-list result count should be readable");
        assert!(
            returns_zero_values,
            "product-list request returns no values"
        );
        assert_eq!(
            store_product_list_request_count(env),
            request_count_before.wrapping_add(1),
            "secure product-list request should update the host model"
        );
    }

    #[cfg(feature = "client-retail")]
    fn store_secure_member_status(
        state: &mut rilua::vm::state::LuaState,
        ordinary: Val,
        secure_namespace: Val,
        member: &'static str,
    ) -> (usize, Option<String>) {
        match table_get_static(state, ordinary, member) {
            Val::Function(_) => {
                let missing = (!matches!(
                    table_get_static(state, secure_namespace, member),
                    Val::Function(_)
                ))
                .then(|| member.to_string());
                (1, missing)
            }
            Val::Nil => (0, None),
            value => (
                0,
                Some(format!(
                    "{member} has ordinary value type {}",
                    value.type_name()
                )),
            ),
        }
    }

    #[cfg(feature = "client-retail")]
    fn store_product_list_request_count(env: &WowLuaEnv) -> u32 {
        env.state()
            .borrow()
            .store_glue
            .store_secure
            .product_list_request_count
    }
}
