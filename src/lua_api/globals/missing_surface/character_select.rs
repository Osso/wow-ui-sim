//! Glue-screen character-select helpers backed by seeded battle.net game
//! accounts.

use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_api::state_types::BnetGameAccount;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const DEFAULT_WARBAND_GROUP_COUNT: f64 = 4.0;
const CHARACTER_SELECT_READY_FLAG: &str = "__wow_character_screen_initialized";
const IN_CHARACTER_SELECT_FLAG: &str = "__wow_in_character_select";

pub(super) fn register_character_select_surface(state: &mut LuaState) -> LuaResult<()> {
    register_character_select_bootstrap(state)?;
    register_character_select_frame_state(state)?;
    register_character_select_character_queries(state)?;
    register_character_select_character_details(state)?;
    Ok(())
}

fn register_character_select_bootstrap(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        state.global,
        "InitializeCharacterScreenData",
        initialize_character_screen_data,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "SetWorldFrameStrata",
        set_world_frame_strata,
    )?;
    Ok(())
}

fn register_character_select_frame_state(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        state.global,
        "SetCharSelectModelFrame",
        set_char_select_model_frame,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "SetCharSelectMapSceneFrame",
        set_char_select_map_scene_frame,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "SetInCharacterSelect",
        set_in_character_select,
    )?;
    Ok(())
}

fn register_character_select_character_queries(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        state.global,
        "GetMaxWarbandGroupCount",
        get_max_warband_group_count,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetActiveTimerunningSeasonID",
        get_active_timerunning_season_id,
    )?;
    table_set_rust_fn_static(state, state.global, "GetNumCharacters", get_num_characters)?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetCharacterSelection",
        get_character_selection,
    )?;
    Ok(())
}

fn register_character_select_character_details(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(state, state.global, "GetCharacterGUID", get_character_guid)?;
    table_set_rust_fn_static(state, state.global, "GetCharacterRace", get_character_race)?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetBasicCharacterInfo",
        get_basic_character_info,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetServiceCharacterInfo",
        get_service_character_info,
    )?;
    Ok(())
}

fn initialize_character_screen_data(state: &mut LuaState) -> LuaResult<u32> {
    set_global_flag(state, CHARACTER_SELECT_READY_FLAG)
}

fn set_world_frame_strata(state: &mut LuaState) -> LuaResult<u32> {
    let frame = crate::lua_bridge::stack_val(state, 1);
    let Some(frame_id) = crate::lua_api::methods::extract_frame_id(state, frame) else {
        return Ok(0);
    };
    let mut sim = crate::lua_api::methods::borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(frame_id) {
        frame.frame_strata = crate::widget::FrameStrata::Background;
    }
    Ok(0)
}

fn set_char_select_model_frame(state: &mut LuaState) -> LuaResult<u32> {
    set_char_select_frame_name(state, "__wow_char_select_model_frame_name")
}

fn set_char_select_map_scene_frame(state: &mut LuaState) -> LuaResult<u32> {
    set_char_select_frame_name(state, "__wow_char_select_map_scene_frame_name")
}

fn set_char_select_frame_name(state: &mut LuaState, slot: &str) -> LuaResult<u32> {
    let frame_name = match crate::lua_bridge::stack_val(state, 1) {
        Val::Str(_) => {
            crate::lua_api::methods::val_to_string(state, crate::lua_bridge::stack_val(state, 1))
                .unwrap_or_default()
        }
        _ => String::new(),
    };
    let frame_name = create_string(state, &frame_name);
    table_set(state, Val::Table(state.global), slot, frame_name);
    Ok(0)
}

fn set_in_character_select(state: &mut LuaState) -> LuaResult<u32> {
    set_global_flag(state, IN_CHARACTER_SELECT_FLAG)
}

fn set_global_flag(state: &mut LuaState, key: &str) -> LuaResult<u32> {
    table_set(state, Val::Table(state.global), key, Val::Bool(true));
    Ok(0)
}

fn get_max_warband_group_count(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(DEFAULT_WARBAND_GROUP_COUNT));
    Ok(1)
}

