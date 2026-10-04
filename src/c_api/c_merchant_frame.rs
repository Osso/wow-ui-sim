//! `C_MerchantFrame` 12.0.7 probe surface.
//!
//! Retail 12.0.7 currency IDs read the explicit ordered host snapshot.

#[cfg(feature = "client-wowforever")]
pub mod junk;

use crate::c_api::helpers::ensure_namespace;
#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
use crate::lua_api::methods::create_table;
#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register_c_merchant_frame_surface(state: &mut LuaState) -> LuaResult<()> {
    let merchant = ensure_namespace(state, "C_MerchantFrame")?;
    register_patch_12_0_7_merchant_frame_surface(state, merchant)?;
    #[cfg(feature = "client-wowforever")]
    crate::lua_bridge::table_set_rust_fn_static(
        state,
        merchant,
        "GetNumJunkItems",
        junk::get_num_junk_items,
    )?;
    Ok(())
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn register_patch_12_0_7_merchant_frame_surface(
    state: &mut LuaState,
    merchant: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        merchant,
        "GetMerchantCurrencies",
        get_merchant_currencies,
    )
}

#[cfg(not(any(feature = "retail-12-0-7", feature = "retail-12-1-0")))]
fn register_patch_12_0_7_merchant_frame_surface(
    _state: &mut LuaState,
    _merchant: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    Ok(())
}

#[cfg(feature = "retail-12-0-7")]
fn get_merchant_currencies(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED: no SecretArguments declaration; reject secret extras for every caller.
    for value in &state.stack[state.base..state.top] {
        if rilua::table_security::is_secret_value(state, *value) {
            return Err(rilua::runtime_error(
                "GetMerchantCurrencies rejects secret arguments",
            ));
        }
    }
    // INFERRED: preserve host order; absent merchant input returns one empty table.
    let ids = crate::lua_api::methods::borrow_state(state)?
        .merchant_currencies
        .clone();
    let currencies = create_table(state);
    state.push(currencies);
    for (index, id) in ids.into_iter().enumerate() {
        crate::c_api::helpers::set_table_array(
            state,
            currencies,
            index as i64 + 1,
            rilua::Val::Num(f64::from(id)),
        );
    }
    Ok(1)
}

#[cfg(all(feature = "retail-12-1-0", not(feature = "retail-12-0-7")))]
fn get_merchant_currencies(state: &mut LuaState) -> LuaResult<u32> {
    let currencies = create_table(state);
    state.push(currencies);
    Ok(1)
}
