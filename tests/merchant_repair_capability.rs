#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

const CLOSED_QUERY: &str = r#"
    assert(CanMerchant() == false)
    assert(CanMerchantRepair() == false, "no merchant cannot repair")
    assert(select('#', CanMerchantRepair()) == 1)
"#;

const BAG_EVENT: &str = r#"
    assert(MerchantFrame:IsEventRegistered("BAG_UPDATE"))
    A_Admin.ClearBags()
    A_Admin.AddBagItem(0, 1, 6948, 1)
    A_Admin.FireEvent("BAG_UPDATE", 0)
    assert(not MerchantRepairAllButton:IsShown(), "repair-all must stay hidden")
    assert(not MerchantRepairItemButton:IsShown(), "repair-item must stay hidden")
    assert(not MerchantGuildBankRepairButton:IsShown(), "guild repair must stay hidden")
    print("MERCHANT_REPAIR_BAG_DONE")
"#;

#[test]
fn merchant_repair_default_and_bootstrap() {
    let env = WowLuaEnv::new().unwrap();
    assert!(!env.state().borrow().merchant_repair_capable);
    env.exec(CLOSED_QUERY).unwrap();
    env.loader_env().restore_post_cleanup_globals().unwrap();
    env.exec(CLOSED_QUERY).unwrap();
}

#[test]
fn merchant_repair_requires_open_and_configured_capability() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(CLOSED_QUERY).unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.merchant_frame_open = true;
        state.merchant_items = vec![6948, 117];
    }
    env.exec("assert(CanMerchant()); assert(CanMerchantRepair() == false)")
        .unwrap();
    env.state().borrow_mut().merchant_repair_capable = true;
    env.exec("assert(CanMerchant()); assert(CanMerchantRepair() == true)")
        .unwrap();
    {
        let state = env.state().borrow();
        assert!(state.merchant_frame_open);
        assert_eq!(state.merchant_items, vec![6948, 117]);
    }
    env.exec("CloseMerchant()").unwrap();
    env.exec(CLOSED_QUERY).unwrap();
    let state = env.state().borrow();
    assert!(state.merchant_repair_capable);
    assert_eq!(state.merchant_items, vec![6948, 117]);
    assert_eq!(state.events.last().unwrap().name, "MERCHANT_CLOSED");
}

#[test]
fn merchant_repair_environments_are_independent() {
    let first = WowLuaEnv::new().unwrap();
    let second = WowLuaEnv::new().unwrap();
    {
        let mut state = first.state().borrow_mut();
        state.merchant_frame_open = true;
        state.merchant_repair_capable = true;
        state.merchant_items = vec![117];
    }
    first.exec("assert(CanMerchantRepair() == true)").unwrap();
    second.exec(CLOSED_QUERY).unwrap();
    assert!(!second.state().borrow().merchant_repair_capable);
    assert!(second.state().borrow().merchant_items.is_empty());
    first.loader_env().restore_post_cleanup_globals().unwrap();
    first.exec("assert(CanMerchantRepair() == true)").unwrap();
    assert_eq!(first.state().borrow().merchant_items, vec![117]);
}

#[test]
fn merchant_repair_cached_bag_event() {
    let root = tempfile::tempdir().unwrap();
    let output = std::process::Command::new("timeout")
        .args(["90", env!("CARGO_BIN_EXE_wow-sim")])
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            BAG_EVENT,
            "lua-errors",
        ])
        .env("XDG_DATA_HOME", root.path().join("data"))
        .env("WOW_SIM_WTF_PATH", root.path().join("wtf"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("MERCHANT_REPAIR_BAG_DONE"), "{stdout}");
    assert!(stdout.trim_end().ends_with("[]"), "{stdout}");
}
