use super::ensure_namespace;
use super::item_spell::parse_prefixed_id;
use crate::lua_api::methods::{borrow_state, create_table, table_get, table_set, val_to_string};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const SOCKET_NAMESPACE: &str = "C_ItemSocketInfo";
const SOCKET_STATE_KEY: &str = "_state";

type SocketFn = fn(&mut LuaState) -> LuaResult<u32>;

const SOCKET_METHODS: &[(&'static str, SocketFn)] = &[
    ("AcceptSockets", c_item_socket_info_accept_sockets),
    ("ClickSocketButton", c_item_socket_info_click_socket_button),
    ("CloseSocketInfo", c_item_socket_info_close_socket_info),
    ("CompleteSocketing", c_item_socket_info_complete_socketing),
    ("GetCurrUIType", c_item_socket_info_get_curr_ui_type),
    (
        "GetExistingSocketInfo",
        c_item_socket_info_get_existing_socket_info,
    ),
    (
        "GetExistingSocketLink",
        c_item_socket_info_get_existing_socket_link,
    ),
    ("GetNewSocketInfo", c_item_socket_info_get_new_socket_info),
    ("GetNewSocketLink", c_item_socket_info_get_new_socket_link),
    ("GetNumSockets", c_item_socket_info_get_num_sockets),
    (
        "GetSocketItemBoundTradeable",
        c_item_socket_info_get_socket_item_bound_tradeable,
    ),
    ("GetSocketItemInfo", c_item_socket_info_get_socket_item_info),
    (
        "GetSocketItemRefundable",
        c_item_socket_info_get_socket_item_refundable,
    ),
    ("GetSocketTypes", c_item_socket_info_get_socket_types),
    (
        "HasBoundGemProposed",
        c_item_socket_info_has_bound_gem_proposed,
    ),
    (
        "IsArtifactRelicItem",
        c_item_socket_info_is_artifact_relic_item,
    ),
];

pub(super) fn register_item_socket_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, SOCKET_NAMESPACE)?;
    ensure_socket_state_table(state);
    for &(name, func) in SOCKET_METHODS {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    #[cfg(not(feature = "retail-12-0-0"))]
    table_set_rust_fn_static(
        state,
        state.global,
        "IsArtifactRelicItem",
        c_item_socket_info_is_artifact_relic_item,
    )?;
    Ok(())
}

pub(super) fn socketed_item_id(state: &mut LuaState) -> Option<u32> {
    let socket_state = ensure_socket_state_table(state);
    let item_info = table_get(state, socket_state, "itemInfo");
    let item_id = table_get(state, item_info, "itemID");
    let link = table_get(state, item_info, "link");
    item_id_from_val(state, item_id).or_else(|| item_id_from_val(state, link))
}

pub(super) fn new_socket_item_id(state: &mut LuaState, index: i32) -> Option<u32> {
    socket_item_id_from_bucket(state, "newSockets", index)
}

pub(super) fn existing_socket_item_id(state: &mut LuaState, index: i32) -> Option<u32> {
    socket_item_id_from_bucket(state, "existingSockets", index)
}

fn socket_item_id_from_bucket(state: &mut LuaState, bucket: &str, index: i32) -> Option<u32> {
    let socket_state = ensure_socket_state_table(state);
    let sockets = table_get(state, socket_state, bucket);
    let socket = table_array_get(state, sockets, index);
    let item_id = table_get(state, socket, "itemID");
    let link = table_get(state, socket, "link");
    item_id_from_val(state, item_id).or_else(|| item_id_from_val(state, link))
}

fn ensure_socket_state_table(state: &mut LuaState) -> Val {
    let namespace = Val::Table(
        ensure_namespace(state, SOCKET_NAMESPACE).expect("C_ItemSocketInfo namespace should exist"),
    );
    let socket_state = match table_get(state, namespace, SOCKET_STATE_KEY) {
        table @ Val::Table(_) => table,
        _ => {
            let table = create_table(state);
            table_set(state, namespace, SOCKET_STATE_KEY, table);
            table
        }
    };
    ensure_socket_subtable(state, socket_state, "artifactRelicItemIDs");
    ensure_socket_subtable(state, socket_state, "clickProposals");
    ensure_socket_subtable(state, socket_state, "existingSockets");
    ensure_socket_subtable(state, socket_state, "newSockets");
    ensure_socket_subtable(state, socket_state, "socketTypes");
    socket_state
}

fn ensure_socket_subtable(state: &mut LuaState, socket_state: Val, key: &str) {
    if matches!(table_get(state, socket_state, key), Val::Table(_)) {
        return;
    }
    let table = create_table(state);
    table_set(state, socket_state, key, table);
}

