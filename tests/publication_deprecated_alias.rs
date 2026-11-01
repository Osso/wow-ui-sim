#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "common/publication_sweep.rs"]
mod sweep;

prefork_full_ui_case! {
fn death_recap_deprecated_alias_reads_supplied_events(env: &WowLuaEnv) {
    use wow_ui_sim::lua_api::state::{DeathRecapEntry, DeathRecapEventInfo};

    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.death_recaps.clear();
        for (id, hp, amount, eligible) in [(11, 80, 20, true), (22, 45, 55, false)] {
            sim.death_recaps.push(DeathRecapEntry {
                recap_id: id,
                default_eligible: eligible,
                events: vec![DeathRecapEventInfo {
                    current_hp: hp,
                    amount: Some(amount),
                    timestamp: Some(id as f64),
                    source_guid: Some(format!("Creature-0-0-0-0-{id}-0000000000")),
                }],
                max_health: 100,
                zone_name: "fixture".into(),
                killing_blows: vec![],
            });
        }
    }
    env.exec(r#"
        assert(DeathRecap_GetEvents == C_DeathRecap.GetRecapEvents)
        assert(select('#', DeathRecap_GetEvents(-1)) == 0)
        assert(DeathRecap_GetEvents()[1].currentHP == 80)
        assert(DeathRecap_GetEvents(11)[1].amount == 20)
        assert(DeathRecap_GetEvents(22)[1].currentHP == 45)
        assert(DeathRecap_GetEvents(22)[1].amount == 55)
        assert(DeathRecap_GetEvents(22)[1].timestamp == 22)
        assert(DeathRecap_GetEvents(22)[1].sourceGUID == 'Creature-0-0-0-0-22-0000000000')
        assert(select('#', DeathRecap_GetEvents(999)) == 0)
    "#).unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        let recap = &mut sim.death_recaps[1];
        recap.default_eligible = true;
        let event = recap.events[0].clone();
        recap.events.resize(12, event);
    }
    env.exec("assert(#DeathRecap_GetEvents() == 10 and DeathRecap_GetEvents()[1].currentHP == 45)")
        .unwrap();
}
}

prefork_full_ui_case! {
fn deprecated_alias_attribution_requires_loaded_publisher_and_exact_identity(env: &WowLuaEnv) {
    let empty = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    assert!(sweep::read_deprecated_aliases(&empty).is_empty());
    {
        let aliases = sweep::read_deprecated_aliases(&env);
        let removed = |symbol: &str| sweep::Entry {
            id: symbol.into(),
            section: "global-api".into(),
            direction: "removed".into(),
            symbol: symbol.into(),
            page_default: None,
            kind: None,
        };
        // Bare and namespaced direct aliases from different loaded deprecation files.
        for (symbol, file) in [
            ("CombatLogAddFilter", "Deprecated_CombatLog.lua"),
            ("DeathRecap_GetEvents", "Deprecated_CombatLog.lua"),
            (
                "C_Housing.IsInsideOwnHouse",
                "Mainline/Deprecated_12_1_0.lua",
            ),
            (
                "C_UnitAuras.RemovePrivateAuraAppliedSound",
                "Shared/Deprecated_12_1_0.lua",
            ),
            ("GetInventorySlotInfo", "Deprecated_PaperDoll.lua"),
        ] {
            let (_, detail, ok, _, _) = sweep::probe_entry(&env, &removed(symbol), true, &aliases);
            assert!(ok, "{symbol}: {detail}");
            assert!(detail.contains(file), "{symbol}: {detail}");
        }
        // A stale cache mapping must not bless an unrelated replacement function.
        env.exec("CombatLogAddFilter = function() return 'unrelated' end")
            .unwrap();
        assert!(!sweep::probe_entry(&env, &removed("CombatLogAddFilter"), true, &aliases).2);
    };
}
}
