//! Mists legacy reputation globals over the shared C_Reputation scenario rows.
use super::helpers::set_table_array;
use crate::lua_api::methods::{
    borrow_state_mut, create_string, table_get_static, table_set_static,
};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::cmp::Ordering;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(state, state.global, "GetNumFactions", get_num_factions)?;
    table_set_rust_fn_static(state, state.global, "GetFactionInfo", get_faction_info)?;
    table_set_rust_fn_static(
        state,
        state.global,
        "CollapseFactionHeader",
        collapse_faction_header,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "ExpandFactionHeader",
        expand_faction_header,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "CollapseAllFactionHeaders",
        collapse_all_faction_headers,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "ExpandAllFactionHeaders",
        expand_all_faction_headers,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetSelectedFaction",
        get_selected_faction,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "SetSelectedFaction",
        set_selected_faction,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "SetWatchedFactionIndex",
        set_watched_faction_index,
    )
}

const DEFAULT_HEADER_SORT_VALUE: f64 = 0.0;
const LEGACY_HEADER_GROUP_KEY: &str = "__wowLegacyReputationHeader";

struct ReputationRow {
    value: Val,
    name: Option<Vec<u8>>,
    has_record: bool,
}

struct ReputationGroup {
    header: ReputationRow,
    is_special: bool,
    sort_value: f64,
    collapsed: bool,
    rows: Vec<ReputationRow>,
}

struct ReputationList {
    rows: Vec<Val>,
    visible_count: usize,
}

fn reputation_state(state: &mut LuaState) -> Val {
    table_get_static(state, Val::Table(state.global), "__wow_reputation_state")
}

fn rebuild_faction_list(state: &mut LuaState) -> ReputationList {
    let state_table = reputation_state(state);
    let rows_table = table_get_static(state, state_table, "factions");
    let input_rows = read_faction_rows(state, rows_table);
    let (leading_rows, mut groups) = assign_faction_groups(state, input_rows);
    groups.sort_unstable_by(compare_reputation_groups);

    let mut visible_rows: Vec<Val> = leading_rows.iter().map(|row| row.value).collect();
    let mut hidden_rows = Vec::new();
    for group in &mut groups {
        sort_reputation_rows(&mut group.rows);
        visible_rows.push(group.header.value);
        if group.collapsed {
            // Native cross-group hidden order is undefined; this is storage order only.
            hidden_rows.extend(group.rows.drain(..).map(|row| row.value));
        } else {
            visible_rows.extend(group.rows.drain(..).map(|row| row.value));
        }
    }

    let visible_count = visible_rows.len();
    visible_rows.extend(hidden_rows);
    for (index, row) in visible_rows.iter().copied().enumerate() {
        set_table_array(state, rows_table, index as i64 + 1, row);
    }

    ReputationList {
        rows: visible_rows,
        visible_count,
    }
}

fn read_faction_rows(state: &LuaState, rows: Val) -> Vec<Val> {
    let mut result = Vec::new();
    let mut index = 1;
    loop {
        let row = integer_table_value(state, rows, index);
        if row == Val::Nil {
            return result;
        }
        result.push(row);
        index += 1;
    }
}

fn integer_table_value(state: &LuaState, table: Val, index: i32) -> Val {
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.get_int(i64::from(index)))
        .unwrap_or(Val::Nil)
}

fn assign_faction_groups(
    state: &mut LuaState,
    rows: Vec<Val>,
) -> (Vec<ReputationRow>, Vec<ReputationGroup>) {
    let mut leading_rows = Vec::new();
    let mut groups = Vec::new();
    let mut current_group = None;

    for row in rows {
        if is_header(state, row) {
            groups.push(read_reputation_group(state, row));
            current_group = Some(groups.len() - 1);
            continue;
        }
        assign_reputation_row(state, row, &mut groups, current_group, &mut leading_rows);
    }

    (leading_rows, groups)
}

fn read_reputation_group(state: &mut LuaState, row: Val) -> ReputationGroup {
    ReputationGroup {
        header: read_reputation_row(state, row),
        is_special: boolean_field(state, row, "isSpecialHeader"),
        // The native numeric field is sorted descending; Expansion is inferred.
        // `headerSortValue` is simulator fixture metadata, not a claimed native name.
        sort_value: number_or(state, row, "headerSortValue", DEFAULT_HEADER_SORT_VALUE),
        collapsed: is_collapsed(state, row),
        rows: Vec::new(),
    }
}

