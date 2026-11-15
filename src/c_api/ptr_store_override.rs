//! PTR R75 rewards override and host-supplied catalog interaction state.

use crate::c_api::helpers::{bad_argument, ensure_namespace};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const VALIDATE_USAGE: &str = "Usage: local isValid = C_CatalogShop.ValidateOverrideShopInteraction([shoppingSessionUUIDStr])";

/// Neutral host inputs, not a simulation of the unknown native writers.
#[derive(Debug, Default)]
pub struct CatalogShopInteraction {
    pub mode: i32,
    /// Raw bytes preserve native byte-for-byte matching, including empty strings.
    pub session: Vec<u8>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let store = ensure_namespace(state, "C_StorePublic")?;
    table_set_rust_fn_static(
        state,
        store,
        "IsRewardsShopOverrideEnabled",
        is_rewards_shop_override_enabled,
    )?;
    let catalog = ensure_namespace(state, "C_CatalogShop")?;
    table_set_rust_fn_static(
        state,
        catalog,
        "ValidateOverrideShopInteraction",
        validate_override_shop_interaction,
    )
}

fn is_rewards_shop_override_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = {
        let sim = borrow_state(state)?;
        let mirror = |name| {
            sim.cvars.get_server_mirrored_bool(name).ok_or_else(|| {
                rilua::runtime_error(format!("missing-input: PTR store mirror {name}"))
            })
        };
        mirror("rewardsShopOverrideEnabled")? && !mirror("bpayStoreEnable")?
    };
    state.push(Val::Bool(enabled));
    Ok(1)
}

/// Resolve optional Lua string/number inputs without lossy UTF-8 conversion.
fn read_optional_session(state: &LuaState) -> LuaResult<Option<Vec<u8>>> {
    let argument_error = || bad_argument(1, "ValidateOverrideShopInteraction", VALIDATE_USAGE);
    match stack_val(state, 1) {
        Val::Nil => Ok(None),
        value @ Val::Num(_) => Ok(Some(value.to_string().into_bytes())),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .map(|value| Some(value.data().to_vec()))
            .ok_or_else(argument_error),
        _ => Err(argument_error()),
    }
}

fn validate_override_shop_interaction(state: &mut LuaState) -> LuaResult<u32> {
    let session = read_optional_session(state)?;
    let valid = {
        let sim = borrow_state(state)?;
        let interaction = &sim.catalog_shop_interaction;
        interaction.mode == 3 && session.as_deref() == Some(interaction.session.as_slice())
    };
    state.push(Val::Bool(valid));
    Ok(1)
}
