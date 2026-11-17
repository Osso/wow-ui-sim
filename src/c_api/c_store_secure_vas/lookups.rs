use super::INVALID_ARGUMENT;
use crate::c_api::c_store_glue;
use crate::c_api::helpers;
use crate::c_api::store_guid::{Outcome, parse};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set_static as set_static,
};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::cmp::Ordering;

const GET_CHARACTERS: &str = "GetCharactersForRealm";
const GET_CHARACTER: &str = "GetCharacterInfoByGUID";
const GET_ACCOUNT_GUID: &str = "GetWoWAccountGUIDFromName";
const GET_ELIGIBLE_RACES: &str = "GetEligibleRacesForVASService";
const GET_GUILD_INFO: &str = "GetVASGuildMasterInfoForCharacterByGUID";
const GET_SERVICE_TYPE: &str = "GetVasServiceType";

pub(super) fn get_realm_list(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let mut rows = borrow_state(state)?
        .store_glue
        .store_secure
        .vas
        .realms
        .clone();
    rows.sort_by(|left, right| right.character_count.cmp(&left.character_count));
    super::pending_error(state, "GetRealmList")?;
    push_realms(state, &rows, false);
    Ok(1)
}

pub(super) fn get_vas_realm_list(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let rows = {
        let mut sim = borrow_state_mut(state)?;
        let realms = &mut sim.store_glue.store_secure.vas.vas_realms;
        realms.sort_by(|left, right| left.realm_name.as_bytes().cmp(right.realm_name.as_bytes()));
        realms.clone()
    };
    super::pending_error(state, "GetVASRealmList")?;
    push_realms(state, &rows, true);
    Ok(1)
}

pub(super) fn get_characters_for_realm(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let realm_address = read_realm_address(state, 1, GET_CHARACTERS)?;
    let guild_only = read_truthy_argument(state, 2, GET_CHARACTERS)?;
    let rows = characters_for_realm(state, realm_address, guild_only)?;
    super::pending_error(state, GET_CHARACTERS)?;
    push_characters(state, &rows);
    Ok(1)
}

pub(super) fn get_character_info_by_guid(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let guid = read_guid(state, 1, GET_CHARACTER)?;
    let character = find_character_object(state, &guid)?;
    super::pending_error(state, GET_CHARACTER)?;
    push_optional_character(state, character.as_ref());
    Ok(1)
}

pub(super) fn get_wow_account_guid_from_name(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let name = read_text_bytes(state, 1, GET_ACCOUNT_GUID)?;
    let local = read_truthy_argument(state, 2, GET_ACCOUNT_GUID)?;
    let guid = account_guid(state, &name, local)?;
    super::pending_error(state, GET_ACCOUNT_GUID)?;
    push_guid(state, guid.as_deref());
    Ok(1)
}

pub(super) fn get_eligible_races_for_vas_service(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let guid = read_guid(state, 1, GET_ELIGIBLE_RACES)?;
    let service = read_service_type(state, 2, GET_ELIGIBLE_RACES)?;
    let result = race_eligibility(state, &guid, service)?;
    super::pending_error(state, GET_ELIGIBLE_RACES)?;
    push_optional_races(state, result.as_deref());
    Ok(1)
}

pub(super) fn get_guild_master_info_for_character(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let guid = read_guid(state, 1, GET_GUILD_INFO)?;
    let info = guild_master_info(state, &guid)?;
    super::pending_error(state, GET_GUILD_INFO)?;
    push_optional_guild_info(state, info.as_ref());
    Ok(1)
}

pub(super) fn get_vas_service_type(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let product_id = read_signed_product_id(state, GET_SERVICE_TYPE)?;
    let service = service_type_for_product(state, product_id)?;
    super::pending_error(state, GET_SERVICE_TYPE)?;
    state.push(service.map_or(Val::Nil, |value| Val::Num(f64::from(value))));
    Ok(1)
}

fn characters_for_realm(
    state: &LuaState,
    realm_address: u32,
    guild_only: bool,
) -> LuaResult<Vec<crate::c_api::store_vas_model::StoreVasCharacterInput>> {
    let sim = borrow_state(state)?;
    let vas = &sim.store_glue.store_secure.vas;
    let mut rows: Vec<_> = vas
        .characters
        .iter()
        .filter(|row| row.realm_address == realm_address)
        .filter(|row| !guild_only || is_guild_character(row, &vas.guild_character_guids))
        .cloned()
        .collect();
    rows.sort_by(compare_characters);
    Ok(rows)
}

