//! Who query routing shared by the legacy and C_FriendList adapters.
//! Explicit response input, not a server, query parser, or native timing model.
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Debug, Default)]
pub struct WhoRouting {
    pub to_ui: bool,
    pub last_query: Option<String>,
    pub pending: Vec<String>,
    pub ui_results: Vec<String>,
    /// Last `C_FriendList.SortWho` order; `None` keeps the response order.
    pub sort: Option<WhoSort>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhoSort {
    pub key: String,
    pub ascending: bool,
}

pub(crate) fn send_who(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    let query = String::from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    sim.who_routing.last_query = Some(query);
    let response = sim.who_routing.pending.clone();
    if sim.who_routing.to_ui {
        sim.who_routing.ui_results = response;
        sim.events.push_simple("WHO_LIST_UPDATE");
    } else {
        sim.system_chat_log.extend(response);
    }
    Ok(0)
}

pub(crate) fn register_c_friend_list(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::WowForever {
        return Ok(());
    }
    // Forever 70707 FriendListDocumentation.lua:330-343 and :411-421.
    crate::lua_bridge::table_set_rust_fn_static(state, namespace, "SendWho", c_send_who)?;
    crate::lua_bridge::table_set_rust_fn_static(state, namespace, "SortWho", sort_who)
}

/// `C_FriendList.SendWho(filter, origin?, filters?)`: the optional origin and
/// `SendWhoFilters` are validated; the query routes like the legacy global.
fn c_send_who(state: &mut LuaState) -> LuaResult<u32> {
    if !matches!(stack_val(state, 2), Val::Nil | Val::Num(_)) {
        return Err(runtime_error(
            "C_FriendList.SendWho: origin must be a number",
        ));
    }
    if !matches!(stack_val(state, 3), Val::Nil | Val::Table(_)) {
        return Err(runtime_error(
            "C_FriendList.SendWho: filters must be a SendWhoFilters table",
        ));
    }
    send_who(state)
}

/// `C_FriendList.SortWho(sorting, ascending?)` reorders the current results
/// and queues `WHO_LIST_UPDATE`. INFERRED from the Retail column header
/// (`FriendsFrame.lua:2729-2733`), which sorts without refreshing itself.
/// INFERRED: without `ascending`, repeating the current key reverses the
/// order and a new key sorts ascending.
fn sort_who(state: &mut LuaState) -> LuaResult<u32> {
    let key = String::from_stack(state, 1)?;
    let ascending = match stack_val(state, 2) {
        Val::Nil => None,
        Val::Bool(flag) => Some(flag),
        _ => {
            return Err(runtime_error(
                "C_FriendList.SortWho: ascending must be a boolean",
            ));
        }
    };
    let mut sim = borrow_state_mut(state)?;
    let routing = &mut sim.who_routing;
    let ascending = ascending.unwrap_or_else(
        || !matches!(&routing.sort, Some(sort) if sort.key == key && sort.ascending),
    );
    routing.sort = Some(WhoSort { key, ascending });
    sim.events.push_simple("WHO_LIST_UPDATE");
    Ok(0)
}
