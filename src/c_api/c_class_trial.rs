//! Supplied class-trial character state for Retail and WoW Forever consumers.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// INFERRED (R88en (native evidence pending)): timer values use the GetTime clock.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct ClassTrialState {
    pub is_trial_character: bool,
    pub logout_deadline: f64,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_ClassTrial")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsClassTrialCharacter",
        is_trial_character,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetClassTrialLogoutTimeSeconds",
        get_logout_deadline,
    )?;
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetClassTrialState", set_class_trial_state)
}

fn set_class_trial_state(state: &mut LuaState) -> LuaResult<u32> {
    let is_trial_character = bool::from_stack(state, 1)?;
    let logout_deadline = f64::from_stack(state, 2)?;
    if !logout_deadline.is_finite() || logout_deadline < 0.0 {
        return Err(runtime_error(
            "A_Admin.SetClassTrialState: deadline must be finite and nonnegative",
        ));
    }
    borrow_state_mut(state)?.class_trial = ClassTrialState {
        is_trial_character,
        logout_deadline,
    };
    Ok(0)
}

fn is_trial_character(state: &mut LuaState) -> LuaResult<u32> {
    let is_trial = borrow_state(state)?.class_trial.is_trial_character;
    state.push(Val::Bool(is_trial));
    Ok(1)
}

fn get_logout_deadline(state: &mut LuaState) -> LuaResult<u32> {
    let deadline = borrow_state(state)?.class_trial.logout_deadline;
    state.push(Val::Num(deadline));
    Ok(1)
}
