//! `UnitIsPlayer` + related unit-identity probes, resolved against SimState.
//!
//! Real WoW resolves unit tokens (`"player"`, `"target"`, `"partyN"`, ...)
//! through a unit map that answers "who is this token pointing at right now?".
//! The sim doesn't have a combat/grouping engine, but it does have enough
//! state to answer the common token set correctly — player character always
//! exists, party members are modelled in `SimState.party_members`, and the
//! current target/focus carry an `is_player` flag.
//!
//! Tokens the sim resolves:
//!
//! | Token                  | Resolution                                           |
//! |------------------------|------------------------------------------------------|
//! | `player` / `self`      | always true (sim has a player character)             |
//! | `target`               | `sim.current_target.is_player`                       |
//! | `focus`                | `sim.current_focus.is_player`                        |
//! | `partyN` (N=1..4)      | true iff party slot N is populated (PartyMembers are |
//! |                        | player-characters by definition)                     |
//! | `raid1`..`raid40`      | not supported yet (sim has no raid roster) → false   |
//! | `mouseover` / `pet` /  | no sim state → false                                 |
//! | `npcN`, etc.           |                                                      |
//!
//! Caller-visible signatures:
//!
//! - `UnitIsPlayer(unit: string) -> boolean`
//! - `UnitIsHumanPlayer(unit: string) -> boolean`
//!
//! Returns `false` for non-string args (matches WoW's "nil → false"
//! behaviour). The sim does not model non-human player controllers yet, so
//! `UnitIsHumanPlayer` follows the same token resolution as `UnitIsPlayer`.

use crate::lua_api::methods::borrow_state;
use crate::lua_api::state::SimState;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub fn unit_is_player(state: &mut LuaState) -> LuaResult<u32> {
    push_unit_is_player(state)
}

pub fn unit_is_human_player(state: &mut LuaState) -> LuaResult<u32> {
    push_unit_is_player(state)
}

fn push_unit_is_player(state: &mut LuaState) -> LuaResult<u32> {
    let unit = match crate::lua_bridge::stack_val(state, 1) {
        Val::Str(s) => {
            let arena = &state.gc.string_arena;
            arena
                .get(s)
                .and_then(|lua_str| std::str::from_utf8(lua_str.data()).ok())
                .map(str::to_owned)
        }
        _ => None,
    };
    let is_player = match unit {
        Some(token) => {
            let sim = borrow_state(state)?;
            resolve_unit_is_player(&sim, &token)
        }
        None => false,
    };
    state.push(Val::Bool(is_player));
    Ok(1)
}

pub(crate) fn resolve_unit_is_player(sim: &SimState, token: &str) -> bool {
    if token.eq_ignore_ascii_case("player") || token.eq_ignore_ascii_case("self") {
        return true;
    }
    if token.eq_ignore_ascii_case("target") {
        return sim.current_target.as_ref().is_some_and(|t| t.is_player);
    }
    if token.eq_ignore_ascii_case("focus") {
        return sim.current_focus.as_ref().is_some_and(|t| t.is_player);
    }
    if let Some(idx) = party_slot_index(token) {
        // PartyMember entries are always player-characters in the sim.
        return sim.party_members.get(idx).is_some();
    }
    // raidN, mouseover, pet, npcN, arbitrary tokens → not a player.
    false
}

/// Parse `"party1"`..`"party4"` and return the 0-based index, or `None`.
/// Accepts any ASCII case.
fn party_slot_index(token: &str) -> Option<usize> {
    let token = token.as_bytes();
    if token.len() != 6 || !token[..5].eq_ignore_ascii_case(b"party") {
        return None;
    }
    match token[5] {
        b'1' => Some(0),
        b'2' => Some(1),
        b'3' => Some(2),
        b'4' => Some(3),
        _ => None,
    }
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    use rilua::LuaApiMut;
    let state = lua.state_mut();
    table_set_rust_fn_static(state, state.global, "UnitIsPlayer", unit_is_player)?;
    table_set_rust_fn_static(
        state,
        state.global,
        "UnitIsHumanPlayer",
        unit_is_human_player,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::party_slot_index;

    #[test]
    fn party_slot_index_matches_four_slots() {
        assert_eq!(party_slot_index("party1"), Some(0));
        assert_eq!(party_slot_index("party2"), Some(1));
        assert_eq!(party_slot_index("party3"), Some(2));
        assert_eq!(party_slot_index("party4"), Some(3));
    }

    #[test]
    fn party_slot_index_rejects_out_of_range_and_wrong_shape() {
        assert_eq!(party_slot_index("party0"), None);
        assert_eq!(party_slot_index("party5"), None);
        assert_eq!(party_slot_index("party10"), None);
        assert_eq!(party_slot_index("partypet"), None);
        assert_eq!(party_slot_index("raid1"), None);
        assert_eq!(party_slot_index(""), None);
    }

    #[test]
    fn party_slot_index_is_case_insensitive() {
        assert_eq!(party_slot_index("Party1"), Some(0));
        assert_eq!(party_slot_index("PARTY2"), Some(1));
    }
}
