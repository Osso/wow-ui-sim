//! `C_Discord` service probes backed by local simulator state.
//!
//! The simulator does not contact Discord. This surface models deterministic
//! local state so addons can exercise enablement, OAuth intent, guild-link
//! intent, guild settings, and seeded server/channel metadata without relying
//! on Lua inert defaults. Forever 1.60.1 DiscordDocumentation.lua declares the
//! same members; Forever publishes only those whose modeled shape matches.

use crate::c_api::helpers::ensure_namespace;
#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set_num,
};
#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
use rilua::Val;
use rilua::vm::state::LuaState;

pub(crate) fn register_c_discord_surface(state: &mut LuaState) -> LuaResult<()> {
    let discord = ensure_namespace(state, "C_Discord")?;
    register_patch_12_1_discord_surface(state, discord)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
type DiscordFn = fn(&mut LuaState) -> LuaResult<u32>;

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
const DISCORD_METHODS: &[(&str, DiscordFn)] = &[
    ("Authorize", authorize),
    ("GetDisplayNameType", get_display_name_type),
    ("GetNumDiscordServers", get_num_discord_servers),
    ("GetServerLinkableChannels", get_server_linkable_channels),
    ("GetServerName", get_server_name),
    ("GuildLink", guild_link),
    ("GuildUnlink", guild_unlink),
    ("IsEnabled", is_enabled),
    ("IsGuildChannelLinked", is_guild_channel_linked),
    ("IsGuildSettingSet", is_guild_setting_set),
    ("IsUserOAuthed", is_user_oauthed),
    ("RefreshAuth", refresh_auth),
    ("SetGuildSetting", set_guild_setting),
    ("UpdateDiscordServers", update_discord_servers),
    ("UpdateGuildLobby", update_guild_lobby),
];

/// Kept off Forever: its docs type the user IDs as DiscordMemberOpaqueID, and
/// these channel/link-status shapes differ from the documented returns.
#[cfg(feature = "retail-12-1-0")]
const RETAIL_ONLY_DISCORD_METHODS: &[(&str, DiscordFn)] = &[
    ("GetDiscordChannelName", get_discord_channel_name),
    ("GetDiscordUserID", get_discord_user_id),
    ("GetDiscordUserName", get_discord_user_name),
    ("GetGuildLinkStatus", get_guild_link_status),
    ("GetNumDiscordChannels", get_num_discord_channels),
];

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn register_patch_12_1_discord_surface(
    state: &mut LuaState,
    discord: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    for &(name, method) in DISCORD_METHODS {
        table_set_rust_fn_static(state, discord, name, method)?;
    }
    #[cfg(feature = "retail-12-1-0")]
    for &(name, method) in RETAIL_ONLY_DISCORD_METHODS {
        table_set_rust_fn_static(state, discord, name, method)?;
    }
    Ok(())
}

#[cfg(not(any(feature = "retail-12-1-0", feature = "client-wowforever")))]
fn register_patch_12_1_discord_surface(
    _state: &mut LuaState,
    _discord: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    Ok(())
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
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

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
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

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn get_num_discord_servers(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.discord.servers.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn get_server_linkable_channels(state: &mut LuaState) -> LuaResult<u32> {
    let channels = borrow_state(state)?.discord.linkable_channel_names();
    let table = string_array_table(state, &channels);
    state.push(table);
    Ok(1)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn get_server_name(state: &mut LuaState) -> LuaResult<u32> {
    let server_index = i32::from_stack(state, 1)?;
    let name = borrow_state(state)?
        .discord
        .server_name(server_index)
        .map(str::to_string);
    push_optional_string(state, name.as_deref());
    Ok(1)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn guild_link(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.discord.guild_linked = true;
    Ok(0)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn guild_unlink(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.discord.guild_linked = false;
    Ok(0)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.cvars.get_bool("discordClientEnabled");
    state.push(Val::Bool(enabled));
    Ok(1)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn is_guild_channel_linked(state: &mut LuaState) -> LuaResult<u32> {
    let linked = borrow_state(state)?.discord.guild_linked;
    state.push(Val::Bool(linked));
    Ok(1)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
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

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn is_user_oauthed(state: &mut LuaState) -> LuaResult<u32> {
    let authorized = borrow_state(state)?.discord.oauth_authorized;
    state.push(Val::Bool(authorized));
    Ok(1)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn refresh_auth(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.discord.auth_refresh_requested = true;
    Ok(0)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn set_guild_setting(state: &mut LuaState) -> LuaResult<u32> {
    let setting = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let enabled = bool::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .discord
        .guild_settings
        .insert(setting, enabled);
    Ok(0)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn update_discord_servers(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.discord.server_update_requested = true;
    Ok(0)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
fn update_guild_lobby(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .discord
        .guild_lobby_update_requested = true;
    Ok(0)
}

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
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

#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
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