fn assign_reputation_row(
    state: &mut LuaState,
    row: Val,
    groups: &mut [ReputationGroup],
    current_group: Option<usize>,
    leading_rows: &mut Vec<ReputationRow>,
) {
    let faction = read_reputation_row(state, row);
    let existing_header = table_get_static(state, row, LEGACY_HEADER_GROUP_KEY);
    let group_index = groups
        .iter()
        .position(|group| group.header.value == existing_header)
        .or(current_group);
    if let Some(group_index) = group_index {
        let header = groups[group_index].header.value;
        table_set_static(state, row, LEGACY_HEADER_GROUP_KEY, header);
        groups[group_index].rows.push(faction);
    } else {
        leading_rows.push(faction);
    }
}

fn read_reputation_row(state: &mut LuaState, row: Val) -> ReputationRow {
    ReputationRow {
        value: row,
        name: read_string_field(state, row, "name"),
        // A nonzero scenario factionID marks a record-backed row (INFERRED mapping).
        has_record: number_field(state, row, "factionID").is_some_and(|id| id != 0.0),
    }
}

fn read_string_field(state: &mut LuaState, row: Val, key: &'static str) -> Option<Vec<u8>> {
    let Val::Str(reference) = table_get_static(state, row, key) else {
        return None;
    };
    state
        .gc
        .string_arena
        .get(reference)
        .map(|string| string.data().to_vec())
}

fn compare_reputation_groups(left: &ReputationGroup, right: &ReputationGroup) -> Ordering {
    right
        .is_special
        .cmp(&left.is_special)
        .then_with(|| right.sort_value.total_cmp(&left.sort_value))
        .then_with(|| left.header.name.cmp(&right.header.name))
        .then_with(|| right.header.has_record.cmp(&left.header.has_record))
}

fn sort_reputation_rows(rows: &mut Vec<ReputationRow>) {
    let (mut named_rows, missing_records): (Vec<_>, Vec<_>) = std::mem::take(rows)
        .into_iter()
        .partition(|row| row.has_record && row.name.is_some());
    named_rows.sort_unstable_by(|left, right| left.name.cmp(&right.name));
    rows.extend(named_rows);
    rows.extend(missing_records);
}

fn is_header(state: &mut LuaState, row: Val) -> bool {
    matches!(table_get_static(state, row, "isHeader"), Val::Bool(true))
}

fn is_collapsed(state: &mut LuaState, row: Val) -> bool {
    matches!(table_get_static(state, row, "isCollapsed"), Val::Bool(true))
}

fn number_field(state: &mut LuaState, row: Val, key: &'static str) -> Option<f64> {
    match table_get_static(state, row, key) {
        Val::Num(value) => Some(value),
        _ => None,
    }
}

fn number_or(state: &mut LuaState, row: Val, key: &'static str, default: f64) -> f64 {
    number_field(state, row, key).unwrap_or(default)
}

fn boolean_field(state: &mut LuaState, row: Val, key: &'static str) -> bool {
    matches!(table_get_static(state, row, key), Val::Bool(true))
}

fn get_num_factions(state: &mut LuaState) -> LuaResult<u32> {
    let list = rebuild_faction_list(state);
    state.push(Val::Num(list.visible_count as f64));
    Ok(1)
}

fn get_faction_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let list = rebuild_faction_list(state);
    let row = index
        .checked_sub(1)
        .and_then(|offset| usize::try_from(offset).ok())
        .and_then(|offset| list.rows.get(offset).copied())
        .unwrap_or(Val::Nil);
    if row == Val::Nil {
        return Ok(push_missing_faction_info(state));
    }
    Ok(push_faction_info(state, row))
}

fn push_missing_faction_info(state: &mut LuaState) -> u32 {
    state.push(Val::Nil);
    state.push(Val::Nil);
    state.push(Val::Num(1.0));
    for _ in 0..3 {
        state.push(Val::Num(0.0));
    }
    for _ in 0..7 {
        state.push(Val::Nil);
    }
    13
}

fn push_faction_info(state: &mut LuaState, row: Val) -> u32 {
    let name = table_get_static(state, row, "name");
    state.push(name);
    let description = match table_get_static(state, row, "description") {
        Val::Nil | Val::Bool(false) => create_string(state, ""),
        value => value,
    };
    state.push(description);

    let standing_id = number_field(state, row, "reaction")
        .or_else(|| number_field(state, row, "standing"))
        .unwrap_or(4.0);
    let standing_id = if (1.0..=8.0).contains(&standing_id) {
        standing_id
    } else {
        4.0
    };
    state.push(Val::Num(standing_id));
    let bar_min = number_field(state, row, "currentReactionThreshold")
        .or_else(|| number_field(state, row, "bottom"))
        .unwrap_or(0.0);
    let bar_max = number_field(state, row, "nextReactionThreshold")
        .or_else(|| number_field(state, row, "topValue"))
        .or_else(|| number_field(state, row, "top"))
        .unwrap_or(0.0);
    let bar_value = number_field(state, row, "currentStanding")
        .or_else(|| number_field(state, row, "standing"))
        .unwrap_or(0.0);
    state.push(Val::Num(bar_min));
    state.push(Val::Num(bar_max));
    state.push(Val::Num(bar_value));

    let is_header = is_header(state, row);
    let at_war = boolean_field(state, row, "atWarWith");
    let can_toggle_at_war = boolean_field(state, row, "canToggleAtWar");
    let collapsed = is_collapsed(state, row);
    let is_watched = boolean_field(state, row, "isWatched");
    let is_child = boolean_field(state, row, "isChild");
    let faction_id = number_or(state, row, "factionID", 0.0);
    state.push(Val::Bool(at_war));
    state.push(Val::Bool(can_toggle_at_war));
    state.push(Val::Bool(is_header));
    state.push(Val::Bool(collapsed));
    state.push(Val::Bool(!is_header));
    state.push(Val::Bool(is_watched));
    state.push(Val::Bool(is_child));
    state.push(Val::Num(faction_id));

    if is_header {
        14
    } else {
        // R21 proves the 16-value arity, not the final two field meanings.
        state.push(Val::Nil);
        state.push(Val::Nil);
        16
    }
}

