#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn store_catalog_methods_remain_in_secureenv_after_environment_cleanup() {
    test_timeout! {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_EnvironmentCleanup"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_EnvironmentCleanup"));
                assert_empty_secure_store_catalog(env);
                env.apply_post_load_workarounds();
                assert_empty_secure_store_catalog(env);
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

fn assert_empty_secure_store_catalog(env: &WowLuaEnv) {
    env.exec_rilua_secure(
        r#"
        local store = C_StoreSecure
        assert(type(store.GetProductGroups) == "function")
        assert(type(store.GetProductGroupInfo) == "function")
        assert(type(store.GetProducts) == "function")
        assert(type(store.GetEntryInfo) == "function")
        assert(type(store.GetProductInfo) == "function")
        assert(type(store.IsAvailable) == "function")
        assert(type(store.GetCurrencyInfo) == "function")
        assert(type(store.GetProductList) == "function")
        assert(#store.GetProductGroups() == 0)
        assert(store.GetProductGroupInfo(1) == nil)
        assert(#store.GetProducts(1) == 0)
        assert(store.GetEntryInfo(1) == nil)
        assert(store.GetProductInfo(1) == nil)
        assert(store.IsAvailable() == false)
        assert(store.HasProductList() == false)
        assert(store.HasPurchaseList() == false)
        "#,
    )
    .expect("Retail catalog methods should remain available in secureenv");
}
