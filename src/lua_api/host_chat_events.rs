//! Dispatch explicit host chat snapshots through normal public event listeners.

use super::WowLuaEnv;
use super::host_chat_inputs::{DiscordChatInfo, HostChatArgument, HostChatMessage};
use crate::event::EventArg;
use crate::lua_api::methods::{create_string, create_table, table_set};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, Val};

impl WowLuaEnv {
    /// INFERRED: publish one queued message, consuming it once before listeners.
    /// Read lockdown at delivery, not when the host queues the message.
    pub fn publish_next_host_chat(&self) -> crate::Result<bool> {
        let message = self.state.borrow_mut().host_chat_inputs.pending.pop_front();
        let Some(message) = message else {
            return Ok(false);
        };
        self.publish_host_chat_message(&message)?;
        Ok(true)
    }

    fn publish_host_chat_message(&self, message: &HostChatMessage) -> crate::Result<()> {
        validate_host_chat_classification(message)?;
        let lockdown = self.state.borrow().chat_messaging_lockdown;
        let restricted = lockdown && message.kind.secret_during_lockdown();
        let (saved_top, arguments) = {
            let mut lua = self.lua.borrow_mut();
            let state = lua.state_mut();
            let saved_top = state.top;
            let mut arguments = message
                .arguments
                .iter()
                .enumerate()
                .map(|(index, argument)| {
                    let lockdown_secret = restricted && !never_secret_argument(index);
                    push_host_chat_argument(state, argument, lockdown_secret)
                })
                .collect::<Vec<_>>();
            append_discord_info(state, &mut arguments, &message.discord_info);
            (saved_top, arguments)
        };
        let result = self.fire_event_with_args(message.kind.event_name(), &arguments);
        self.lua.borrow_mut().state_mut().top = saved_top;
        result
    }
}

// Zero-based positions of NeverSecret payloads in cached ChatInfoDocumentation.
fn never_secret_argument(index: usize) -> bool {
    matches!(index, 2 | 3 | 5 | 6 | 7 | 8 | 9 | 10 | 13 | 14 | 15 | 16)
}

/// `ChatMessageEventParams` positions before `discordInfo` (12.1.0 payload 18).
const DISCORD_INFO_POSITION: usize = 17;

/// Missing leading payloads are nil so Blizzard's `arg18` read finds discordInfo.
#[cfg(feature = "retail-12-1-0")]
fn append_discord_info(state: &mut LuaState, arguments: &mut Vec<Val>, info: &DiscordChatInfo) {
    arguments.resize(DISCORD_INFO_POSITION, Val::Nil);
    // INFERRED: the struct is never secret-wrapped, including during lockdown.
    let table = create_table(state);
    state.push(table);
    for (name, text) in [
        ("globalName", &info.global_name),
        ("lastOnlineGUID", &info.last_online_guid),
        ("lastOnlineName", &info.last_online_name),
        ("forwardedMessage", &info.forwarded_message),
    ] {
        let value = create_string(state, text);
        table_set(state, table, name, value);
    }
    table_set(state, table, "userID", Val::Num(info.user_id));
    table_set(
        state,
        table,
        "type",
        Val::Num(f64::from(info.display_name_type)),
    );
    for (name, flag) in [
        ("hasAttachment", info.has_attachment),
        ("hasPoll", info.has_poll),
        ("hasEmbed", info.has_embed),
        ("hasSticker", info.has_sticker),
        ("hasEmoji", info.has_emoji),
        ("hasError", info.has_error),
        ("hasForwardedMessage", info.has_forwarded_message),
        ("fromDiscord", info.from_discord),
    ] {
        table_set(state, table, name, Val::Bool(flag));
    }
    arguments.push(table);
}

#[cfg(not(feature = "retail-12-1-0"))]
fn append_discord_info(_: &mut LuaState, _: &mut Vec<Val>, _: &DiscordChatInfo) {}

fn validate_host_chat_classification(message: &HostChatMessage) -> crate::Result<()> {
    if message.arguments.len() > DISCORD_INFO_POSITION {
        return Err(
            rilua::runtime_error("host chat payload exceeds ChatMessageEventParams").into(),
        );
    }
    for (index, argument) in message.arguments.iter().enumerate() {
        if argument.secret && never_secret_argument(index) {
            return Err(
                rilua::runtime_error("host chat NeverSecret payload cannot be secret").into(),
            );
        }
    }
    Ok(())
}

fn push_host_chat_argument(
    state: &mut LuaState,
    argument: &HostChatArgument,
    lockdown_secret: bool,
) -> Val {
    let secret = argument.secret || lockdown_secret;
    let value = match &argument.value {
        EventArg::String(text) if secret => {
            rilua::table_security::wrap_host_secret_string(state, text)
        }
        EventArg::Number(number) if secret => {
            rilua::table_security::wrap_host_secret_number(state, *number)
        }
        EventArg::Boolean(value) if secret => {
            rilua::table_security::wrap_host_secret_bool(state, *value)
        }
        EventArg::String(text) => create_string(state, text),
        EventArg::Number(number) => Val::Num(*number),
        EventArg::Boolean(value) => Val::Bool(*value),
        EventArg::Nil => Val::Nil,
    };
    // Root every allocation through all listeners, nested dispatch and GC.
    state.push(value);
    value
}
