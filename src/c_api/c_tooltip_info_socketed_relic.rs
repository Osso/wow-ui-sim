//! WM97 DQ4 + WM104: viewed-artifact socket selection for `GetSocketedRelic`.
//!
//! Retail and PTR `TooltipInfoDocumentation.lua:1012` declare this getter.
//! Native evidence is Retail 12.1.5 (70688); catalog and relic data are 70391.

use crate::c_api::item_descriptor::{self, ItemDescriptor};
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_socketed_relic_context;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const API_NAME: &str = "C_TooltipInfo.GetSocketedRelic";
/// WM97 DQ4: the internal socket bound; the derived record supplies the rest.
const MAX_INTERNAL_SLOT: i64 = 3;

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetSocketedRelic", get_socketed_relic)
}

fn get_socketed_relic(state: &mut LuaState) -> LuaResult<u32> {
    let slot = parse_slot_index(state)?;
    let Some(descriptor) = select_socket(state, slot)? else {
        return emulated_missing_tooltip();
    };
    let player_level = player_level(state)?;
    let item = descriptor
        .resolve(player_level)
        .map_err(|error| rilua::runtime_error(format!("{API_NAME}: {error}")))?;
    let Some(item) = item else {
        return emulated_missing_tooltip();
    };
    let relic = item_descriptor::relic_metadata(item.gem_properties, i32::from(item.item_level))
        .map_err(|error| rilua::runtime_error(format!("{API_NAME}: {error}")))?;
    let tooltip = match relic {
        Some(relic) => {
            tooltip_for_socketed_relic_context(state, descriptor.item_id, &item, &relic)?
        }
        None => crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context(
            state,
            descriptor.item_id,
            Some(&item),
        ),
    };
    state.push(tooltip);
    Ok(1)
}

/// WM63 G9: the shared index reader subtracts one before truncation and accepts
/// 0 through 4294967295 before that subtraction.
fn parse_slot_index(state: &LuaState) -> LuaResult<i64> {
    let value = unwrap_secret(state, stack_val(state, 1))
        .map_err(|error| rilua::runtime_error(format!("{API_NAME}: argument 1: {error}")))?;
    let Val::Num(number) = value else {
        return Err(rilua::runtime_error(format!(
            "{API_NAME}: argument 1 requires a number"
        )));
    };
    if !number.is_finite() || !(0.0..=4_294_967_295.0).contains(&number) {
        return Err(rilua::runtime_error(format!(
            "{API_NAME}: argument 1 requires a value from 0 through 4294967295"
        )));
    }
    Ok((number - 1.0).trunc() as i64)
}

/// WM97 DQ4: artifact predicate, socket-marker bound and derived socket count,
/// then the socket's stored descriptor. Empty or unresolvable sockets miss.
fn select_socket(state: &LuaState, slot: i64) -> LuaResult<Option<ItemDescriptor>> {
    let model = borrow_state(state)?;
    let Some(info) = model.viewed_artifact.info.as_ref() else {
        return Ok(None);
    };
    let Ok(artifact_item_id) = u32::try_from(info.item_id) else {
        return Ok(None);
    };
    let Some(artifact) = crate::items::get_item(artifact_item_id) else {
        return Ok(None);
    };
    if artifact.artifact_id == 0 {
        return Ok(None);
    }
    let markers = artifact
        .socket_types
        .iter()
        .filter(|socket_type| **socket_type != 0)
        .count() as u32;
    let socket_count = markers.wrapping_add(item_descriptor::instance_socket_extra(
        model.viewed_artifact.instance_flags,
        model.viewed_artifact.instance_enchantment_slot6,
    ));
    if !(0..MAX_INTERNAL_SLOT).contains(&slot)
        || slot as u32 >= markers
        || slot as u32 >= socket_count
    {
        return Ok(None);
    }
    let Some(relic_slot) = model.viewed_artifact.relic_slots.get(slot as usize) else {
        return Ok(None);
    };
    Ok(relic_slot.socket.clone())
}

fn player_level(state: &LuaState) -> LuaResult<i32> {
    let model = borrow_state(state)?;
    let player = &model.player;
    Ok(if player.effective_level != 0 {
        player.effective_level
    } else {
        player.level
    })
}

/// EMULATED: WM63 leaves the native failure arity UNKNOWN.
fn emulated_missing_tooltip() -> LuaResult<u32> {
    Ok(0)
}
