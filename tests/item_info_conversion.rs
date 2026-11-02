//! ItemInfo conversion cases from the Retail 70688 shared parser evidence.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_info_signed_numbers_truncate_without_positive_check() {
    let env = WowLuaEnv::new().unwrap();
    let values: (i32, i32, i32) = env
        .eval("return C_Item.GetItemIDForItemInfo(-1.9), C_Item.GetItemIDForItemInfo(0), C_Item.GetItemIDForItemInfo(6948.9)")
        .unwrap();
    assert_eq!(values, (-1, 0, 6948));
}

#[test]
fn item_info_item_links_resolve_id() {
    let env = WowLuaEnv::new().unwrap();
    let values: (i32, i32) = env
        .eval("return C_Item.GetItemIDForItemInfo('item:6948'), C_Item.GetItemIDForItemInfo('|Hitem:6948::::::::|h[Hearthstone]|h')")
        .unwrap();
    assert_eq!(values, (6948, 6948));
}

#[test]
fn item_info_names_resolve_against_generated_items() {
    let env = WowLuaEnv::new().unwrap();
    let id: i32 = env
        .eval("return C_Item.GetItemIDForItemInfo('Hearthstone')")
        .unwrap();
    assert_eq!(id, 6948);
}

#[test]
fn item_info_unresolved_strings_resolve_to_zero() {
    let env = WowLuaEnv::new().unwrap();
    let id: i32 = env
        .eval("return C_Item.GetItemIDForItemInfo('no such item WM23')")
        .unwrap();
    assert_eq!(id, 0);
}
