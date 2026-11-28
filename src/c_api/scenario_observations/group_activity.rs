use super::project;
use super::{borrow_state_mut, read_input, register_methods};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct GroupActivity {
    invitations_allowed: bool,
    delve_complete: bool,
}

pub(super) fn supply(state: &mut LuaState) -> LuaResult<u32> {
    let observation = read_input(state)?;
    borrow_state_mut(state)?
        .scenario_observations
        .group_activity = observation;
    Ok(0)
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_PartyInfo",
        &[
            // PartyInfoDocumentation.lua:35
            ("CanInvite", invitations_allowed),
            // PartyInfoDocumentation.lua:402
            ("IsDelveComplete", delve_complete),
        ],
    )
}

fn invitations_allowed(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| Val::Bool(o.group_activity.invitations_allowed))
}

fn delve_complete(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| Val::Bool(o.group_activity.delve_complete))
}
