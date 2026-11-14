//! INFERRED host name-ID adapter, isolated from VERIFIED numeric-slot contracts.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::real::inventory_candidates::{
    CandidateLocation, InventoryCandidate,
};

#[test]
fn inferred_one_based_slot_name_adapter() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.player.present = true;
        sim.inventory_candidates
            .slot_names
            .insert("HeadSlot".into(), 1);
        sim.inventory_candidates.accepted.insert(
            (0, false),
            vec![InventoryCandidate {
                location: CandidateLocation::Equipment(0),
                link: Some("host-supplied text".into()),
            }],
        );
    }
    env.exec(
        r#"
        local t={sentinel=42}
        assert(GetInventoryItemsForSlot('hEaDsLoT',t)==t)
        assert(t[1048577]=='host-supplied text' and t.sentinel==42)
    "#,
    )
    .unwrap();
}
