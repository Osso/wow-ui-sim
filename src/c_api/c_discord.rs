//! `C_Discord` service probes backed by local simulator state.
//!
//! The simulator does not contact Discord. This surface models deterministic
//! local state so addons can exercise enablement, OAuth intent, guild-link
//! intent, guild settings, and seeded server/channel metadata without relying
//! on Lua inert defaults.

use crate::c_api::helpers::ensure_namespace;
#[cfg(feature = "retail-12-1-0")]
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set_num,
};
#[cfg(feature = "retail-12-1-0")]
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
#[cfg(feature = "retail-12-1-0")]
use rilua::Val;
use rilua::vm::state::LuaState;

pub(crate) fn register_c_discord_surface(state: &mut LuaState) -> LuaResult<()> {
    let discord = ensure_namespace(state, "C_Discord")?;
    register_patch_12_1_discord_surface(state, discord)
}

#[cfg(feature = "retail-12-1-0")]
fn register_patch_12_1_discord_surface(
    state: &mut LuaState,
    discord: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, discord, "Authorize", authorize)?;
    table_set_rust_fn_static(
        state,
        discord,
        "GetDiscordChannelName",
        get_discord_channel_name,
    )?;
    table_set_rust_fn_static(state, discord, "GetDiscordUserID", get_discord_user_id)?;
    table_set_rust_fn_static(state, discord, "GetDiscordUserName", get_discord_user_name)?;
    table_set_rust_fn_static(state, discord, "GetDisplayNameType", get_display_name_type)?;
    table_set_rust_fn_static(state, discord, "GetGuildLinkStatus", get_guild_link_status)?;
    table_set_rust_fn_static(
        state,
        discord,
        "GetNumDiscordChannels",
        get_num_discord_channels,
    )?;
    table_set_rust_fn_static(
        state,
        discord,
        "GetNumDiscordServers",
        get_num_discord_servers,
    )?;
    table_set_rust_fn_static(
        state,
        discord,
        "GetServerLinkableChannels",
        get_server_linkable_channels,
    )?;
    table_set_rust_fn_static(state, discord, "GetServerName", get_server_name)?;
    table_set_rust_fn_static(state, discord, "GuildLink", guild_link)?;
    table_set_rust_fn_static(state, discord, "GuildUnlink", guild_unlink)?;
    table_set_rust_fn_static(state, discord, "IsEnabled", is_enabled)?;
    table_set_rust_fn_static(
        state,
        discord,
        "IsGuildChannelLinked",
        is_guild_channel_linked,
    )?;
    table_set_rust_fn_static(state, discord, "IsGuildSettingSet", is_guild_setting_set)?;
    table_set_rust_fn_static(state, discord, "IsUserOAuthed", is_user_oauthed)?;
    table_set_rust_fn_static(state, discord, "RefreshAuth", refresh_auth)?;
    table_set_rust_fn_static(state, discord, "SetGuildSetting", set_guild_setting)?;
    table_set_rust_fn_static(
        state,
        discord,
        "UpdateDiscordServers",
        update_discord_servers,
    )?;
    table_set_rust_fn_static(state, discord, "UpdateGuildLobby", update_guild_lobby)
}

#[cfg(not(feature = "retail-12-1-0"))]
fn register_patch_12_1_discord_surface(
    _state: &mut LuaState,
    _discord: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
fn authorize(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.discord.oauth_authorized = true;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn get_discord_channel_name(state: &mut LuaState) -> LuaResult<u32> {
    let channel_index = i32::from_stack(state, 1)?;
    let name = borrow_state(state)?
        .discord
        .channel_name(channel_index)
        .map(str::to_string);
    push_optional_string(state, name.as_deref());
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_discord_user_id(state: &mut LuaState) -> LuaResult<u32> {
    let user_id = borrow_state(state)?.discord.user_id.clone();
    push_optional_string(state, user_id.as_deref());
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_discord_user_name(state: &mut LuaState) -> LuaResult<u32> {
    // ItemRefHandlers passes `tonumber(discordUserID)`; key both forms by decimal text.
    let user_id = match crate::lua_bridge::stack_val(state, 1) {
        Val::Num(id) => format!("{id:.0}"),
        _ => Option::<String>::from_stack(state, 1)?.unwrap_or_default(),
    };
    // INFERRED: unknown users yield an empty name; the return is non-nilable.
    let name = borrow_state(state)?
        .discord
        .user_names
        .get(&user_id)
        .cloned()
        .unwrap_or_default();
    push_optional_string(state, Some(&name));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_display_name_type(state: &mut LuaState) -> LuaResult<u32> {
    let display_name_type = borrow_state(state)?.discord.display_name_type;
    state.push(Val::Num(f64::from(display_name_type)));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_guild_link_status(state: &mut LuaState) -> LuaResult<u32> {
    let status = borrow_state(state)?.discord.guild_link_status;
    push_optional_i32(state, status);
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_num_discord_channels(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.discord.channels.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_num_discord_servers(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.discord.servers.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_server_linkable_channels(state: &mut LuaState) -> LuaResult<u32> {
    let channels = borrow_state(state)?.discord.linkable_channel_names();
    let table = string_array_table(state, &channels);
    state.push(table);
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_server_name(state: &mut LuaState) -> LuaResult<u32> {
    let server_index = i32::from_stack(state, 1)?;
    let name = borrow_state(state)?
        .discord
        .server_name(server_index)
        .map(str::to_string);
    push_optional_string(state, name.as_deref());
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn guild_link(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.discord.guild_linked = true;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn guild_unlink(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.discord.guild_linked = false;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.cvars.get_bool("discordClientEnabled");
    state.push(Val::Bool(enabled));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn is_guild_channel_linked(state: &mut LuaState) -> LuaResult<u32> {
    let linked = borrow_state(state)?.discord.guild_linked;
    state.push(Val::Bool(linked));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn is_guild_setting_set(state: &mut LuaState) -> LuaResult<u32> {
    let setting = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let is_set = borrow_state(state)?
        .discord
        .guild_settings
        .get(&setting)
        .copied()
        .unwrap_or(false);
    state.push(Val::Bool(is_set));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn is_user_oauthed(state: &mut LuaState) -> LuaResult<u32> {
    let authorized = borrow_state(state)?.discord.oauth_authorized;
    state.push(Val::Bool(authorized));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn refresh_auth(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.discord.auth_refresh_requested = true;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn set_guild_setting(state: &mut LuaState) -> LuaResult<u32> {
    let setting = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let enabled = bool::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .discord
        .guild_settings
        .insert(setting, enabled);
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn update_discord_servers(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.discord.server_update_requested = true;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn update_guild_lobby(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .discord
        .guild_lobby_update_requested = true;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn push_optional_string(state: &mut LuaState, value: Option<&str>) {
    match value {
        Some(value) => {
            let value = create_string(state, value);
            state.push(value);
        }
        None => state.push(Val::Nil),
    }
}

#[cfg(feature = "retail-12-1-0")]
fn push_optional_i32(state: &mut LuaState, value: Option<i32>) {
    match value {
        Some(value) => state.push(Val::Num(f64::from(value))),
        None => state.push(Val::Nil),
    }
}

#[cfg(feature = "retail-12-1-0")]
fn string_array_table(state: &mut LuaState, values: &[String]) -> Val {
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    for (index, value) in values.iter().enumerate() {
        let value = create_string(state, value);
        table_set_num(state, table_ref, (index + 1) as f64, value);
    }
    Val::Table(table_ref)
}
