//! Explicit Recent Allies inputs and independent, stack-rooted query snapshots.

use super::{ensure_namespace, helpers::set_table_array};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set_static};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Disabled and empty by default: inferred simulator policy, not native evidence.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecentAlliesInput {
    pub enabled: bool,
    pub entries: Vec<RecentAllyData>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentAllyData {
    pub state_data: RecentAllyStateData,
    pub character_data: RecentAllyCharacterData,
    pub interaction_data: RecentAllyInteractionData,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentAllyStateData {
    pub is_online: bool,
    pub is_dnd: bool,
    pub is_afk: bool,
    pub is_converted_legacy_friend: bool,
    pub pin_expiration_date: Option<i64>,
    pub friend_request_sent_this_session: bool,
    pub current_location: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentAllyCharacterData {
    pub guid: String,
    pub name: String,
    pub full_name: String,
    pub realm_name: String,
    pub level: i32,
    pub class_id: i32,
    pub race_id: i32,
    pub sex: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentAllyInteractionData {
    pub interactions: Vec<RecentAllyInteraction>,
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentAllyInteraction {
    /// Numeric value of the documented RolodexType enum; no inferred variants.
    pub interaction_type: i32,
    pub description: String,
    pub timestamp: i64,
    pub context_data: RecentAllyInteractionContextData,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentAllyInteractionContextData {
    pub item_id: Option<i64>,
    pub location_name: Option<String>,
    pub activity_difficulty_id: Option<i32>,
    pub activity_difficulty_level: Option<i32>,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_RecentAllies")?;
    table_set_rust_fn_static(state, namespace, "GetRecentAllies", get_recent_allies)
}

fn get_recent_allies(state: &mut LuaState) -> LuaResult<u32> {
    let entries = {
        let sim = borrow_state(state)?;
        if !sim.recent_allies.enabled {
            return Ok(0);
        }
        sim.recent_allies.entries.clone()
    };
    let sequence = push_snapshot_table(state);
    for (index, ally) in entries.iter().enumerate() {
        let row = push_ally_table(state, ally);
        set_table_array(state, sequence, (index + 1) as i64, row);
        state.top -= 1;
    }
    Ok(1)
}

/// Builders leave one table on the VM stack, rooted across all child allocations.
fn push_snapshot_table(state: &mut LuaState) -> Val {
    let table = create_table(state);
    state.push(table);
    table
}

/// Transfer the child's stack root to its already-rooted parent.
fn attach_snapshot_field(state: &mut LuaState, parent: Val, key: &'static str, child: Val) {
    table_set_static(state, parent, key, child);
    state.top -= 1;
}

fn push_ally_table(state: &mut LuaState, ally: &RecentAllyData) -> Val {
    let row = push_snapshot_table(state);
    let status = push_state_table(state, &ally.state_data);
    attach_snapshot_field(state, row, "stateData", status);
    let character = push_character_table(state, &ally.character_data);
    attach_snapshot_field(state, row, "characterData", character);
    let interactions = push_interaction_data_table(state, &ally.interaction_data);
    attach_snapshot_field(state, row, "interactionData", interactions);
    row
}

fn push_state_table(state: &mut LuaState, input: &RecentAllyStateData) -> Val {
    let row = push_snapshot_table(state);
    for (key, value) in [
        ("isOnline", input.is_online),
        ("isDND", input.is_dnd),
        ("isAFK", input.is_afk),
        ("isConvertedLegacyFriend", input.is_converted_legacy_friend),
        (
            "friendRequestSentThisSession",
            input.friend_request_sent_this_session,
        ),
    ] {
        table_set_static(state, row, key, Val::Bool(value));
    }
    set_optional_number(state, row, "pinExpirationDate", input.pin_expiration_date);
    set_optional_string(
        state,
        row,
        "currentLocation",
        input.current_location.as_deref(),
    );
    row
}

fn push_character_table(state: &mut LuaState, input: &RecentAllyCharacterData) -> Val {
    let row = push_snapshot_table(state);
    for (key, value) in [
        ("guid", input.guid.as_str()),
        ("name", input.name.as_str()),
        ("fullName", input.full_name.as_str()),
        ("realmName", input.realm_name.as_str()),
    ] {
        set_optional_string(state, row, key, Some(value));
    }
    for (key, value) in [
        ("level", input.level),
        ("classID", input.class_id),
        ("raceID", input.race_id),
        ("sex", input.sex),
    ] {
        table_set_static(state, row, key, Val::Num(value as f64));
    }
    row
}

fn push_interaction_data_table(state: &mut LuaState, input: &RecentAllyInteractionData) -> Val {
    let row = push_snapshot_table(state);
    set_optional_string(state, row, "note", input.note.as_deref());
    let sequence = push_snapshot_table(state);
    for (index, interaction) in input.interactions.iter().enumerate() {
        let child = push_interaction_table(state, interaction);
        set_table_array(state, sequence, (index + 1) as i64, child);
        state.top -= 1;
    }
    attach_snapshot_field(state, row, "interactions", sequence);
    row
}

fn push_interaction_table(state: &mut LuaState, input: &RecentAllyInteraction) -> Val {
    let row = push_snapshot_table(state);
    table_set_static(state, row, "type", Val::Num(input.interaction_type as f64));
    table_set_static(state, row, "timestamp", Val::Num(input.timestamp as f64));
    set_optional_string(state, row, "description", Some(&input.description));
    let context = push_context_table(state, &input.context_data);
    attach_snapshot_field(state, row, "contextData", context);
    row
}

fn push_context_table(state: &mut LuaState, input: &RecentAllyInteractionContextData) -> Val {
    let row = push_snapshot_table(state);
    for (key, value) in [
        ("itemID", input.item_id),
        (
            "activityDifficultyID",
            input.activity_difficulty_id.map(i64::from),
        ),
        (
            "activityDifficultyLevel",
            input.activity_difficulty_level.map(i64::from),
        ),
    ] {
        set_optional_number(state, row, key, value);
    }
    set_optional_string(state, row, "locationName", input.location_name.as_deref());
    row
}

fn set_optional_number(state: &mut LuaState, row: Val, key: &'static str, value: Option<i64>) {
    if let Some(value) = value {
        table_set_static(state, row, key, Val::Num(value as f64));
    }
}

fn set_optional_string(state: &mut LuaState, row: Val, key: &'static str, value: Option<&str>) {
    if let Some(value) = value {
        let text = create_string(state, value);
        // table_set_static roots the value while interning the field name.
        table_set_static(state, row, key, text);
    }
}
