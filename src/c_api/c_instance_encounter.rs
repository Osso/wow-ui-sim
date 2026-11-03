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
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }

    let namespace = ensure_namespace(state, "C_InstanceEncounter")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsEncounterInProgress",
        is_encounter_in_progress,
    )?;
    if cfg!(feature = "retail-12-0-0") || ACTIVE == ClientProfile::WowForever {
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

#[cfg(all(test, feature = "client-wowforever"))]
mod tests {
    use crate::loader::{discover_blizzard_addons_for_screen, load_addon};
    use crate::lua_api::WowLuaEnv;
    use crate::screen::ScreenKind;

    #[test]
    fn forever_deprecated_encounter_queries_read_live_state() {
        let addons = crate::paths::default_blizzard_ui_addons_path().unwrap();
        let (_, toc) = discover_blizzard_addons_for_screen(&addons, ScreenKind::Game)
            .into_iter()
            .find(|(name, _)| name == "Blizzard_DeprecatedInstanceEncounter")
            .expect("Forever should select the encounter compatibility addon in game");
        let env = WowLuaEnv::new().unwrap();
        load_addon(&env.loader_env(), &toc).unwrap();
        env.exec(
            "assert(debug.getinfo(IsEncounterInProgress).source:find('Deprecated_InstanceEncounter.lua', 1, true))",
        )
        .unwrap();

        let query = "return IsEncounterInProgress(), IsEncounterLimitingResurrections(), \
                     IsEncounterSuppressingRelease(), C_InstanceEncounter.ShouldShowTimelineForEncounter()";
        let read = || env.eval::<(bool, bool, bool, bool)>(query).unwrap();
        assert_eq!(read(), (false, false, false, false));
        {
            let mut state = env.state().borrow_mut();
            state.world.encounter_in_progress = true;
            state.encounter_policy.limiting_resurrections = true;
            state.encounter_policy.show_timeline = true;
        }
        assert_eq!(read(), (true, true, false, true));
        {
            let mut state = env.state().borrow_mut();
            state.world.encounter_in_progress = false;
            state.encounter_policy.limiting_resurrections = false;
            state.encounter_policy.suppressing_release = true;
            state.encounter_policy.show_timeline = false;
        }
        assert_eq!(read(), (false, false, true, false));
    }
}