fn compare_characters(
    left: &crate::c_api::store_vas_model::StoreVasCharacterInput,
    right: &crate::c_api::store_vas_model::StoreVasCharacterInput,
) -> Ordering {
    right
        .level
        .cmp(&left.level)
        .then_with(|| match (left.guid_order, right.guid_order) {
            (Some(left), Some(right)) => left.cmp(&right),
            _ => Ordering::Equal,
        })
}

fn is_guild_character(
    character: &crate::c_api::store_vas_model::StoreVasCharacterInput,
    guild_guids: &std::collections::HashSet<String>,
) -> bool {
    character.guid.as_deref().is_some_and(|guid| {
        let parsed = parse(guid.as_bytes());
        matches!(parsed, Outcome::Known(_))
            && guild_guids
                .iter()
                .any(|member| guid_matches(&parsed, member))
    })
}

pub(super) fn find_character_object<'a>(
    state: &'a LuaState,
    guid: &Outcome,
) -> LuaResult<Option<crate::c_api::store_vas_model::StoreVasCharacterInput>> {
    if !matches!(guid, Outcome::Known(_)) {
        return Ok(None);
    }
    let sim = borrow_state(state)?;
    Ok(sim
        .store_glue
        .store_secure
        .vas
        .character_objects
        .iter()
        .find(|row| {
            row.guid
                .as_deref()
                .is_some_and(|text| guid_matches(guid, text))
        })
        .cloned())
}

fn account_guid(state: &LuaState, name: &[u8], local: bool) -> LuaResult<Option<String>> {
    let sim = borrow_state(state)?;
    let vas = &sim.store_glue.store_secure.vas;
    let value = if local {
        vas.local_wow_account_guid.as_ref()
    } else {
        vas.wow_account_guids.get(name)
    };
    Ok(value.filter(|guid| !guid.is_empty()).cloned())
}

fn race_eligibility(
    state: &LuaState,
    guid: &Outcome,
    service_type: u8,
) -> LuaResult<Option<Vec<crate::c_api::store_vas_model::StoreVasEligibleRaceInput>>> {
    if !matches!(guid, Outcome::Known(_)) {
        return Ok(None);
    }
    let sim = borrow_state(state)?;
    Ok(sim
        .store_glue
        .store_secure
        .vas
        .race_eligibility
        .iter()
        .find(|entry| entry.service_type == service_type && guid_matches(guid, &entry.guid))
        .and_then(|entry| entry.results.clone()))
}

fn guild_master_info(
    state: &LuaState,
    guid: &Outcome,
) -> LuaResult<Option<crate::c_api::store_vas_model::StoreVasGuildMasterInfoInput>> {
    if !matches!(guid, Outcome::Known(_)) {
        return Ok(None);
    }
    let sim = borrow_state(state)?;
    Ok(sim
        .store_glue
        .store_secure
        .vas
        .guild_master_info
        .iter()
        .find(|entry| guid_matches(guid, &entry.guid))
        .cloned())
}

pub(super) fn service_type_for_product(
    state: &LuaState,
    product_id: i32,
) -> LuaResult<Option<u32>> {
    let sim = borrow_state(state)?;
    let product_type = sim
        .store_glue
        .store_secure
        .catalog
        .products
        .iter()
        .find(|product| product.product_id == product_id)
        .and_then(|product| product.product_type);
    Ok(product_type.and_then(service_type_for_product_type))
}

fn service_type_for_product_type(product_type: u32) -> Option<u32> {
    match product_type {
        7 => Some(0),
        8 => Some(1),
        9 => Some(2),
        10 => Some(3),
        15 => Some(4),
        16 => Some(5),
        19 => Some(6),
        20 => Some(7),
        21 => Some(8),
        22 => Some(9),
        23 => Some(4),
        _ => None,
    }
}

pub(super) fn read_guid(state: &LuaState, index: i32, method: &str) -> LuaResult<Outcome> {
    let value = read_secure_argument(state, index, method)?;
    let bytes = text_bytes(state, value, index, method)?;
    Ok(borrow_state(state)?.store_glue.resolve_guid(&bytes))
}

