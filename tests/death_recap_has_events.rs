//! Retail deprecated alias consumer for supplied recap records.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn death_recap_has_events_deprecated_alias_reads_supplied_recaps(env: &WowLuaEnv) {
    use wow_ui_sim::lua_api::state::{DeathRecapEntry, DeathRecapEventInfo};
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.death_recaps.clear();
        for (id, eligible) in [(11, true), (22, false)] {
            sim.death_recaps.push(DeathRecapEntry {
                recap_id: id,
                default_eligible: eligible,
                events: if id == 11 { vec![] } else { vec![DeathRecapEventInfo {
                    current_hp: 45, amount: Some(55), timestamp: None, source_guid: None,
                }] },
                max_health: 100, zone_name: "fixture".into(), killing_blows: vec![],
            });
        }
    }
    env.exec(r#"
        assert(DeathRecap_HasEvents == C_DeathRecap.HasRecapEvents)
        assert(select('#', DeathRecap_HasEvents()) == 1)
        assert(DeathRecap_HasEvents() == true)
        assert(DeathRecap_HasEvents(nil) == true)
        assert(DeathRecap_HasEvents('22.9') == true)
        assert(DeathRecap_HasEvents(999) == false)
        assert(DeathRecap_HasEvents(-1) == false)
        for _, value in ipairs({false, {}, 'not a number', 2147483648, -2147483649}) do
            assert(not pcall(DeathRecap_HasEvents, value))
        end
    "#).unwrap();
    {
        let state = env.state();
        state.borrow_mut().death_recaps[0].default_eligible = false;
    }
    env.exec(r#"
        assert(DeathRecap_HasEvents() == false)
        assert(DeathRecap_HasEvents(11) == true)
        assert(DeathRecap_HasEvents(22) == true)
    "#).unwrap();
}
}
