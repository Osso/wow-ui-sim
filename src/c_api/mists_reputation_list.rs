//! Mists legacy reputation globals over the shared C_Reputation scenario rows.
use crate::lua_api::methods::{create_string, table_get, table_get_static, table_set_static};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

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
    )
}

fn reputation_state(state: &mut LuaState) -> Val {
    table_get_static(state, Val::Table(state.global), "__wow_reputation_state")
}

fn faction_rows(state: &mut LuaState) -> Vec<Val> {
    let reputation_state = reputation_state(state);
    let rows = table_get_static(state, reputation_state, "factions");
    let mut result = Vec::new();
    let mut index = 1;
    loop {
        let row = table_get(state, rows, &index.to_string());
        if row == Val::Nil {
            return result;
        }
        result.push(row);
        index += 1;
    }
}

fn visible_faction_rows(state: &mut LuaState, rows: &[Val]) -> Vec<Val> {
    let mut visible = Vec::new();
    let mut hide_children = false;
    for &row in rows {
        if is_header(state, row) {
            hide_children = is_collapsed(state, row);
            visible.push(row);
        } else if !hide_children {
            visible.push(row);
        }
    }
    visible
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
    let rows = faction_rows(state);
    let count = visible_faction_rows(state, &rows).len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_faction_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let rows = faction_rows(state);
    let row = index
        .checked_sub(1)
        .and_then(|offset| usize::try_from(offset).ok())
        .and_then(|offset| rows.get(offset).copied())
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
    state.push(Val::Num(
        number_field(state, row, "currentReactionThreshold")
            .or_else(|| number_field(state, row, "bottom"))
            .unwrap_or(0.0),
    ));
    state.push(Val::Num(
        number_field(state, row, "nextReactionThreshold")
            .or_else(|| number_field(state, row, "topValue"))
            .or_else(|| number_field(state, row, "top"))
            .unwrap_or(0.0),
    ));
    state.push(Val::Num(
        number_field(state, row, "currentStanding")
            .or_else(|| number_field(state, row, "standing"))
            .unwrap_or(0.0),
    ));

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
    let rows = faction_rows(state);
    let visible = visible_faction_rows(state, &rows);
    let selected = index
        .checked_sub(1)
        .and_then(|offset| usize::try_from(offset).ok())
        .and_then(|offset| visible.get(offset).copied());

    if let Some(row) = selected.filter(|&row| is_header(state, row)) {
        table_set_static(state, row, "isCollapsed", Val::Bool(collapsed));
    } else {
        set_all_headers_collapsed(state, &rows, collapsed);
    }
    Ok(0)
}

fn collapse_all_faction_headers(state: &mut LuaState) -> LuaResult<u32> {
    let rows = faction_rows(state);
    set_all_headers_collapsed(state, &rows, true);
    Ok(0)
}

fn expand_all_faction_headers(state: &mut LuaState) -> LuaResult<u32> {
    let rows = faction_rows(state);
    set_all_headers_collapsed(state, &rows, false);
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
    let reputation_state = reputation_state(state);
    let selected_id = number_or(state, reputation_state, "legacySelectedFactionID", 0.0);
    if selected_id == 0.0 {
        state.push(Val::Num(0.0));
        return Ok(1);
    }
    let rows = faction_rows(state);
    for (index, row) in visible_faction_rows(state, &rows).into_iter().enumerate() {
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
    let rows = faction_rows(state);
    let selected_row = index
        .checked_sub(1)
        .and_then(|offset| usize::try_from(offset).ok())
        .and_then(|offset| visible_faction_rows(state, &rows).get(offset).copied());
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