pub(super) fn read_text_bytes(state: &LuaState, index: i32, method: &str) -> LuaResult<Vec<u8>> {
    let value = read_secure_argument(state, index, method)?;
    text_bytes(state, value, index, method)
}

pub(super) fn text_bytes(
    state: &LuaState,
    value: Val,
    index: i32,
    method: &str,
) -> LuaResult<Vec<u8>> {
    match value {
        Val::Num(number) => Ok(Val::Num(number).to_string().into_bytes()),
        Val::Str(reference) => Ok(state
            .gc
            .string_arena
            .get(reference)
            .map(|string| string.data().to_vec())
            .unwrap_or_default()),
        _ => Err(helpers::bad_argument(index, method, INVALID_ARGUMENT)),
    }
}

pub(super) fn read_realm_address(state: &LuaState, index: i32, method: &str) -> LuaResult<u32> {
    let value = read_secure_argument(state, index, method)?;
    let number = numeric_value(state, value)
        .filter(|number| number.is_finite())
        .ok_or_else(|| helpers::bad_argument(index, method, INVALID_ARGUMENT))?;
    Ok(number.trunc() as u32)
}

pub(super) fn read_signed_product_id(state: &LuaState, method: &str) -> LuaResult<i32> {
    let value = read_secure_argument(state, 1, method)?;
    let number = numeric_value(state, value)
        .filter(|number| number.is_finite() && (i32::MIN as f64..=i32::MAX as f64).contains(number))
        .ok_or_else(|| helpers::bad_argument(1, method, INVALID_ARGUMENT))?;
    Ok(number.trunc() as i32)
}

pub(super) fn numeric_value(state: &LuaState, value: Val) -> Option<f64> {
    match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| helpers::parse_numeric_string(string.data())),
        _ => None,
    }
}

pub(super) fn read_truthy_argument(state: &LuaState, index: i32, method: &str) -> LuaResult<bool> {
    let value = read_secure_argument(state, index, method)?;
    Ok(value != Val::Bool(false))
}

fn read_service_type(state: &LuaState, index: i32, method: &str) -> LuaResult<u8> {
    let value = read_secure_argument(state, index, method)?;
    let number = numeric_value(state, value).unwrap_or(0.0);
    if !number.is_finite() {
        return Err(helpers::bad_argument(index, method, INVALID_ARGUMENT));
    }
    let service = number.trunc() as i128 as u8;
    if service > 9 {
        return Err(helpers::bad_argument(index, method, INVALID_ARGUMENT));
    }
    Ok(service)
}

pub(super) fn read_secure_argument(state: &LuaState, index: i32, method: &str) -> LuaResult<Val> {
    c_store_glue::read_argument_at(state, index, method, INVALID_ARGUMENT, "C_StoreSecure")
}

pub(super) fn guid_matches(guid: &Outcome, text: &str) -> bool {
    matches!((guid, parse(text.as_bytes())), (Outcome::Known(left), Outcome::Known(right)) if *left == right)
}

fn push_realms(
    state: &mut LuaState,
    realms: &[crate::c_api::store_vas_model::StoreVasRealmInput],
    vas: bool,
) {
    let result = create_table(state);
    state.push(result);
    for (index, realm) in realms.iter().enumerate() {
        let row = create_table(state);
        state.push(row);
        set_number(
            state,
            row,
            "virtualRealmAddress",
            f64::from(realm.virtual_realm_address),
        );
        set_text(state, row, "realmName", &realm.realm_name);
        let count = if vas { 0 } else { realm.character_count };
        set_number(state, row, "characterCount", f64::from(count));
        set_optional_bool(state, row, "pvp", realm.pvp);
        set_optional_bool(state, row, "rp", realm.rp);
        helpers::set_table_array(state, result, index as i64 + 1, row);
        state.top -= 1;
    }
}

fn push_characters(
    state: &mut LuaState,
    characters: &[crate::c_api::store_vas_model::StoreVasCharacterInput],
) {
    let result = create_table(state);
    state.push(result);
    for (index, character) in characters.iter().enumerate() {
        let row = push_character(state, character);
        helpers::set_table_array(state, result, index as i64 + 1, row);
    }
}

