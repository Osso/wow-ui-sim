//! Forever outbound intent records. Acceptance is local simulator policy, not
//! network delivery or native throttling. See docs/specs/addon-messages.md.

use super::ensure_namespace;
use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state::MessageLogEntry;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::{is_secret_value, unwrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// Enum.SendAddonMessageResult values from pinned ChatConstantsDocumentation.
const SUCCESS: u32 = 0;
const INVALID_PREFIX: u32 = 1;
const INVALID_MESSAGE: u32 = 2;
const INVALID_CHAT_TYPE: u32 = 4;
const NOT_IN_GROUP: u32 = 5;
const TARGET_REQUIRED: u32 = 6;
const GENERAL_ERROR: u32 = 9;
const NOT_IN_GUILD: u32 = 10;
const TARGET_OFFLINE: u32 = 12;
const PREFIX_BYTE_LIMIT: usize = 16;
const CHAT_BYTE_LIMIT: usize = 255;
const BNET_BYTE_LIMIT: usize = 4078;
const RAID_ROSTER_MINIMUM: usize = 6;

pub(super) fn register_chat(state: &mut LuaState) -> LuaResult<()> {
    let table = ensure_namespace(state, "C_ChatInfo")?;
    table_set_rust_fn_static(state, table, "SendAddonMessage", send_addon_message)?;
    table_set_rust_fn_static(
        state,
        table,
        "SendAddonMessageLogged",
        send_addon_message_logged,
    )
}

pub(super) fn send_game_data(state: &mut LuaState) -> LuaResult<u32> {
    // The VM authenticates secret access without clearing caller taint.
    let account = unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(account) = account else {
        return Err(rilua::runtime_error("gameAccountID must be a number"));
    };
    let prefix = read_string(state, unwrap_secret(state, stack_val(state, 2))?, 2)?;
    let data = read_string(state, unwrap_secret(state, stack_val(state, 3))?, 3)?;
    let result = {
        let sim = borrow_state(state)?;
        validate_bnet(&sim, account, &prefix, &data)
    };
    if result == SUCCESS {
        append_record(
            state,
            "bnet_game_data",
            prefix,
            data,
            String::new(),
            account.to_string(),
        )?;
    }
    push_result(state, result)
}

fn send_addon_message(state: &mut LuaState) -> LuaResult<u32> {
    send_chat(state, "addon")
}

fn send_addon_message_logged(state: &mut LuaState) -> LuaResult<u32> {
    send_chat(state, "addon_logged")
}

fn send_chat(state: &mut LuaState, kind: &str) -> LuaResult<u32> {
    for index in 1..=4 {
        if is_secret_value(state, stack_val(state, index)) {
            return Err(rilua::runtime_error(
                "addon chat does not allow secret arguments",
            ));
        }
    }
    let prefix = read_string(state, stack_val(state, 1), 1)?;
    let message = read_string(state, stack_val(state, 2), 2)?;
    let channel = read_optional_string(state, 3)?.unwrap_or_else(|| "PARTY".into());
    let target = read_optional_string(state, 4)?.unwrap_or_default();
    let result = {
        let sim = borrow_state(state)?;
        validate_chat(&sim, &prefix, &message, &channel, &target)
    };
    if result == SUCCESS {
        append_record(state, kind, prefix, message, channel, target)?;
    }
    push_result(state, result)
}

fn read_string(state: &LuaState, value: Val, index: i32) -> LuaResult<String> {
    let Val::Str(reference) = value else {
        return Err(rilua::runtime_error(format!(
            "argument {index} must be a string"
        )));
    };
    let bytes = state
        .gc
        .string_arena
        .get(reference)
        .ok_or_else(|| {
            rilua::runtime_error(format!("string at argument {index} has been collected"))
        })?
        .data();
    // MessageLogEntry stores Rust strings, matching the existing bridge boundary.
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| rilua::runtime_error(format!("string at argument {index} is not valid UTF-8")))
}

fn read_optional_string(state: &LuaState, index: i32) -> LuaResult<Option<String>> {
    match stack_val(state, index) {
        Val::Nil => Ok(None),
        value => read_string(state, value, index).map(Some),
    }
}

fn validate_payload(prefix: &str, message: &str, limit: usize) -> u32 {
    if prefix.is_empty() || prefix.len() > PREFIX_BYTE_LIMIT {
        INVALID_PREFIX
    } else if message.len() > limit {
        INVALID_MESSAGE
    } else {
        SUCCESS
    }
}

fn validate_chat(sim: &SimState, prefix: &str, message: &str, channel: &str, target: &str) -> u32 {
    let result = validate_payload(prefix, message, CHAT_BYTE_LIMIT);
    if result != SUCCESS {
        return result;
    }
    // Routing/result mappings are inferred, using the existing group/guild model.
    match channel {
        "PARTY" if !sim.party_group_active => NOT_IN_GROUP,
        "RAID" if !sim.party_group_active || sim.party_members.len() < RAID_ROSTER_MINIMUM => {
            NOT_IN_GROUP
        }
        "GUILD" if sim.world.guild_name.is_none() => NOT_IN_GUILD,
        "WHISPER" if target.is_empty() => TARGET_REQUIRED,
        "PARTY" | "RAID" | "GUILD" | "WHISPER" => SUCCESS,
        "CHANNEL" | "INSTANCE_CHAT" => GENERAL_ERROR,
        _ => INVALID_CHAT_TYPE,
    }
}

fn validate_bnet(sim: &SimState, account: f64, prefix: &str, data: &str) -> u32 {
    let result = validate_payload(prefix, data, BNET_BYTE_LIMIT);
    if result != SUCCESS {
        return result;
    }
    if !account.is_finite() || account <= 0.0 || account.fract() != 0.0 || account > i32::MAX as f64
    {
        return GENERAL_ERROR;
    }
    let online = sim
        .bnet_friends
        .iter()
        .flat_map(|friend| &friend.game_accounts)
        .any(|game| game.game_account_id == account as i32 && game.is_online);
    if online { SUCCESS } else { TARGET_OFFLINE }
}

fn append_record(
    state: &mut LuaState,
    kind: &str,
    prefix: String,
    message: String,
    channel: String,
    target: String,
) -> LuaResult<()> {
    borrow_state_mut(state)?.message_log.push(MessageLogEntry {
        kind: kind.into(),
        prefix,
        message,
        channel,
        target,
    });
    Ok(())
}

fn push_result(state: &mut LuaState, result: u32) -> LuaResult<u32> {
    state.push(Val::Num(result as f64));
    Ok(1)
}
