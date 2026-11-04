#![cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-mists"
))]

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

const STORE_GROUP_CONTRACT: &str = include_str!("fixtures/store_group_contract.lua");

#[test]
fn store_group_shape_resolves_current_demo_entry_for_active_profile() {
    let env = WowLuaEnv::new().expect("create fresh store API environment");
    assert_store_group_contract(&env);
}

#[test]
fn store_group_shape_survives_original_environment_cleanup() {
    test_timeout! {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_EnvironmentCleanup"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_EnvironmentCleanup"));
                assert_store_group_contract(env);
                env.apply_post_load_workarounds();
                assert_store_group_contract(env);
            },
        );
    }
}

#[test]
fn store_group_restore_preserves_existing_getter() {
    let env = WowLuaEnv::new().expect("create fresh store API environment");
    env.exec("StoreGroupCustomGetter = function() return 47 end; C_StoreSecure.GetProductGroups = StoreGroupCustomGetter")
        .unwrap();
    env.restore_post_cleanup_globals();
    let unchanged: bool = env
        .eval("return C_StoreSecure.GetProductGroups == StoreGroupCustomGetter and C_StoreSecure.GetProductGroups() == 47")
        .unwrap();
    assert!(unchanged);
}

fn assert_store_group_contract(env: &WowLuaEnv) {
    let mode = match ACTIVE {
        ClientProfile::Retail | ClientProfile::Ptr => "records",
        ClientProfile::Mists => "ids",
        profile => panic!("store group contract not audited for {profile:?}"),
    };
    let chunk = format!("local store_group_mode = {mode:?}\n{STORE_GROUP_CONTRACT}");
    let passed: bool = env
        .eval(&chunk)
        .expect("active-profile store group shape and demo lookups must match");
    assert!(passed, "shared store group contract must finish");
}
