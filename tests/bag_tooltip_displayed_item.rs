#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! Bag-item tooltip data carries the item GUID as top-level `guid`.
//! INFERRED R88g9 (native evidence pending) for the field. Original consumer:
//! `TooltipUtil.GetDisplayedItem` (Blizzard_SharedXMLGame/Tooltip/TooltipUtil.lua:9-19)
//! resolves the hovered bag item through `C_Item.GetItemLinkByGUID(tooltipData.guid)`.

use wow_ui_sim::lua_api::WowLuaEnv;

const HOVER_BAG_SLOT: &str = r#"
local slot = ...
GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
GameTooltip:SetBagItem(0, slot)
local _, link, itemID = TooltipUtil.GetDisplayedItem(GameTooltip)
return link and link:match("|Hitem:(%d+):") or "none", itemID or 0
"#;

fn hover_bag_slot(env: &WowLuaEnv, slot: i32) -> (String, i64) {
    let chunk = format!("return (function(...) {HOVER_BAG_SLOT} end)({slot})");
    env.eval(&chunk).expect("the original tooltip should show the bag item")
}

fn assert_hovered_bag_items_resolve_by_guid(env: &WowLuaEnv) {
    env.exec("A_Admin.ClearBags(); A_Admin.AddBagItem(0, 1, 6948); A_Admin.AddBagItem(0, 2, 1728)")
        .expect("supply two bag items");
    assert_eq!(hover_bag_slot(env, 1), ("6948".to_owned(), 6948));
    assert_eq!(hover_bag_slot(env, 2), ("1728".to_owned(), 1728));
}

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn hovered_bag_items_resolve_through_their_tooltip_guid(env: &WowLuaEnv) {
    assert_hovered_bag_items_resolve_by_guid(env);
}
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_hovered_bag_items_resolve_through_their_tooltip_guid() {
    crate::common::with_timeout(120, || {
        let env = crate::common::forever_game_env();
        assert_hovered_bag_items_resolve_by_guid(&env);
    });
}