fn table_array_get(state: &LuaState, table: Val, index: i32) -> Val {
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.get(Val::Num(index as f64), &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

fn table_array_set(state: &mut LuaState, table: Val, index: i32, value: Val) {
    let Val::Table(table_ref) = table else { return };
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(Val::Num(index as f64), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
}

fn table_array_has_true(state: &LuaState, table: Val, index: i32) -> bool {
    matches!(table_array_get(state, table, index), Val::Bool(true))
}

fn item_id_from_val(state: &mut LuaState, value: Val) -> Option<u32> {
    match value {
        Val::Num(number) if number > 0.0 => Some(number as u32),
        Val::Str(_) => val_to_string(state, value).and_then(|text| parse_item_id_text(&text)),
        Val::Table(_) => {
            let item_id = table_get(state, value, "itemID");
            let link = table_get(state, value, "link");
            item_id_from_val(state, item_id).or_else(|| item_id_from_val(state, link))
        }
        _ => None,
    }
}

fn parse_item_id_text(text: &str) -> Option<u32> {
    parse_prefixed_id(text, "item")
        .or_else(|| {
            text.strip_prefix("item:").and_then(|tail| {
                tail.split(':')
                    .next()
                    .and_then(|digits| digits.parse::<u32>().ok())
            })
        })
        .or_else(|| text.parse::<u32>().ok())
}

fn bump_counter(state: &mut LuaState, socket_state: Val, key: &str) {
    let next = match table_get(state, socket_state, key) {
        Val::Num(value) => value + 1.0,
        _ => 1.0,
    };
    table_set(state, socket_state, key, Val::Num(next));
}

fn accept_sockets(state: &mut LuaState) -> bool {
    let socket_state = ensure_socket_state_table(state);
    let existing_sockets = table_get(state, socket_state, "existingSockets");
    let new_sockets = table_get(state, socket_state, "newSockets");
    let num_sockets = match table_get(state, socket_state, "numSockets") {
        Val::Num(value) if value > 0.0 => value as i32,
        _ => 0,
    };

    for index in 1..=num_sockets {
        let pending = table_array_get(state, new_sockets, index);
        if matches!(pending, Val::Table(_)) {
            table_array_set(state, existing_sockets, index, pending);
        }
    }

    let empty_new_sockets = create_table(state);
    table_set(state, socket_state, "newSockets", empty_new_sockets);
    table_set(state, socket_state, "hasBoundGemProposed", Val::Bool(false));
    bump_counter(state, socket_state, "acceptCount");
    true
}

fn close_socket_state(state: &mut LuaState) -> bool {
    let socket_state = ensure_socket_state_table(state);
    let was_open = matches!(table_get(state, socket_state, "isOpen"), Val::Bool(true));
    table_set(state, socket_state, "isOpen", Val::Bool(false));
    let empty_new_sockets = create_table(state);
    table_set(state, socket_state, "newSockets", empty_new_sockets);
    table_set(state, socket_state, "hasBoundGemProposed", Val::Bool(false));
    bump_counter(state, socket_state, "closeCount");
    was_open
}

fn push_socket_info_triplet(state: &mut LuaState, socket: Val) {
    let name = table_get(state, socket, "name");
    let icon = table_get(state, socket, "icon");
    let matches_socket = table_get(state, socket, "gemMatchesSocket");
    state.push(name);
    state.push(icon);
    state.push(matches_socket);
}

fn c_item_socket_info_accept_sockets(state: &mut LuaState) -> LuaResult<u32> {
    let accepted = accept_sockets(state);
    state.push(Val::Bool(accepted));
    Ok(1)
}

fn c_item_socket_info_click_socket_button(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let socket_state = ensure_socket_state_table(state);
    let num_sockets = match table_get(state, socket_state, "numSockets") {
        Val::Num(value) if value > 0.0 => value as i32,
        _ => 0,
    };
    if index <= 0 || (num_sockets > 0 && index > num_sockets) {
        state.push(Val::Bool(false));
        return Ok(1);
    }

    table_set(
        state,
        socket_state,
        "selectedSocketIndex",
        Val::Num(index as f64),
    );

    let proposals = table_get(state, socket_state, "clickProposals");
    let proposal = table_array_get(state, proposals, index);
    if matches!(proposal, Val::Table(_)) {
        let new_sockets = table_get(state, socket_state, "newSockets");
        table_array_set(state, new_sockets, index, proposal);
        if matches!(table_get(state, proposal, "isBound"), Val::Bool(true)) {
            table_set(state, socket_state, "hasBoundGemProposed", Val::Bool(true));
        }
    }

    state.push(Val::Bool(true));
    Ok(1)
}

fn c_item_socket_info_close_socket_info(state: &mut LuaState) -> LuaResult<u32> {
    let was_open = close_socket_state(state);
    state.push(Val::Bool(was_open));
    Ok(1)
}

fn c_item_socket_info_complete_socketing(state: &mut LuaState) -> LuaResult<u32> {
    accept_sockets(state);
    Ok(0)
}

fn c_item_socket_info_get_curr_ui_type(state: &mut LuaState) -> LuaResult<u32> {
    let socket_state = ensure_socket_state_table(state);
    match table_get(state, socket_state, "uiType") {
        value @ Val::Num(_) => state.push(value),
        _ => state.push(Val::Num(0.0)),
    }
    Ok(1)
}

fn c_item_socket_info_get_existing_socket_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let socket_state = ensure_socket_state_table(state);
    let sockets = table_get(state, socket_state, "existingSockets");
    push_socket_info_triplet(state, table_array_get(state, sockets, index));
    Ok(3)
}

fn c_item_socket_info_get_existing_socket_link(state: &mut LuaState) -> LuaResult<u32> {
    push_socket_link(state, "existingSockets")
}

fn c_item_socket_info_get_new_socket_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let socket_state = ensure_socket_state_table(state);
    let sockets = table_get(state, socket_state, "newSockets");
    push_socket_info_triplet(state, table_array_get(state, sockets, index));
    Ok(3)
}

fn c_item_socket_info_get_new_socket_link(state: &mut LuaState) -> LuaResult<u32> {
    push_socket_link(state, "newSockets")
}

fn push_socket_link(state: &mut LuaState, bucket: &str) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let socket_state = ensure_socket_state_table(state);
    let sockets = table_get(state, socket_state, bucket);
    let socket = table_array_get(state, sockets, index);
    let link = table_get(state, socket, "link");
    state.push(link);
    Ok(1)
}

