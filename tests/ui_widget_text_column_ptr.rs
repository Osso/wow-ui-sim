//! PTR supplied-state consumer and owning publication observation; no native parity.
#![cfg(feature = "client-ptr")]

#[path = "widget_text_column/fixtures.rs"]
mod fixtures;
#[path = "widget_text_column/ptr_lifecycle.rs"]
mod lifecycle;
#[path = "common/prefork_full_ui_preload.rs"]
mod owning_startup;
#[path = "common/publication_sweep.rs"]
mod sweep;

#[test]
fn ptr_owning_9_1_0_observation_after_original_startup_and_consumer() {
    crate::common::with_timeout(90, || {
        assert_eq!(
            wow_ui_sim::client_profile::ACTIVE,
            wow_ui_sim::client_profile::ClientProfile::Ptr
        );
        let cache =
            wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("resolve active PTR cache");
        assert!(
            cache.ends_with("ptr/AddOns"),
            "wrong cache: {}",
            cache.display()
        );
        // Same Game discovery/bootstrap/events as the existing full-UI owner harness.
        // No prefork feature, preload fork or private callback invocation is needed.
        let env = owning_startup::load_full_game_ui()
            .expect("ordinary PTR startup prerequisite; failure is not a TextColumn regression");
        lifecycle::observe_preloaded(&env);
        sweep::run_publication_sweep(
            &env,
            &sweep::SweepSpec {
                register: include_str!("../data/patch-api/sources/9.1.0-wikitext-register.json"),
                known_gaps: include_str!("data/patch_9_1_0_sweep_known_gaps.json"),
                row_count: 179,
                register_env: "TEXT_COLUMN_PTR_P910_REGISTER",
                out_env: "TEXT_COLUMN_PTR_P910_OUT",
                later_registers: &[
                    include_str!("../data/patch-api/sources/9.1.5-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/9.2.0-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/9.2.5-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/9.2.7-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/10.0.0-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/10.0.2-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/10.0.5-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/10.0.7-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/10.1.0-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/10.1.5-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/10.1.7-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/10.2.0-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/10.2.5-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/10.2.6-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/10.2.7-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/11.0.0-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/11.0.2-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/11.0.5-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/11.0.7-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/11.1.0-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/11.1.5-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/11.1.7-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/11.2.0-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/11.2.5-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/11.2.7-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/12.0.0-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/12.0.1-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/12.0.5-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/12.0.7-wikitext-register.json"),
                    include_str!("../data/patch-api/sources/12.1.0-wikitext-register.json"),
                ],
            },
        );
    });
}
