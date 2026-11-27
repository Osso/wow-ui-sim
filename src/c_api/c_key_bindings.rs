//! Active binding contexts shared by cached HouseEditor mode consumers.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const LAST_BINDING_CONTEXT: f64 = 9.0;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_KeyBindings")?;
    for (name, function) in [
        ("ActivateBindingContext", activate as rilua::RustFn),
        ("DeactivateBindingContext", deactivate),
        ("IsBindingContextActive", is_active),
    ] {
        table_set_rust_fn_static(state, ns, name, function)?;
    }
    register_binding_context_enum(state)
}

fn register_binding_context_enum(state: &mut LuaState) -> LuaResult<()> {
    let final_context = match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail => "ReservedFutureFeatureBinding01",
        crate::client_profile::ClientProfile::WowForever => "GamepadModeInGameCore",
        _ => return Ok(()),
    };
    let enums = ensure_namespace(state, "Enum")?;
    let contexts = crate::c_api::ensure_namespace_table_field(state, enums, "BindingContext")?;
    for (name, value) in [
        ("None", 0.0),
        ("HousingEditor", 1.0),
        ("HousingEditorBasicDecorMode", 2.0),
        ("HousingEditorExpertDecorMode", 3.0),
        ("HousingEditorCustomizeMode", 4.0),
        ("HousingEditorCleanupMode", 5.0),
        ("HousingEditorLayoutMode", 6.0),
        ("HousingEditorBasicAndExpertDecorMode", 7.0),
        ("HousingEditorExteriorCustomizationMode", 8.0),
        (final_context, 9.0),
    ] {
        crate::c_api::set_namespace_member(state, Val::Table(contexts), name, Val::Num(value))?;
    }
    Ok(())
}

fn read_context(state: &LuaState) -> LuaResult<i32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    match value {
        Val::Num(value)
            if (0.0..=LAST_BINDING_CONTEXT).contains(&value) && value.fract() == 0.0 =>
        {
            Ok(value as i32)
        }
        _ => Err(rilua::runtime_error(
            "binding context must be an integer in 0..9",
        )),
    }
}

fn activate(state: &mut LuaState) -> LuaResult<u32> {
    let context = read_context(state)?;
    // Cached Basic/Expert modes activate both a specific and a shared context.
    // INFERRED: repeated activation is idempotent; this is not a routing-priority model.
    borrow_state_mut(state)?
        .keybindings
        .active_contexts
        .insert(context);
    Ok(0)
}

fn deactivate(state: &mut LuaState) -> LuaResult<u32> {
    let context = read_context(state)?;
    borrow_state_mut(state)?
        .keybindings
        .active_contexts
        .remove(&context);
    Ok(0)
}

fn is_active(state: &mut LuaState) -> LuaResult<u32> {
    let context = read_context(state)?;
    let active = borrow_state(state)?
        .keybindings
        .active_contexts
        .contains(&context);
    state.push(Val::Bool(active));
    Ok(1)
}
