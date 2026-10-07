//! Retail 12.0.5 trusted addon instruction accounting and shutdown exemptions.
//! Quotas and reset periods are host policy, not native elapsed-time parity.

use super::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const DEFAULT_INSTRUCTION_LIMIT: u64 = 10_000_000;

pub(crate) fn with_addon_budget<T>(
    state: &mut LuaState,
    owner: &str,
    operation: impl FnOnce(&mut LuaState) -> LuaResult<T>,
) -> LuaResult<T> {
    if state.instruction_budget(owner).is_none() {
        state.set_instruction_budget(owner, Some(DEFAULT_INSTRUCTION_LIMIT));
    }
    state.with_instruction_owner(owner, operation)
}

pub(crate) fn with_event_exemption<T>(
    state: &mut LuaState,
    event: Option<&str>,
    operation: impl FnOnce(&mut LuaState) -> T,
) -> T {
    if matches!(event, Some("PLAYER_LOGOUT" | "ADDONS_UNLOADING")) {
        state.with_instruction_budget_exemption(operation)
    } else {
        operation(state)
    }
}

pub(crate) fn call_frame_handler(
    state: &mut LuaState,
    frame_id: u64,
    handler: Val,
    args: &[Val],
    event: Option<&str>,
) -> Result<Vec<Val>, String> {
    let owner = frame_addon_owner(state, frame_id).map_err(|error| error.to_string())?;
    with_event_exemption(state, event, |state| {
        let call = |state: &mut LuaState| {
            super::script_helpers::protected_lua_pcall_state(state, handler, args)
                .map_err(rilua::runtime_error)
        };
        let result = match owner {
            Some(owner) => with_addon_budget(state, &owner, call),
            None => call(state),
        };
        result.map_err(|error| error.to_string())
    })
}

fn frame_addon_owner(state: &LuaState, frame_id: u64) -> LuaResult<Option<String>> {
    let sim = borrow_state(state)?;
    let addon = sim
        .widgets
        .get(frame_id)
        .and_then(|frame| frame.owner_addon)
        .and_then(|index| sim.addons.get(index as usize));
    Ok(addon
        .filter(|addon| {
            addon.folder_name != "__BuiltIn"
                && !crate::blizzard_ui_sync::is_builtin_addon_folder(&addon.folder_name)
        })
        .map(|addon| addon.folder_name.clone()))
}

pub(crate) fn reset_frame_budgets(state: &mut LuaState) -> LuaResult<()> {
    let owners: Vec<String> = borrow_state(state)?
        .addons
        .iter()
        .map(|addon| addon.folder_name.clone())
        .collect();
    for owner in owners {
        if state.instruction_budget(&owner).is_some() {
            state.reset_instruction_usage(&owner)?;
        }
    }
    Ok(())
}
