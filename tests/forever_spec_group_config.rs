//! WM99 ordinary group selection over supplied, ordered player configurations.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::forever_talents::LoadedTraitConfig;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn spec_group_config_selects_first_flagged_combat_record_independent_of_active() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_SpecializationInfo.GetCombatConfigIDForSpecGroup(1) == nil)")
        .unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.talents.active_config_id = 765;
        sim.talents.forever.loaded_configs = vec![
            LoadedTraitConfig { id: 10, is_combat: false, combat_flags: 9 },
            LoadedTraitConfig { id: 11, is_combat: true, combat_flags: 0 },
            LoadedTraitConfig { id: 42, is_combat: true, combat_flags: 1 },
            LoadedTraitConfig { id: 43, is_combat: true, combat_flags: 1 },
            LoadedTraitConfig { id: 91, is_combat: true, combat_flags: 8 },
        ];
    }
    env.exec(
        r#"
        assert(C_SpecializationInfo.GetCombatConfigIDForSpecGroup(1) == 42)
        assert(C_SpecializationInfo.GetCombatConfigIDForSpecGroup(2) == 91)
        assert(C_SpecializationInfo.GetCombatConfigIDForSpecGroup(99) == 91)
        assert(select('#', C_SpecializationInfo.GetCombatConfigIDForSpecGroup(1)) == 1)
        "#,
    ).unwrap();
    env.state().borrow_mut().talents.forever.loaded_configs[2].id = 0;
    env.exec("assert(C_SpecializationInfo.GetCombatConfigIDForSpecGroup(1) == 0)")
        .unwrap();
    env.state().borrow_mut().talents.forever.loaded_configs.clear();
    env.exec(
        r#"
        assert(C_SpecializationInfo.GetCombatConfigIDForSpecGroup(1) == nil)
        assert(C_SpecializationInfo.GetCombatConfigIDForSpecGroup(99) == nil)
        assert(select('#', C_SpecializationInfo.GetCombatConfigIDForSpecGroup(99)) == 1)
        "#,
    ).unwrap();
}

#[test]
fn spec_group_config_requires_an_index_with_native_usage_diagnostic() {
    let env = WowLuaEnv::new().unwrap();
    for call in ["C_SpecializationInfo.GetCombatConfigIDForSpecGroup()", "C_SpecializationInfo.GetCombatConfigIDForSpecGroup(nil)"] {
        let error = env.exec(call).expect_err("required group index").to_string();
        assert!(error.contains("Usage: local configID = C_SpecializationInfo.GetCombatConfigIDForSpecGroup(groupIndex)"), "{error}");
    }
}