fn get_active_timerunning_season_id(state: &mut LuaState) -> LuaResult<u32> {
    let season = borrow_state(state)?.timerunning_season_id;
    match season {
        Some(id) => state.push(Val::Num(id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_num_characters(state: &mut LuaState) -> LuaResult<u32> {
    let count = seeded_character_accounts(state).len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_character_selection(state: &mut LuaState) -> LuaResult<u32> {
    let count = seeded_character_accounts(state).len();
    state.push(Val::Num(if count > 0 { 1.0 } else { 0.0 }));
    Ok(1)
}

fn get_character_guid(state: &mut LuaState) -> LuaResult<u32> {
    let character_index = i32::from_stack(state, 1)?;
    let guid = seeded_character_accounts(state)
        .get(usize::try_from(character_index.saturating_sub(1)).unwrap_or(usize::MAX))
        .map(|ga| ga.wow_account_guid.clone());
    match guid {
        Some(guid) => {
            let guid_value = create_string(state, &guid);
            state.push(guid_value);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_character_race(state: &mut LuaState) -> LuaResult<u32> {
    let character_index = i32::from_stack(state, 1)?;
    let Some(account) = seeded_character_accounts(state)
        .get(usize::try_from(character_index.saturating_sub(1)).unwrap_or(usize::MAX))
        .cloned()
    else {
        state.push(Val::Nil);
        state.push(Val::Nil);
        return Ok(2);
    };
    let race_id = match account.race_name.as_str() {
        "Human" => 1.0,
        "Dwarf" => 3.0,
        _ => 0.0,
    };
    state.push(Val::Num(race_id));
    let race_name = create_string(state, &account.race_name);
    state.push(race_name);
    Ok(2)
}

fn get_basic_character_info(state: &mut LuaState) -> LuaResult<u32> {
    let guid = match crate::lua_bridge::stack_val(state, 1) {
        Val::Str(_) => {
            crate::lua_api::methods::val_to_string(state, crate::lua_bridge::stack_val(state, 1))
                .unwrap_or_default()
        }
        _ => {
            state.push(Val::Nil);
            return Ok(1);
        }
    };
    let Some(account) = seeded_character_accounts(state)
        .into_iter()
        .find(|ga| ga.wow_account_guid == guid)
    else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let character_info = build_character_info_table(state, &account);
    state.push(character_info);
    Ok(1)
}

fn get_service_character_info(state: &mut LuaState) -> LuaResult<u32> {
    let table = create_table(state);
    state.push(table);
    Ok(1)
}

fn seeded_character_accounts(state: &mut LuaState) -> Vec<BnetGameAccount> {
    borrow_state(state)
        .map(|sim| {
            sim.bnet_friends
                .iter()
                .flat_map(|friend| friend.game_accounts.iter())
                .filter(|ga| ga.client_program == "WoW")
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn build_character_info_table(state: &mut LuaState, account: &BnetGameAccount) -> Val {
    let table = create_table(state);
    set_character_info_strings(state, table, account);
    set_character_info_numbers(state, table, account);
    set_character_info_flags(state, table);
    set_character_info_empty_tables(state, table);
    table
}

fn set_character_info_strings(state: &mut LuaState, table: Val, account: &BnetGameAccount) {
    let name_field = if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Ptr {
        "characterName"
    } else {
        "name"
    };
    set_string_field(state, table, name_field, &account.character_name);
    set_string_field(state, table, "realmName", &account.realm_name);
    set_string_field(state, table, "realmAddress", &account.realm_display_name);
    set_string_field(state, table, "guid", &account.wow_account_guid);
    set_string_field(state, table, "className", &account.class_name);
    set_string_field(state, table, "classFilename", &account.class_name);
    set_string_field(state, table, "areaName", &account.area_name);
    set_string_field(state, table, "faction", &account.faction_name);
}

fn set_character_info_numbers(state: &mut LuaState, table: Val, account: &BnetGameAccount) {
    table_set(
        state,
        table,
        "experienceLevel",
        Val::Num(account.character_level as f64),
    );
    for key in [
        "raceID",
        "specID",
        "profession0",
        "profession1",
        "money",
        "lastLoginBuild",
    ] {
        table_set(state, table, key, Val::Num(0.0));
    }
}

fn set_character_info_flags(state: &mut LuaState, table: Val) {
    for key in [
        "boostInProgress",
        "isTrialBoostCompleted",
        "revokedCharacterUpgrade",
        "isExpansionTrialCharacter",
        "isLocked",
        "isLockedByExpansion",
        "isRevokedCharacterUpgrade",
        "isTrialBoost",
        "hasCustomize",
        "customizeDisabled",
        "hasFactionChange",
        "hasRaceChange",
    ] {
        table_set(state, table, key, Val::Bool(false));
    }
}

fn set_character_info_empty_tables(state: &mut LuaState, table: Val) {
    let mail_senders = create_table(state);
    table_set(state, table, "mailSenders", mail_senders);
}

fn set_string_field(state: &mut LuaState, table: Val, key: &str, value: &str) {
    let value = create_string(state, value);
    table_set(state, table, key, value);
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn character_select_globals_setters_store_expected_flags_and_names() {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        let (initialized, in_character_select, model_name, map_scene_name): (
            bool,
            bool,
            String,
            String,
        ) = env
            .eval(
                r#"
                InitializeCharacterScreenData()
                SetInCharacterSelect()
                SetCharSelectModelFrame("ModelFFX")
                SetCharSelectMapSceneFrame("MapScene")
                return __wow_character_screen_initialized,
                       __wow_in_character_select,
                       __wow_char_select_model_frame_name,
                       __wow_char_select_map_scene_frame_name
                "#,
            )
            .expect("character select globals should be stored");

        assert!(initialized);
        assert!(in_character_select);
        assert_eq!(model_name, "ModelFFX");
        assert_eq!(map_scene_name, "MapScene");
    }
}
