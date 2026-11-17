//! Retail native mixin globals.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::security::register_native_function_with_flags;
use crate::lua_api::methods::create_table;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

mod secure_registry;
use secure_registry::resolve_mixin_methods;

/// Earlier Retail builds keep their existing flag route; R88y supplies the current flag.
const LEGACY_RETAIL_MIXIN_FLAGS: u8 = 7;

/// Register the verified Retail native entries. Bootstrap fallbacks stay ordinary-only.
pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }
    if crate::client_profile::is_active_retail_12_1_5() {
        crate::c_api::register_global_function_from_r88y(lua, "Mixin", mixin)?;
        crate::c_api::register_global_function_from_r88y(
            lua,
            "CreateFromMixins",
            create_from_mixins,
        )?;
    } else {
        register_native_function_with_flags(lua, "Mixin", mixin, LEGACY_RETAIL_MIXIN_FLAGS)?;
        register_native_function_with_flags(
            lua,
            "CreateFromMixins",
            create_from_mixins,
            LEGACY_RETAIL_MIXIN_FLAGS,
        )?;
    }
    Ok(())
}

fn mixin(state: &mut LuaState) -> LuaResult<u32> {
    let argument_count = state.top.saturating_sub(state.base);
    let object = stack_val(state, 1);
    apply_mixins(state, object, 2, argument_count)?;
    state.push(object);
    Ok(1)
}

fn create_from_mixins(state: &mut LuaState) -> LuaResult<u32> {
    let argument_count = state.top.saturating_sub(state.base);
    let object = create_table(state);
    state.push(object);
    apply_mixins(state, object, 1, argument_count)?;
    Ok(1)
}

fn apply_mixins(
    state: &mut LuaState,
    object: Val,
    first_mixin_index: usize,
    argument_count: usize,
) -> LuaResult<()> {
    for index in first_mixin_index..=argument_count {
        let mixin = stack_val(state, index as i32);
        apply_mixin(state, object, mixin)?;
    }
    Ok(())
}

fn apply_mixin(state: &mut LuaState, object: Val, mixin: Val) -> LuaResult<()> {
    let Val::Table(methods_ref) = resolve_mixin_methods(state, mixin) else {
        return Ok(());
    };
    let Some(methods) = state.gc.tables.get(methods_ref) else {
        return Ok(());
    };

    for (key, value) in collect_mixin_entries(methods) {
        state.settable(object, key, value)?;
    }
    Ok(())
}

fn collect_mixin_entries(methods: &Table) -> Vec<(Val, Val)> {
    let array_entries = methods
        .array_slice()
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(index, value)| {
            (!matches!(value, Val::Nil)).then_some((Val::Num((index + 1) as f64), value))
        });
    let hash_entries = methods
        .hash_entries()
        .into_iter()
        .filter(|(_, value)| !matches!(value, Val::Nil));
    array_entries.chain(hash_entries).collect()
}
