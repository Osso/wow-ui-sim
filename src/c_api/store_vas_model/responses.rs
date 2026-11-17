use super::{StoreVasGuildFollowInfoInput, StoreVasGuildMasterInfoInput};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::WowLuaEnv;
use rilua::Val;

pub fn receive_bnet_transfer_validation(
    env: &WowLuaEnv,
    sequence: u64,
    error_code: i32,
    bnet_guid: String,
    account_guids: Vec<(Vec<u8>, String)>,
) -> crate::Result<()> {
    if ACTIVE != ClientProfile::Retail || !is_current_request(env, sequence)? {
        return Ok(());
    }
    if error_code == 0 {
        replace_bnet_transfer_state(env, bnet_guid, account_guids)?;
    }
    let error = (error_code != 0).then_some(Val::Num(f64::from(error_code)));
    let payload = error.unwrap_or(Val::Nil);
    env.fire_event_with_args("VAS_TRANSFER_VALIDATION_UPDATE", &[payload])
}

pub fn receive_guild_master_info(
    env: &WowLuaEnv,
    realm_address: u32,
    records: Vec<StoreVasGuildMasterInfoInput>,
) -> crate::Result<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }
    update_guild_master_info(env, records)?;
    env.fire_event_with_args(
        "STORE_GUILD_MASTER_INFO_RECEIVED",
        &[Val::Num(f64::from(realm_address))],
    )
}

pub fn receive_guild_follow_info(
    env: &WowLuaEnv,
    info: StoreVasGuildFollowInfoInput,
) -> crate::Result<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }
    update_guild_follow_info(env, info.clone())?;
    let guid = nonempty_guid(env, &info.guid);
    let info_table = guild_follow_info_table(env, &info)?;
    env.fire_event_with_args("STORE_GUILD_FOLLOW_INFO_RECEIVED", &[guid, info_table])
}

fn is_current_request(env: &WowLuaEnv, sequence: u64) -> crate::Result<bool> {
    Ok(env.state().borrow().store_glue.request_sequence == sequence)
}

fn replace_bnet_transfer_state(
    env: &WowLuaEnv,
    bnet_guid: String,
    account_guids: Vec<(Vec<u8>, String)>,
) -> crate::Result<()> {
    let mut sim = env.state().borrow_mut();
    let vas = &mut sim.store_glue.store_secure.vas;
    vas.bnet_transfer_guid = Some(bnet_guid);
    vas.bnet_game_accounts = account_guids
        .iter()
        .map(|(account_name, _)| account_name.clone())
        .collect();
    vas.wow_account_guids = account_guids.into_iter().collect();
    Ok(())
}

fn update_guild_master_info(
    env: &WowLuaEnv,
    records: Vec<StoreVasGuildMasterInfoInput>,
) -> crate::Result<()> {
    let mut sim = env.state().borrow_mut();
    let infos = &mut sim.store_glue.store_secure.vas.guild_master_info;
    for record in records {
        replace_guild_master_record(infos, record);
    }
    Ok(())
}

fn replace_guild_master_record(
    records: &mut Vec<StoreVasGuildMasterInfoInput>,
    record: StoreVasGuildMasterInfoInput,
) {
    let existing = records
        .iter()
        .position(|row| same_guid(&row.guid, &record.guid));
    match existing {
        Some(index) => records[index] = record,
        None => records.push(record),
    }
}

fn update_guild_follow_info(
    env: &WowLuaEnv,
    info: StoreVasGuildFollowInfoInput,
) -> crate::Result<()> {
    let mut sim = env.state().borrow_mut();
    let records = &mut sim.store_glue.store_secure.vas.guild_follow_info;
    let existing = records
        .iter()
        .position(|row| same_guid(&row.guid, &info.guid));
    match existing {
        Some(index) => records[index] = info,
        None => records.push(info),
    }
    Ok(())
}

fn same_guid(left: &str, right: &str) -> bool {
    let left = crate::c_api::store_guid::parse(left.as_bytes());
    let right = crate::c_api::store_guid::parse(right.as_bytes());
    matches!((left, right), (crate::c_api::store_guid::Outcome::Known(a), crate::c_api::store_guid::Outcome::Known(b)) if a == b)
}

fn nonempty_guid(env: &WowLuaEnv, guid: &str) -> Val {
    if guid.is_empty() {
        Val::Nil
    } else {
        env.lua_string(guid)
    }
}

fn guild_follow_info_table(
    env: &WowLuaEnv,
    info: &StoreVasGuildFollowInfoInput,
) -> crate::Result<Val> {
    let transferred_realm = info
        .transferred_realm
        .as_deref()
        .map(quote_lua_string)
        .unwrap_or_else(|| "nil".into());
    let code = format!(
        "return {{ transferredRealm = {transferred_realm}, factionChanged = {} }}",
        info.faction_changed
    );
    env.eval::<Val>(&code)
}

fn quote_lua_string(value: &str) -> String {
    let mut quoted = String::from("\"");
    for character in value.chars() {
        append_lua_character(&mut quoted, character);
    }
    quoted.push('"');
    quoted
}

fn append_lua_character(quoted: &mut String, character: char) {
    match character {
        '"' => quoted.push_str("\\\""),
        '\\' => quoted.push_str("\\\\"),
        '\n' => quoted.push_str("\\n"),
        '\r' => quoted.push_str("\\r"),
        '\t' => quoted.push_str("\\t"),
        character if character.is_control() => {
            for byte in character.to_string().bytes() {
                append_lua_byte_escape(quoted, byte);
            }
        }
        character => quoted.push(character),
    }
}

fn append_lua_byte_escape(quoted: &mut String, byte: u8) {
    quoted.push('\\');
    quoted.push(char::from(b'0' + byte / 100));
    quoted.push(char::from(b'0' + byte / 10 % 10));
    quoted.push(char::from(b'0' + byte % 10));
}
