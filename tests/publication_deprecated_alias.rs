#![cfg(feature = "client-retail")]

#[path = "common/prefork_full_ui_preload.rs"]
mod full_ui;
#[path = "common/publication_sweep.rs"]
mod sweep;

#[test]
fn deprecated_alias_attribution_requires_loaded_publisher_and_exact_identity() {
    let empty = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    assert!(sweep::read_deprecated_aliases(&empty).is_empty());
    crate::common::with_exclusive_workload(|| {
        let env = full_ui::preload_full_game_ui().unwrap();
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
    });
}