fn c_item_socket_info_get_num_sockets(state: &mut LuaState) -> LuaResult<u32> {
    let socket_state = ensure_socket_state_table(state);
    match table_get(state, socket_state, "numSockets") {
        value @ Val::Num(_) => state.push(value),
        _ => state.push(Val::Num(0.0)),
    }
    Ok(1)
}

fn c_item_socket_info_get_socket_item_bound_tradeable(state: &mut LuaState) -> LuaResult<u32> {
    push_socket_item_bool(state, "isBoundTradeable")
}

fn c_item_socket_info_get_socket_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let socket_state = ensure_socket_state_table(state);
    let item_info = table_get(state, socket_state, "itemInfo");
    let name = table_get(state, item_info, "name");
    let icon = table_get(state, item_info, "icon");
    let quality = table_get(state, item_info, "quality");
    state.push(name);
    state.push(icon);
    state.push(quality);
    Ok(3)
}

fn c_item_socket_info_get_socket_item_refundable(state: &mut LuaState) -> LuaResult<u32> {
    push_socket_item_bool(state, "isRefundable")
}

fn push_socket_item_bool(state: &mut LuaState, key: &str) -> LuaResult<u32> {
    let socket_state = ensure_socket_state_table(state);
    let item_info = table_get(state, socket_state, "itemInfo");
    match table_get(state, item_info, key) {
        value @ Val::Bool(_) => state.push(value),
        _ => state.push(Val::Bool(false)),
    }
    Ok(1)
}

fn c_item_socket_info_get_socket_types(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let socket_state = ensure_socket_state_table(state);
    let socket_types = table_get(state, socket_state, "socketTypes");
    state.push(table_array_get(state, socket_types, index));
    Ok(1)
}

fn c_item_socket_info_has_bound_gem_proposed(state: &mut LuaState) -> LuaResult<u32> {
    let socket_state = ensure_socket_state_table(state);
    match table_get(state, socket_state, "hasBoundGemProposed") {
        value @ Val::Bool(_) => state.push(value),
        _ => state.push(Val::Bool(false)),
    }
    Ok(1)
}

/// `IsArtifactRelicItem(item)` returns true when `item` resolves to an
/// id present in the simulator-side `state.artifact_relic_items` set.
/// The legacy Lua-side `socketState.artifactRelicItemIDs` table is also
/// consulted as a fallback so existing tests and addon code that seed
/// the namespace's `_state` directly continue to work.
fn c_item_socket_info_is_artifact_relic_item(state: &mut LuaState) -> LuaResult<u32> {
    let argument = stack_val(state, 1);
    let item_id = item_id_from_val(state, argument);
    let is_relic = match item_id {
        Some(id) => artifact_relic_lookup(state, id),
        None => false,
    };
    state.push(Val::Bool(is_relic));
    Ok(1)
}

fn artifact_relic_lookup(state: &mut LuaState, item_id: u32) -> bool {
    let known_in_state = borrow_state(state)
        .map(|st| st.artifact_relic_items.contains(&(item_id as i32)))
        .unwrap_or(false);
    if known_in_state {
        return true;
    }
    let socket_state = ensure_socket_state_table(state);
    let relic_ids = table_get(state, socket_state, "artifactRelicItemIDs");
    table_array_has_true(state, relic_ids, item_id as i32)
}