fn push_optional_character(
    state: &mut LuaState,
    character: Option<&crate::c_api::store_vas_model::StoreVasCharacterInput>,
) {
    let value = character.map_or(Val::Nil, |character| push_character(state, character));
    state.push(value);
}

fn push_character(
    state: &mut LuaState,
    character: &crate::c_api::store_vas_model::StoreVasCharacterInput,
) -> Val {
    let row = create_table(state);
    state.push(row);
    set_optional_text(state, row, "name", character.name.as_deref());
    set_optional_text(state, row, "className", character.class_name.as_deref());
    set_optional_text(state, row, "raceName", character.race_name.as_deref());
    set_number(state, row, "level", f64::from(character.level));
    set_optional_text(
        state,
        row,
        "classFileName",
        character.class_file_name.as_deref(),
    );
    set_optional_text(
        state,
        row,
        "raceFileName",
        character.race_file_name.as_deref(),
    );
    set_guid(state, row, "guid", character.guid.as_deref());
    set_number(state, row, "wowAccount", f64::from(character.wow_account));
    set_number(
        state,
        row,
        "currentServer",
        f64::from(character.current_server),
    );
    set_number(state, row, "faction", f64::from(character.faction));
    set_number(state, row, "sex", f64::from(character.sex));
    set_optional_text(
        state,
        row,
        "createScreenIconAtlas",
        character.create_screen_icon_atlas.as_deref(),
    );
    state.top -= 1;
    row
}

fn push_optional_races(
    state: &mut LuaState,
    races: Option<&[crate::c_api::store_vas_model::StoreVasEligibleRaceInput]>,
) {
    let Some(races) = races else {
        state.push(Val::Nil);
        return;
    };
    let result = create_table(state);
    state.push(result);
    for (index, race) in races.iter().enumerate() {
        let row = create_table(state);
        state.push(row);
        set_text(state, row, "raceName", &race.race_name);
        set_bool(state, row, "isAlliedRace", race.is_allied_race);
        set_bool(
            state,
            row,
            "isHeritageArmorUnlocked",
            race.is_heritage_armor_unlocked,
        );
        helpers::set_table_array(state, result, index as i64 + 1, row);
        state.top -= 1;
    }
}

fn push_optional_guild_info(
    state: &mut LuaState,
    info: Option<&crate::c_api::store_vas_model::StoreVasGuildMasterInfoInput>,
) {
    let Some(info) = info else {
        state.push(Val::Nil);
        return;
    };
    let saved_top = state.top;
    let result = create_table(state);
    state.push(result);
    set_text(state, result, "guildName", &info.guild_name);
    let members = create_table(state);
    set_static(state, result, "guildMemberInfos", members);
    state.push(members);
    for (index, member) in info.members.iter().enumerate() {
        let row = create_table(state);
        state.push(row);
        set_guid(state, row, "guid", member.guid.as_deref());
        set_text(state, row, "memberName", &member.member_name);
        helpers::set_table_array(state, members, index as i64 + 1, row);
        state.top -= 1;
    }
    state.top = saved_top;
    state.push(result);
}

fn set_text(state: &mut LuaState, table: Val, key: &'static str, text: &str) {
    let value = create_string(state, text);
    set_static(state, table, key, value);
}

fn set_optional_text(state: &mut LuaState, table: Val, key: &'static str, text: Option<&str>) {
    let value = text.map_or(Val::Nil, |text| create_string(state, text));
    set_static(state, table, key, value);
}

fn set_guid(state: &mut LuaState, table: Val, key: &'static str, text: Option<&str>) {
    set_optional_text(state, table, key, text.filter(|text| !text.is_empty()));
}

pub(super) fn push_guid(state: &mut LuaState, text: Option<&str>) {
    let value = text
        .filter(|text| !text.is_empty())
        .map_or(Val::Nil, |text| create_string(state, text));
    state.push(value);
}

fn set_number(state: &mut LuaState, table: Val, key: &'static str, number: f64) {
    set_static(state, table, key, Val::Num(number));
}

fn set_bool(state: &mut LuaState, table: Val, key: &'static str, value: bool) {
    set_static(state, table, key, Val::Bool(value));
}

fn set_optional_bool(state: &mut LuaState, table: Val, key: &'static str, value: Option<bool>) {
    let value = value.map_or(Val::Nil, Val::Bool);
    set_static(state, table, key, value);
}
