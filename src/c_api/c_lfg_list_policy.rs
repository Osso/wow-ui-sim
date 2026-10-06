//! Explicit host endgame-edit policy, without invented native eligibility thresholds.
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// INFERRED: unset policy denies eligibility and marks no restricted activities.
#[derive(Debug, Default)]
pub struct EndgameEditPolicy {
    pub player_valid: bool,
    pub restricted_activities: std::collections::HashSet<u32>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_LFGList")?;
    table_set_rust_fn_static(state, ns, "IsPlayerValidForEndgameFieldEdits", |s| {
        let valid = borrow_state(s)?.lfg_endgame_policy.player_valid;
        s.push(Val::Bool(valid));
        Ok(1)
    })?;
    table_set_rust_fn_static(state, ns, "ListingUsesEndgameEditRestrictions", |s| {
        let activity = u32::from_stack(s, 1)?;
        let restricted = borrow_state(s)?
            .lfg_endgame_policy
            .restricted_activities
            .contains(&activity);
        s.push(Val::Bool(restricted));
        Ok(1)
    })
}
