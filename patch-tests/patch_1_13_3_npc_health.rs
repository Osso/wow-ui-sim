//! Existing current Era NPC-health subset, not native 11303 parity/signatures.
use serde_json::json;
use wow_ui_sim::client_profile::{ACTIVE, ACTIVE_INTERFACE_VERSION, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::TargetInfo;

fn npc(health: i32, health_max: i32) -> TargetInfo {
    TargetInfo {
        unit_id: "target".into(),
        name: "P1133 Health Fixture".into(),
        class_index: 1,
        level: 60,
        health,
        health_max,
        power: 0,
        power_max: 0,
        power_type: 0,
        power_type_name: "MANA".into(),
        is_player: false,
        is_enemy: true,
        guid: "Creature-0-0-0-0-1133-1".into(),
        classification: "normal".into(),
        creature_type: "Humanoid".into(),
        reaction: 1,
        interaction: Default::default(),
    }
}

#[test]
fn current_era_npc_health_reads_values_across_state_changes() {
    assert_eq!(ACTIVE, ClientProfile::Era);
    assert_eq!(ACTIVE_INTERFACE_VERSION, 11507);
    let env = WowLuaEnv::new().expect("own current bare Era env; no Blizzard load/cache");
    let raw_types: (String, String) = env
        .eval("return type(rawget(_G, 'UnitHealth')), type(rawget(_G, 'UnitHealthMax'))")
        .expect("registered functions, not name-factory lookup");
    assert_eq!(raw_types, ("function".into(), "function".into()));
    let mut observations = Vec::new();
    for (health, health_max) in [(7501, 16003), (93, 24005), (18007, 24005)] {
        env.state().borrow_mut().current_target = Some(npc(health, health_max));
        let observed: (i32, i32) = env
            .eval("return UnitHealth('target'), UnitHealthMax('target')")
            .expect("read seeded NPC snapshot");
        assert_eq!(
            observed,
            (health, health_max),
            "absolute health, not percent"
        );
        let percentage = 100.0 * f64::from(health) / f64::from(health_max);
        assert_ne!(f64::from(observed.0), percentage);
        assert_ne!(observed.1, 100);
        observations.push(json!({"seed":{"health":health,"health_max":health_max,
            "is_player":false},"observed":observed,"percentage_control":percentage}));
    }
    let output =
        std::env::var_os("WOW_SIM_P1133_HEALTH_OUT").expect("absolute own result path required");
    assert!(std::path::Path::new(&output).is_absolute());
    let proof = json!({"scope":"current bare Era NPC numeric values across three snapshot mutations",
        "configured_interface":ACTIVE_INTERFACE_VERSION,"source_interface":11303,
        "raw_types":raw_types,"observations":observations,"runtime_changes":0,
        "native_credit":false,"historical_signature_credit":false,"source_ledger_mutated":false});
    std::fs::write(output, serde_json::to_vec_pretty(&proof).unwrap())
        .expect("own model observations");
    eprintln!("P1133_NPC_HEALTH {proof}");
}