fn collapse_faction_header(state: &mut LuaState) -> LuaResult<u32> {
    set_faction_header_collapsed(state, true)
}

fn expand_faction_header(state: &mut LuaState) -> LuaResult<u32> {
    set_faction_header_collapsed(state, false)
}

fn set_faction_header_collapsed(state: &mut LuaState, collapsed: bool) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let list = rebuild_faction_list(state);
    let selected = index
        .checked_sub(1)
        .and_then(|offset| usize::try_from(offset).ok())
        .and_then(|offset| list.rows.get(offset).copied());

    if let Some(row) = selected.filter(|&row| is_header(state, row)) {
        table_set_static(state, row, "isCollapsed", Val::Bool(collapsed));
    } else {
        set_all_headers_collapsed(state, &list.rows, collapsed);
    }
    rebuild_faction_list(state);
    Ok(0)
}

fn collapse_all_faction_headers(state: &mut LuaState) -> LuaResult<u32> {
    let list = rebuild_faction_list(state);
    set_all_headers_collapsed(state, &list.rows, true);
    rebuild_faction_list(state);
    Ok(0)
}

fn expand_all_faction_headers(state: &mut LuaState) -> LuaResult<u32> {
    let list = rebuild_faction_list(state);
    set_all_headers_collapsed(state, &list.rows, false);
    rebuild_faction_list(state);
    Ok(0)
}

fn set_all_headers_collapsed(state: &mut LuaState, rows: &[Val], collapsed: bool) {
    for &row in rows {
        if is_header(state, row) {
            table_set_static(state, row, "isCollapsed", Val::Bool(collapsed));
        }
    }
}

fn get_selected_faction(state: &mut LuaState) -> LuaResult<u32> {
    let list = rebuild_faction_list(state);
    let reputation_state = reputation_state(state);
    let selected_id = number_or(state, reputation_state, "legacySelectedFactionID", 0.0);
    if selected_id == 0.0 {
        state.push(Val::Num(0.0));
        return Ok(1);
    }
    for (index, row) in list
        .rows
        .iter()
        .take(list.visible_count)
        .copied()
        .enumerate()
    {
        if !is_header(state, row) && number_field(state, row, "factionID") == Some(selected_id) {
            state.push(Val::Num((index + 1) as f64));
            return Ok(1);
        }
    }
    state.push(Val::Num(0.0));
    Ok(1)
}

fn set_selected_faction(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let list = rebuild_faction_list(state);
    let selected_row = index
        .checked_sub(1)
        .and_then(|offset| usize::try_from(offset).ok())
        .filter(|&offset| offset < list.visible_count)
        .and_then(|offset| list.rows.get(offset).copied());
    let selected_id = match selected_row {
        Some(row) if !is_header(state, row) => number_field(state, row, "factionID"),
        _ => None,
    }
    .unwrap_or(0.0);
    let reputation_state = reputation_state(state);
    table_set_static(
        state,
        reputation_state,
        "legacySelectedFactionID",
        Val::Num(selected_id),
    );
    Ok(0)
}

fn set_watched_faction_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let list = rebuild_faction_list(state);
    let row_count = list.rows.len() as i64;
    let reputation_slot = if index == 0 || i64::from(index) >= row_count + 2 {
        Some(-1)
    } else {
        index
            .checked_sub(1)
            .and_then(|offset| usize::try_from(offset).ok())
            .and_then(|offset| list.rows.get(offset).copied())
            .and_then(|row| number_field(state, row, "reputationIndex"))
            .map(|slot| slot as i16)
    };

    if let Some(slot) = reputation_slot {
        borrow_state_mut(state)?.last_watched_reputation_slot_request = Some(slot);
    }
    Ok(0)
}
