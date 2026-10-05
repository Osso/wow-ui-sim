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
        assert_eq!(aliases.len(), 15);
        for name in aliases.keys() {
            let entry = sweep::Entry {
                id: name.clone(),
                section: "global-api".into(),
                direction: "removed".into(),
                symbol: name.clone(),
                page_default: None,
                kind: None,
            };
            let (_, detail, ok, _, _) = sweep::probe_entry(&env, &entry, true, &aliases);
            assert!(ok, "{name}: {detail}");
            assert!(detail.contains("Deprecated_CombatLog.lua"), "{detail}");
        }
        // A stale cache mapping must not bless an unrelated replacement function.
        env.exec("CombatLogAddFilter = function() return 'unrelated' end")
            .unwrap();
        let entry = sweep::Entry {
            id: "replacement".into(),
            section: "global-api".into(),
            direction: "removed".into(),
            symbol: "CombatLogAddFilter".into(),
            page_default: None,
            kind: None,
        };
        assert!(!sweep::probe_entry(&env, &entry, true, &aliases).2);
    });
}
