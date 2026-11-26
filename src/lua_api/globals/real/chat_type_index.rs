//! Retail/Forever chat-type lookup.
//!
//! `ChatTypeInfoColors.lua` consumes this while assigning `ChatTypeInfo[name].id`.
//! INFERRED "R88g9 (native evidence pending)": the fixed-name order projects the
//! pinned Retail `Blizzard_ChatFrameBase/Shared/ChatTypeInfoConstants.lua`
//! declaration order. `SMART_WHISPER` aliases `WHISPER`; `GUILD_ITEM_LOOTED`
//! aliases `GUILD_ACHIEVEMENT`. Channel slots follow `CHANNEL1`..`CHANNEL20`.

use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{Lua, LuaApiMut, LuaResult, Val, runtime_error};

const CHAT_TYPE_NAMES: &[&str] = &[
    "SYSTEM",
    "SAY",
    "PARTY",
    "RAID",
    "GUILD",
    "OFFICER",
    "GUILD_DISCORD",
    "YELL",
    "WHISPER",
    "WHISPER_INFORM",
    "REPLY",
    "EMOTE",
    "TEXT_EMOTE",
    "MONSTER_SAY",
    "MONSTER_PARTY",
    "MONSTER_YELL",
    "MONSTER_WHISPER",
    "MONSTER_EMOTE",
    "CHANNEL",
    "CHANNEL_JOIN",
    "CHANNEL_LEAVE",
    "CHANNEL_LIST",
    "CHANNEL_NOTICE",
    "CHANNEL_NOTICE_USER",
    "TARGETICONS",
    "AFK",
    "DND",
    "IGNORED",
    "SKILL",
    "LOOT",
    "CURRENCY",
    "MONEY",
    "OPENING",
    "TRADESKILLS",
    "PET_INFO",
    "COMBAT_MISC_INFO",
    "COMBAT_XP_GAIN",
    "COMBAT_HONOR_GAIN",
    "COMBAT_FACTION_CHANGE",
    "BG_SYSTEM_NEUTRAL",
    "BG_SYSTEM_ALLIANCE",
    "BG_SYSTEM_HORDE",
    "RAID_LEADER",
    "RAID_WARNING",
    "RAID_BOSS_WHISPER",
    "RAID_BOSS_EMOTE",
    "QUEST_BOSS_EMOTE",
    "ENCOUNTER_EVENT",
    "FILTERED",
    "INSTANCE_CHAT",
    "INSTANCE_CHAT_LEADER",
    "RESTRICTED",
    "ACHIEVEMENT",
    "GUILD_ACHIEVEMENT",
    "PARTY_LEADER",
    "BN_WHISPER",
    "BN_WHISPER_INFORM",
    "BN_ALERT",
    "BN_BROADCAST",
    "BN_BROADCAST_INFORM",
    "BN_INLINE_TOAST_ALERT",
    "BN_INLINE_TOAST_BROADCAST",
    "BN_INLINE_TOAST_BROADCAST_INFORM",
    "BN_WHISPER_PLAYER_OFFLINE",
    "PET_BATTLE_COMBAT_LOG",
    "PET_BATTLE_INFO",
    "COMMUNITIES_CHANNEL",
    "VOICE_TEXT",
    "PING",
];

pub(crate) fn register_all(lua: &mut Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetChatTypeIndex", get_chat_type_index)
}

fn get_chat_type_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = match stack_val(state, 1) {
        Val::Num(_) => 0,
        Val::Str(interned) => {
            let name = state
                .gc
                .string_arena
                .get(interned)
                .map(|bytes| String::from_utf8_lossy(bytes.data()).to_ascii_uppercase())
                .unwrap_or_default();
            lookup_fixed_type(&name)
                .or_else(|| lookup_channel_type(&name))
                .unwrap_or(0)
        }
        value => {
            let got = if state.top <= state.base {
                "no value"
            } else {
                value.type_name()
            };
            return Err(runtime_error(format!(
                "bad argument #1 to 'GetChatTypeIndex' (string expected, got {got})"
            )));
        }
    };
    state.push(Val::Num(index as f64));
    Ok(1)
}

fn lookup_fixed_type(name: &str) -> Option<u32> {
    if name == "SMART_WHISPER" {
        return lookup_fixed_type("WHISPER");
    }
    if name == "GUILD_ITEM_LOOTED" {
        return lookup_fixed_type("GUILD_ACHIEVEMENT");
    }
    CHAT_TYPE_NAMES
        .iter()
        .position(|candidate| *candidate == name)
        .map(|index| chat_type_index(index))
}

fn lookup_channel_type(name: &str) -> Option<u32> {
    let number = name.strip_prefix("CHANNEL")?.parse::<u32>().ok()?;
    (1..=20).contains(&number).then(|| 51 + number)
}

fn chat_type_index(zero_based: usize) -> u32 {
    let index = zero_based + 1;
    if index > 51 {
        (index + 20) as u32
    } else {
        index as u32
    }
}
