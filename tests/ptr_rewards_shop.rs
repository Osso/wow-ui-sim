//! PTR rewards-shop predicates use separate server-mirrored gates.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn rewards_shop_defaults_to_one_false_boolean() {
    if !cfg!(feature = "client-ptr") {
        return;
    }
    let env = WowLuaEnv::new().unwrap();
    let values: (bool, i64, bool, i64) = env
        .eval("return C_StorePublic.IsRewardsShopEnabled(), select('#', C_StorePublic.IsRewardsShopEnabled()), C_GameRules.IsRewardsShopEnabled(), select('#', C_GameRules.IsRewardsShopEnabled())")
        .unwrap();
    assert_eq!(values, (false, 1, false, 1));
}

#[test]
fn rewards_shop_tracks_mirrored_flags_and_integer_rule() {
    if !cfg!(feature = "client-ptr") {
        return;
    }
    let env = WowLuaEnv::new().unwrap();
    let state = env.state();
    let flags = |r: bool, b: bool, o: bool| {
        let state = state.borrow();
        for (name, value) in [
            ("rewardsShopEnabled", r),
            ("bpayStoreEnable", b),
            ("rewardsShopOverrideEnabled", o),
        ] {
            state
                .cvars
                .set_imported(name, if value { "1" } else { "0" });
        }
    };
    let check = |store: bool, rules: bool| {
        let actual: (bool, bool) = env
            .eval("return C_StorePublic.IsRewardsShopEnabled(), C_GameRules.IsRewardsShopEnabled()")
            .unwrap();
        assert_eq!(actual, (store, rules));
    };
    flags(false, true, false);
    env.exec("A_Admin.SetGameRule('RewardsShopFlags', 1)")
        .unwrap();
    check(false, false);
    flags(true, false, false);
    check(true, false);
    flags(true, true, false);
    for (rule, expected) in [
        ("nil", false),
        ("'1'", false),
        ("1.5", false),
        ("0", false),
        ("2", false),
        ("1", true),
        ("3", true),
    ] {
        env.exec(&format!("A_Admin.SetGameRule('RewardsShopFlags', {rule})"))
            .unwrap();
        check(true, expected);
    }
    flags(true, false, true);
    check(true, true);
    flags(true, true, true);
    check(true, true);
}
