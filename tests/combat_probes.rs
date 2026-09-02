//! Integration tests for `src/lua_api/globals/real/combat_probes.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

// ── Defaults ──────────────────────────────────────────────────────────────────

#[test]
fn all_probes_false_by_default() {
    let env = env();
    let probes: (bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            return InCombatLockdown(),
                   IsEncounterInProgress(),
                   IsResting(),
                   IsFlyableArea(),
                   IsBattlefieldArena()
            "#,
        )
        .unwrap();
    assert_eq!(probes, (false, false, false, false, false));
}

#[test]
fn is_in_instance_defaults_to_false_none() {
    let env = env();
    let (flag, kind): (bool, String) = env.eval("return IsInInstance()").unwrap();
    assert!(!flag);
    assert_eq!(kind, "none");
}

// ── Read-through from SimState ───────────────────────────────────────────────

#[test]
fn in_combat_lockdown_reads_player_in_combat() {
    let env = env();
    env.state().borrow_mut().player.in_combat = true;
    let b: bool = env.eval("return InCombatLockdown()").unwrap();
    assert!(b);
}

#[test]
fn is_resting_reads_player_is_resting() {
    let env = env();
    env.state().borrow_mut().player.is_resting = true;
    let b: bool = env.eval("return IsResting()").unwrap();
    assert!(b);
}

#[test]
fn is_encounter_in_progress_reads_world_flag() {
    let env = env();
    env.state().borrow_mut().world.encounter_in_progress = true;
    let b: bool = env.eval("return IsEncounterInProgress()").unwrap();
    assert!(b);
}

#[test]
#[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
fn instance_encounter_progress_reads_the_legacy_encounter_state() {
    let env = env();
    let default_values: (bool, bool) = env
        .eval("return C_InstanceEncounter.IsEncounterInProgress(), IsEncounterInProgress()")
        .unwrap();
    assert_eq!(default_values, (false, false));

    env.state().borrow_mut().world.encounter_in_progress = true;
    let active_values: (bool, bool) = env
        .eval("return C_InstanceEncounter.IsEncounterInProgress(), IsEncounterInProgress()")
        .unwrap();
    assert_eq!(active_values, (true, true));
}

#[test]
fn is_flyable_area_reads_world_flag() {
    let env = env();
    env.state().borrow_mut().world.flyable_area = true;
    let b: bool = env.eval("return IsFlyableArea()").unwrap();
    assert!(b);
}

#[test]
fn is_battlefield_arena_reads_world_flag() {
    let env = env();
    env.state().borrow_mut().world.battlefield_arena = true;
    let b: bool = env.eval("return IsBattlefieldArena()").unwrap();
    assert!(b);
}

#[test]
fn is_in_instance_returns_instance_kind_when_active() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.world.in_instance = true;
        st.world.instance_type = "party".into();
    }
    let (flag, kind): (bool, String) = env.eval("return IsInInstance()").unwrap();
    assert!(flag);
    assert_eq!(kind, "party");
}

#[test]
fn in_combat_lockdown_tracks_combat_exit() {
    let env = env();
    env.state().borrow_mut().player.in_combat = true;
    let in_combat: bool = env.eval("return InCombatLockdown()").unwrap();
    assert!(in_combat);

    env.state().borrow_mut().player.in_combat = false;
    let after_exit: bool = env.eval("return InCombatLockdown()").unwrap();
    assert!(!after_exit);
}
