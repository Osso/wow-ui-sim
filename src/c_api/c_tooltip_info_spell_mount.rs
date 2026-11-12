//! Numeric-secret spell/mount selectors and independent NeverSecret options.
//! Public presentation and secret identity are explicit simulator output policy.

use super::c_spell::{read_public_spell_identifier_at, read_spell_identifier_value};
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_spell_identifier;
use crate::lua_api::methods::{table_get_static, table_set_static};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::{
    SecretPayloadKind, inspect_host_secret_number, is_secret_value, secret_payload_kind,
    wrap_host_secret_number,
};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

/// Publish only after the caller has installed the namespace in globals.
pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetMountBySpellID", get_mount_by_spell_id)?;
    table_set_rust_fn_static(state, namespace, "GetSpellByID", get_spell_by_id)?;
    Ok(())
}

fn get_mount_by_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    push_tooltip(state, "C_TooltipInfo.GetMountBySpellID", 2, true)
}

fn get_spell_by_id(state: &mut LuaState) -> LuaResult<u32> {
    push_tooltip(state, "C_TooltipInfo.GetSpellByID", 6, false)
}

fn push_tooltip(
    state: &mut LuaState,
    api_name: &str,
    last_optional: i32,
    is_mount: bool,
) -> LuaResult<u32> {
    reject_secret_options(state, api_name, last_optional)?;
    let secret_selector = is_secret_value(state, stack_val(state, 1));
    let spell_id = read_spell_selector(state, api_name)?;
    // R81 Q1: dontOverride controls effective selection; isLink reaches cast context.
    let dont_override = !is_mount && stack_val(state, 4).is_truthy();
    let is_link = !is_mount && stack_val(state, 6).is_truthy();
    let tooltip = tooltip_for_spell_identifier(state, spell_id, is_mount, dont_override, is_link);
    // Root the DTO before identity wrappers allocate.
    state.push(tooltip);
    if secret_selector {
        wrap_tooltip_identity(state, tooltip);
    }
    Ok(1)
}

fn read_spell_selector(state: &LuaState, api_name: &str) -> LuaResult<Option<u32>> {
    let value = stack_val(state, 1);
    let mainline = cfg!(any(feature = "profile-retail", feature = "client-ptr"));
    if mainline && secret_payload_kind(state, value) == Some(SecretPayloadKind::Number) {
        return inspect_host_secret_number(state, value, |number| {
            let id = number as u32;
            if f64::from(id) != number {
                return Err(rilua::runtime_error(format!(
                    "{api_name}: argument 1 must be a finite integral u32 number"
                )));
            }
            read_spell_identifier_value(state, Val::Num(number))
        });
    }
    read_public_spell_identifier_at(state, 1, api_name)
}

fn wrap_tooltip_identity(state: &mut LuaState, tooltip: Val) {
    for field in ["id", "spellID"] {
        if let Val::Num(id) = table_get_static(state, tooltip, field) {
            let identity = wrap_host_secret_number(state, id);
            table_set_static(state, tooltip, field, identity);
        }
    }
}

fn reject_secret_options(state: &LuaState, api_name: &str, last_optional: i32) -> LuaResult<()> {
    // All documented optionals stay NeverSecret before alias/model lookup.
    for index in 2..=last_optional {
        if is_secret_value(state, stack_val(state, index)) {
            return Err(rilua::runtime_error(format!(
                "{api_name}: argument {index} rejects secret values under NeverSecret policy"
            )));
        }
    }
    Ok(())
}
