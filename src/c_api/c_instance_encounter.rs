//! `C_InstanceEncounter` state-backed encounter probes.

use crate::c_api::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// INFERRED host-owned encounter policy snapshot; no encounter mechanics inferred.
#[derive(Debug, Default)]
pub struct EncounterPolicy {
    pub limiting_resurrections: bool,
    pub suppressing_release: bool,
    pub show_timeline: bool,
}

pub(crate) fn register_c_instance_encounter_surface(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        return Ok(());
    }

    let namespace = ensure_namespace(state, "C_InstanceEncounter")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsEncounterInProgress",
        is_encounter_in_progress,
    )?;
    #[cfg(feature = "retail-12-0-0")]
    {
        table_set_rust_fn_static(state, namespace, "IsEncounterLimitingResurrections", |s| {
            let value = borrow_state(s)?.encounter_policy.limiting_resurrections;
            s.push(Val::Bool(value));
            Ok(1)
        })?;
        table_set_rust_fn_static(state, namespace, "IsEncounterSuppressingRelease", |s| {
            let value = borrow_state(s)?.encounter_policy.suppressing_release;
            s.push(Val::Bool(value));
            Ok(1)
        })?;
        table_set_rust_fn_static(state, namespace, "ShouldShowTimelineForEncounter", |s| {
            let value = borrow_state(s)?.encounter_policy.show_timeline;
            s.push(Val::Bool(value));
            Ok(1)
        })?;
    }
    Ok(())
}

pub(crate) fn is_encounter_in_progress(state: &mut LuaState) -> LuaResult<u32> {
    let encounter_in_progress = borrow_state(state)?.world.encounter_in_progress;
    state.push(Val::Bool(encounter_in_progress));
    Ok(1)
}
