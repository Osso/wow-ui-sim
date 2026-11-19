//! Retail shape tests for supplied major-faction renown information.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::{MajorFactionData, WowLuaEnv};

#[test]
fn get_major_faction_renown_info_projects_supplied_values() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().major_factions.insert(
        2507,
        MajorFactionData {
            faction_id: 2507,
            name: "Fixture faction".into(),
            renown_level: 4,
            renown_reputation_earned: 311,
            renown_level_threshold: 2500,
            ..Default::default()
        },
    );
    let actual: (i32, i32, i32) = env
        .eval(
            "local d=C_MajorFactions.GetMajorFactionRenownInfo(2507); \
             return d.renownLevel, d.renownReputationEarned, d.renownLevelThreshold",
        )
        .unwrap();
    assert_eq!(actual, (4, 311, 2500));
    env.exec("assert(C_MajorFactions.GetMajorFactionRenownInfo(987654) == nil)")
        .unwrap();
}
