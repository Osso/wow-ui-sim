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

// VERIFIED — R88g.3.q1, Retail 12.1.5.70688: every listed member uses flag 3.
// The cached Store UI inventory is not build-verified.
const RETAIL_STORE_UI_FLAG_3_MEMBERS: [&str; 43] = [
    "AckFailure",
    "ClearPreGeneratedExternalTransactionID",
    "GetBnetTransferInfo",
    "GetCharacterInfoByGUID",
    "GetCharactersForRealm",
    "GetConfirmationInfo",
    "GetCurrencyID",
    "GetCurrencyInfo",
    "GetEligibleRacesForVASService",
    "GetEntryInfo",
    "GetFailureInfo",
    "GetProductGroupInfo",
    "GetProductGroups",
    "GetProductInfo",
    "GetProductList",
    "GetProducts",
    "GetPurchaseList",
    "GetRealmList",
    "GetUnrevokedBoostInfo",
    "GetVASCompletionInfo",
    "GetVASErrors",
    "GetVASGuildMasterInfoForCharacterByGUID",
    "GetVASRealmList",
    "GetVasServiceType",
    "GetWoWAccountGUIDFromName",
    "HasDistributionList",
    "HasDynamicPriceData",
    "HasProductList",
    "HasPurchaseInProgress",
    "HasPurchaseList",
    "IsAvailable",
    "IsDynamicBundle",
    "IsRegionLocked",
    "OpenNydusLink",
    "PurchaseProduct",
    "PurchaseProductConfirm",
    "PurchaseVASProduct",
    "RequestAllDynamicPriceInfo",
    "RequestCharacterGuildFollowInfo",
    "RequestRealmGuildMasterInfo",
    "SetDisconnectOnLogout",
    "SetVASProductReady",
    "ValidateBnetTransfer",
];

pub(crate) fn publish_simulator_members_to_secure(state: &mut LuaState) -> LuaResult<()> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return Ok(());
    }

    let namespace = super::global_val(state, "C_StoreSecure");
    let Val::Table(_) = namespace else {
        return Err(rilua::runtime_error(
            "C_StoreSecure must be registered before secure publication",
        ));
    };
    for member in RETAIL_STORE_UI_FLAG_3_MEMBERS {
        let value = table_get_static(state, namespace, member);
        match value {
            Val::Function(_) => {
                super::register_namespace_value_with_flags(
                    state,
                    "C_StoreSecure",
                    member,
                    value,
                    3,
                )?;
            }
            Val::Nil => {}
            _ => {
                return Err(rilua::runtime_error(format!(
                    "C_StoreSecure.{member} must be a function"
                )));
            }
        }
    }
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
    use crate::lua_api::methods::table_get_static;
    use rilua::Val;

    #[cfg(feature = "client-retail")]
    #[test]
    fn secure_store_ui_can_call_simulated_product_list() {
        if !crate::client_profile::is_active_retail_12_1_5() {
            return;
        }

        let env = WowLuaEnv::new().expect("Lua environment should initialize");
        let (implemented, missing) = {
            let mut lua = env.rilua_mut();
            let state = lua.state_mut();
            let ordinary = crate::c_api::global_val(state, "C_StoreSecure");
            let secure = crate::lua_api::methods::registry_get(state, "__secureenv");
            let mut implemented = 0;
            let mut missing = Vec::new();
            for member in super::RETAIL_STORE_UI_FLAG_3_MEMBERS {
                match table_get_static(state, ordinary, member) {
                    Val::Function(_) => {
                        implemented += 1;
                        if !matches!(table_get_static(state, secure, member), Val::Function(_)) {
                            missing.push(member);
                        }
                    }
                    Val::Nil => {}
                    _ => missing.push(member),
                }
            }
            (implemented, missing)
        };
        assert!(
            implemented > 1,
            "the verified Store UI inventory is not vacuous"
        );
        assert!(
            missing.is_empty(),
            "implemented flag-3 members missing from secure C_StoreSecure: {missing:?}"
        );

        env.exec_rilua_secure("SecureStoreProductListResult = C_StoreSecure.GetProductList()")
            .expect("secure Store UI can call the simulated product-list API");

        let result: bool = env
            .eval("return rawget(__secureenv, 'SecureStoreProductListResult') == true")
            .expect("secure product-list result should be readable");
        assert!(result);
    }
}
