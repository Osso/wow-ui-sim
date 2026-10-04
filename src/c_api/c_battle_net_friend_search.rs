//! 12.1.0 Battle.net friends-list search and title-friend invites over
//! `SimState.bnet_friends`; no Battle.net service is contacted.

use super::c_battle_net::string_array_from_lua_table;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_table, table_get, table_set_num, val_to_string,
};
use crate::lua_api::state_types::BnetFriend;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table_ref, "SearchFriends", search_friends)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SendTitleFriendInviteByName",
        send_title_friend_invite_by_name,
    )
}

/// `AuroraFriendsSearchInfo`: the status flags are alternatives (the friends
/// list ORs the checked filter options together); text, status, and tags must
/// all match.
#[derive(Default)]
struct FriendSearch {
    text: String,
    online: bool,
    offline: bool,
    dnd: bool,
    afk: bool,
    in_queue: bool,
    available_for_queue: bool,
    tags: Vec<String>,
}

impl FriendSearch {
    fn read(state: &mut LuaState, info: Val) -> Self {
        let mut flag = |key| matches!(table_get(state, info, key), Val::Bool(true));
        let (online, offline, dnd, afk) = (
            flag("isOnline"),
            flag("isOffline"),
            flag("isDND"),
            flag("isAFK"),
        );
        let (in_queue, available_for_queue) = (flag("isInQueue"), flag("isAvailableForQueue"));
        let text = table_get(state, info, "searchText");
        let tags = table_get(state, info, "tags");
        Self {
            text: val_to_string(state, text)
                .unwrap_or_default()
                .to_lowercase(),
            online,
            offline,
            dnd,
            afk,
            in_queue,
            available_for_queue,
            tags: string_array_from_lua_table(state, tags),
        }
    }

    fn matches(&self, friend: &BnetFriend) -> bool {
        self.matches_text(friend) && self.matches_status(friend) && self.matches_tags(friend)
    }

    fn matches_text(&self, friend: &BnetFriend) -> bool {
        if self.text.is_empty() {
            return true;
        }
        let character_names = friend
            .game_accounts
            .iter()
            .map(|account| account.character_name.as_str());
        [
            friend.account_name.as_str(),
            friend.battle_tag.as_str(),
            friend.note.as_str(),
            friend.custom_title_friend_name.as_deref().unwrap_or(""),
        ]
        .into_iter()
        .chain(character_names)
        .any(|field| field.to_lowercase().contains(&self.text))
    }

    fn matches_status(&self, friend: &BnetFriend) -> bool {
        let online = friend.game_accounts.iter().any(|account| account.is_online);
        let afk = friend.is_afk || friend.game_accounts.iter().any(|a| a.is_game_afk);
        let dnd = friend.is_dnd || friend.game_accounts.iter().any(|a| a.is_game_busy);
        // INFERRED: queue membership is unmodeled, so no friend is in a queue
        // and every online friend is available for one.
        let candidates = [
            (self.online, online),
            (self.offline, !online),
            (self.dnd, dnd),
            (self.afk, afk),
            (self.in_queue, false),
            (self.available_for_queue, online),
        ];
        let any_requested = candidates.iter().any(|(requested, _)| *requested);
        !any_requested
            || candidates
                .iter()
                .any(|(requested, holds)| *requested && *holds)
    }

    fn matches_tags(&self, friend: &BnetFriend) -> bool {
        self.tags.is_empty() || self.tags.iter().any(|tag| friend.friend_tags.contains(tag))
    }
}

fn search_friends(state: &mut LuaState) -> LuaResult<u32> {
    let search = match stack_val(state, 1) {
        info @ Val::Table(_) => FriendSearch::read(state, info),
        _ => FriendSearch::default(),
    };
    let indices: Vec<usize> = borrow_state(state)?
        .bnet_friends
        .iter()
        .enumerate()
        .filter(|(_, friend)| search.matches(friend))
        .map(|(offset, _)| offset + 1)
        .collect();
    let result = create_table(state);
    let Val::Table(array) = result else {
        unreachable!("create_table must return a table");
    };
    for (position, friend_index) in indices.into_iter().enumerate() {
        table_set_num(
            state,
            array,
            (position + 1) as f64,
            Val::Num(friend_index as f64),
        );
    }
    state.push(result);
    Ok(1)
}

fn send_title_friend_invite_by_name(state: &mut LuaState) -> LuaResult<u32> {
    let name = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    record_title_friend_request(&mut *borrow_state_mut(state)?, &name);
    Ok(0)
}

/// Records an outgoing in-game ("title") friend request once per character
/// name; acceptance is a server decision the simulator does not fabricate.
pub(crate) fn record_title_friend_request(sim: &mut crate::lua_api::state::SimState, name: &str) {
    let name = name.trim();
    let already_sent = sim
        .title_friend_requests
        .iter()
        .any(|sent| sent.eq_ignore_ascii_case(name));
    if !name.is_empty() && !already_sent {
        sim.title_friend_requests.push(name.to_string());
    }
}
