//! Local-player tabard DTOs from existing guild-logo colors and supplied numeric metadata.
//!
//! Filename-to-fileID/style conversion is not inferred. Until the host supplies
//! both numeric fields, the documented optional query result is unavailable.

use crate::c_api::{ensure_namespace, global_val};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_table, table_set_static,
};
use crate::lua_api::state_types::GuildLogo;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuildRequest {
    Demote(String),
    SetLeader(String),
    Disband,
}

#[derive(Debug, Default)]
pub struct GuildRequestInputs {
    pub action_disabled: bool,
    pub transport_available: bool,
    pub submitted: Vec<GuildRequest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuildTabardMetadata {
    pub emblem_file_id: u32,
    pub emblem_style: u32,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE == ClientProfile::Wrath {
        return Ok(());
    }
    let namespace = ensure_namespace(state, "C_GuildInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetGuildTabardInfo",
        get_guild_tabard_info,
    )
}

fn get_guild_tabard_info(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?;
    if unit
        .as_deref()
        .is_some_and(|unit| !matches!(unit, "player" | "self"))
    {
        state.push(Val::Nil);
        return Ok(1);
    }
    let logo = borrow_state(state)?.world.guild_logo.clone();
    let Some(metadata) = logo.tabard_metadata else {
        state.push(Val::Nil);
        return Ok(1);
    };
    push_tabard_info(state, &logo, metadata)?;
    Ok(1)
}

fn push_tabard_info(
    state: &mut LuaState,
    logo: &GuildLogo,
    metadata: GuildTabardMetadata,
) -> LuaResult<()> {
    let row = create_table(state);
    state.push(row); // Keep the DTO rooted across the ColorMixin calls.
    for (field, value) in [
        ("emblemFileID", metadata.emblem_file_id),
        ("emblemStyle", metadata.emblem_style),
    ] {
        table_set_static(state, row, field, Val::Num(f64::from(value)));
    }
    for (field, rgb) in [
        ("backgroundColor", logo.background),
        ("borderColor", logo.border),
        ("emblemColor", logo.emblem),
    ] {
        let factory = global_val(state, "CreateColor");
        let color = call_function_state(
            state,
            factory,
            &[
                Val::Num(rgb.0),
                Val::Num(rgb.1),
                Val::Num(rgb.2),
                Val::Num(1.0),
            ],
        )?;
        table_set_static(state, row, field, color);
    }
    Ok(())
}

pub(crate) fn admin_set_tabard_metadata(state: &mut LuaState) -> LuaResult<u32> {
    let metadata = read_tabard_metadata_args(state)?;
    borrow_state_mut(state)?.world.guild_logo.tabard_metadata = metadata;
    Ok(0)
}

fn read_tabard_metadata_args(state: &mut LuaState) -> LuaResult<Option<GuildTabardMetadata>> {
    let file_id = Option::<f64>::from_stack(state, 1)?;
    let style = Option::<f64>::from_stack(state, 2)?;
    match (file_id, style) {
        (None, None) => Ok(None),
        (Some(file_id), Some(style)) => Ok(Some(GuildTabardMetadata {
            emblem_file_id: parse_tabard_integer(file_id, "emblemFileID")?,
            emblem_style: parse_tabard_integer(style, "emblemStyle")?,
        })),
        _ => Err(runtime_error(
            "A_Admin.SetGuildTabardMetadata: supply emblemFileID and emblemStyle together",
        )),
    }
}

fn parse_tabard_integer(value: f64, field: &str) -> LuaResult<u32> {
    let max = f64::from(u32::MAX);
    if value.fract() == 0.0 && (0.0..=max).contains(&value) {
        return Ok(value as u32);
    }
    Err(runtime_error(format!(
        "A_Admin.SetGuildTabardMetadata: {field} must be a nonnegative u32 integer"
    )))
}
