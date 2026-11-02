use wow_ui_sim::client_profile::ACTIVE;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_globals_follow_active_profile() {
    let env = WowLuaEnv::new().unwrap();
    for name in [
        "SplitContainerItem",
        "GetItemClassInfo",
        "GetItemInfo",
        "GetItemQualityColor",
        "IsConsumableItem",
        "IsEquippableItem",
        "IsItemInRange",
        "UseItemByName",
        "CloseSocketInfo",
    ] {
        let present: bool = env
            .eval(&format!("return type(rawget(_G, {name:?})) == 'function'"))
            .unwrap();
        assert_eq!(present, ACTIVE.has_legacy_item_globals(), "{name}");
    }
}
