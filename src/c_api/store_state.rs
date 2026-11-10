//! Retail/PTR store and promotional-upgrade inputs.

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        return Ok(());
    }

    let store = ensure_namespace(state, "C_StorePublic")?;
    table_set_rust_fn_static(state, store, "IsEnabled", is_store_enabled)?;

    let catalog = ensure_namespace(state, "C_CatalogShop")?;
    table_set_rust_fn_static(state, catalog, "IsShop2Enabled", is_shop2_enabled)?;

    Ok(())
}

fn is_store_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.cvars.get_bool("bpayStoreEnable");
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn is_shop2_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.cvars.get_bool("shop2Enabled");
    state.push(Val::Bool(enabled));
    Ok(1)
}

#[cfg(test)]
mod tests {
    use crate::client_profile::{ACTIVE, ClientProfile};
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn store_mirror_cvars_have_the_pinned_client_defaults() {
        if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
            return;
        }

        let env = WowLuaEnv::new().expect("Lua environment should initialize");
        let state = env.state();
        let state = state.borrow();
        assert_eq!(
            state.cvars.get_default("bpayStoreEnable").as_deref(),
            Some("0")
        );
        assert_eq!(
            state.cvars.get_default("shop2Enabled").as_deref(),
            Some("1")
        );
        assert_eq!(
            state
                .cvars
                .get_default("seenAsiaCharacterUpgradePopup")
                .as_deref(),
            Some("0")
        );
    }
}
