//! `C_CatalogShop` virtual-currency product surface.

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_table};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register_c_catalog_shop_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_CatalogShop")?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetVCProductInfos",
        c_catalog_shop_get_vc_product_infos,
    )?;
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        table_set_rust_fn_static(state, table_ref, "IsShop2Enabled", is_shop2_enabled)?;
    }
    Ok(())
}

fn is_shop2_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?
        .cvars
        .get_server_mirrored_bool("shop2Enabled")
        .ok_or_else(|| {
            rilua::runtime_error(
                "missing-input: C_CatalogShop.IsShop2Enabled has no supplied shop2Enabled value",
            )
        })?;
    state.push(rilua::Val::Bool(enabled));
    Ok(1)
}

fn c_catalog_shop_get_vc_product_infos(state: &mut LuaState) -> LuaResult<u32> {
    let product_infos = create_table(state);
    state.push(product_infos);
    Ok(1)
}
