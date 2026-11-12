//! PTR action-slot spell-label restrictions.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const FUNCTION: &str = "SpellMatchesActionSpellTypeRestriction";
const USAGE: &str = "Usage: local actionMatchesSpellTypeRestriction = C_ActionBar.SpellMatchesActionSpellTypeRestriction(spellID, actionID)";
const VALID_ACTION_SLOT_COUNT: i32 = 408;
const ACTIONS_PER_RESTRICTION_GROUP: u32 = 12;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Ptr {
        return Ok(());
    }

    let namespace = super::helpers::ensure_namespace(state, "C_ActionBar")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SpellMatchesActionSpellTypeRestriction",
        spell_matches_action_spell_type_restriction,
    )
}

fn spell_matches_action_spell_type_restriction(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = super::helpers::read_untainted_signed32_argument(state, 1, FUNCTION, USAGE)?;
    let action_index =
        super::action_button_hover::read_converted_slot_at(state, 2, FUNCTION, USAGE)?;
    if !(0..VALID_ACTION_SLOT_COUNT).contains(&action_index) {
        return Ok(0);
    }

    let matches = {
        let sim = borrow_state(state)?;
        matches_restriction(&sim, spell_id, action_index as u32)
    };
    state.push(Val::Bool(matches));
    Ok(1)
}

fn matches_restriction(sim: &crate::lua_api::SimState, spell_id: i32, action_slot: u32) -> bool {
    let restrictions = &sim.action_bar_spell_type_restrictions;
    if restrictions.is_empty() {
        return true;
    }
    if spell_id == 0 {
        return false;
    }

    let group = action_slot / ACTIONS_PER_RESTRICTION_GROUP;
    let Some(required_label) = restrictions.get(&group).copied() else {
        return true;
    };
    if required_label == 0 {
        return true;
    }

    sim.spell_type_labels
        .get(&spell_id)
        .is_some_and(|labels| labels.contains(&required_label))
}
