//! `C_ProfSpecs.GetStateForTab` (WM87, Retail 12.1.5 build 70688).

use crate::lua_api::SimState;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const USAGE: &str = "Usage: local tabInfo = C_ProfSpecs.GetStateForTab(tabTreeID, configID)";
/// `Enum.ProfessionsSpecTabState.Locked`, the native state without a tab relation.
const TAB_STATE_LOCKED: i32 = 0;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail
        || !cfg!(feature = "retail-12-1-5")
    {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_ProfSpecs")?;
    table_set_rust_fn_static(state, namespace, "GetStateForTab", get_state_for_tab)
}

fn bad_argument(index: i32, reason: &str) -> rilua::LuaError {
    runtime_error(format!(
        "bad argument #{index} to 'GetStateForTab' ({reason})"
    ))
}

/// Shared signed 32-bit converter: inclusive bounds, then truncation.
fn read_id_argument(state: &LuaState, index: i32) -> LuaResult<i32> {
    let original = stack_val(state, index);
    if matches!(original, Val::Nil) {
        return Err(bad_argument(index, USAGE));
    }
    let marked = rilua::table_security::is_secret_value(state, original);
    let value = rilua::table_security::unwrap_secret(state, original).map_err(|_| {
        let denied = format!(
            "{USAGE}. Secret values are only allowed during untainted execution for this argument."
        );
        bad_argument(index, &denied)
    })?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) if !marked => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| super::helpers::parse_numeric_string(string.data())),
        _ => None,
    };
    let in_range = |number: &f64| (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(number);
    number
        .filter(in_range)
        .map(|number| number.trunc() as i32)
        .ok_or_else(|| {
            let range = format!(
                "outside of expected range {} to {} - {USAGE}",
                i32::MIN,
                i32::MAX
            );
            bad_argument(index, &range)
        })
}

/// UNKNOWN (WM87): the simulator has no trait-configuration identity, so the
/// validated configID selects nothing; supplied tab states stand for the
/// player's profession configuration. An unsupplied tree is native Locked.
fn supplied_tab_state(sim: &SimState, tree_id: i32) -> i32 {
    u32::try_from(tree_id)
        .ok()
        .and_then(|tree| sim.profession_spec_inputs.tab_states.get(&tree))
        .copied()
        .unwrap_or(TAB_STATE_LOCKED)
}

fn get_state_for_tab(state: &mut LuaState) -> LuaResult<u32> {
    let tree_id = read_id_argument(state, 1)?;
    read_id_argument(state, 2)?;
    let tab_state = supplied_tab_state(&*borrow_state(state)?, tree_id);
    state.push(Val::Num(f64::from(tab_state)));
    Ok(1)
}
