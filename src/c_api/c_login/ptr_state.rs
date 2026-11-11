//! PTR70668 GetState projection from R44-Q17 (lt-a667 ANSWER 51).

use crate::lua_api::methods::{borrow_state, create_table, table_set};
use crate::screen::ScreenKind;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Resolved host connection inputs. Product gates are supplied independently;
/// false defaults describe an empty host scenario, not product configuration.
#[derive(Clone, Debug)]
pub struct PtrConnectionState {
    pub aurora_state: i32,
    pub connected_to_wow: bool,
    pub connection_object_needs_connect: bool,
    pub pending_connect: bool,
    pub wow_queue: bool,
    pub realm_list_present: bool,
    pub realm_list_available: bool,
    pub waiting_for_realm_list: bool,
    pub rule_sets_enabled: bool,
    pub waiting_for_rule_sets: bool,
    pub super_district_enabled: bool,
    pub super_district_choice_pending: bool,
    pub keeping_character_list_on_realm_join: bool,
}

impl Default for PtrConnectionState {
    fn default() -> Self {
        Self {
            aurora_state: 1,
            connected_to_wow: false,
            connection_object_needs_connect: false,
            pending_connect: false,
            wow_queue: false,
            realm_list_present: false,
            realm_list_available: false,
            waiting_for_realm_list: false,
            rule_sets_enabled: false,
            waiting_for_rule_sets: false,
            super_district_enabled: false,
            super_district_choice_pending: false,
            keeping_character_list_on_realm_join: false,
        }
    }
}

impl PtrConnectionState {
    /// Explicit simulator boot fixture, not a native connection/reset callback.
    pub(crate) fn for_screen(screen: ScreenKind) -> Self {
        Self {
            connected_to_wow: screen != ScreenKind::Login,
            ..Self::default()
        }
    }
}

pub(super) fn get_state(state: &mut LuaState) -> LuaResult<u32> {
    let input = borrow_state(state)?.glue_login.ptr_connection.clone();
    let super_district = input.super_district_enabled && input.super_district_choice_pending;
    let connection = if input.connection_object_needs_connect || input.pending_connect {
        2
    } else if input.wow_queue {
        3
    } else {
        1
    };
    let result = create_table(state);
    for (key, value) in [
        (
            "auroraState",
            Val::Num(f64::from(if input.connected_to_wow {
                1
            } else {
                input.aurora_state
            })),
        ),
        ("connectedToWoW", Val::Bool(input.connected_to_wow)),
        ("wowConnectionState", Val::Num(f64::from(connection))),
        (
            "hasRealmList",
            Val::Bool(input.realm_list_present && input.realm_list_available),
        ),
        (
            "waitingForRealmList",
            Val::Bool(input.waiting_for_realm_list && !super_district),
        ),
        (
            "waitingForRuleSets",
            Val::Bool(input.rule_sets_enabled && input.waiting_for_rule_sets),
        ),
        ("superDistrictChoicePending", Val::Bool(super_district)),
        (
            "keepingCharacterListOnRealmJoin",
            Val::Bool(input.keeping_character_list_on_realm_join),
        ),
        // Verified absent-setting fallback. Setting identity/coercion awaits R47.
        ("hasSelectedExperiencePreset", Val::Bool(true)),
    ] {
        table_set(state, result, key, value);
    }
    state.push(result);
    Ok(1)
}
