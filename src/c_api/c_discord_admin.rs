//! `A_Admin` inputs for the local `C_Discord` model: the server/channel list
//! and the user ID/name records. They do not fire Discord events.

use crate::lua_api::methods::{borrow_state_mut, table_get, val_to_string};
use crate::lua_api::state::{DiscordChannel, DiscordServer};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("SetDiscordServers", set_discord_servers)?
        .set_function("SetDiscordUser", set_discord_user)
}

fn array_entries(state: &LuaState, list: Val) -> LuaResult<Vec<Val>> {
    let Val::Table(list_ref) = list else {
        return Err(runtime_error("A_Admin.SetDiscordServers: expected a table"));
    };
    let table = state.gc.tables.get(list_ref);
    let entries = (1..)
        .map(|index| table.map_or(Val::Nil, |table| table.get_int(index)))
        .take_while(|entry| !matches!(entry, Val::Nil))
        .collect();
    Ok(entries)
}

fn read_name(state: &mut LuaState, entry: Val) -> LuaResult<String> {
    let name = table_get(state, entry, "name");
    match (name, val_to_string(state, name)) {
        (Val::Str(_), Some(name)) => Ok(name),
        _ => Err(runtime_error(
            "A_Admin.SetDiscordServers: each entry needs a string name",
        )),
    }
}

fn read_channel(state: &mut LuaState, entry: Val) -> LuaResult<DiscordChannel> {
    let name = read_name(state, entry)?;
    let linkable = !matches!(table_get(state, entry, "linkable"), Val::Bool(false));
    Ok(DiscordChannel { name, linkable })
}

fn read_server(state: &mut LuaState, entry: Val) -> LuaResult<DiscordServer> {
    let name = read_name(state, entry)?;
    let channel_list = table_get(state, entry, "channels");
    let channels = array_entries(state, channel_list)?
        .into_iter()
        .map(|channel| read_channel(state, channel))
        .collect::<LuaResult<_>>()?;
    Ok(DiscordServer { name, channels })
}

/// `A_Admin.SetDiscordServers({ { name = s, channels = { { name = c, linkable = b } } } })`.
/// Replacing the list clears a guild link that no longer names a supplied channel.
fn set_discord_servers(state: &mut LuaState) -> LuaResult<u32> {
    let servers = array_entries(state, stack_val(state, 1))?
        .into_iter()
        .map(|server| read_server(state, server))
        .collect::<LuaResult<Vec<_>>>()?;
    let mut sim = borrow_state_mut(state)?;
    let discord = &mut sim.discord;
    let still_linked = discord
        .guild_link
        .as_ref()
        .is_some_and(|(server, channel)| {
            servers
                .iter()
                .any(|s| &s.name == server && s.channels.iter().any(|c| &c.name == channel))
        });
    if !still_linked {
        discord.guild_link = None;
    }
    discord.servers = servers;
    Ok(0)
}

/// `A_Admin.SetDiscordUser(userID, userName)`: the local user ID returned by
/// `GetDiscordUserID` and the name `GetDiscordUserName(userID)` reads.
fn set_discord_user(state: &mut LuaState) -> LuaResult<u32> {
    let user_id = String::from_stack(state, 1)?;
    let name = String::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    let discord = &mut sim.discord;
    discord.user_names.insert(user_id.clone(), name);
    discord.user_id = Some(user_id);
    Ok(0)
}
