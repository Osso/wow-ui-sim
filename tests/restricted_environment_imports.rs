#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const RESTRICTED_ENV_IMPORT_PROBE: &str =
    include_str!("common/restricted_environment_import_probe.lua");

#[test]
fn restricted_environment_secure_snippet_imports_item_and_action_apis() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_RestrictedAddOnEnvironment"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_RestrictedAddOnEnvironment"));
    assert_restricted_environment_imports(&env);
}

fn assert_restricted_environment_imports(env: &WowLuaEnv) {
    let result: (bool, bool, bool, bool, bool, bool) = env
        .eval(RESTRICTED_ENV_IMPORT_PROBE)
        .expect("original restricted snippet should return API results");
    assert_eq!(result, (true, true, false, false, true, false));
}
