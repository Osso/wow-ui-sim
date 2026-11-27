//! `C_Item.RequestLoadItemDataByID` through the original Blizzard_ObjectAPI
//! consumer: `Item:ContinueOnItemLoad` registers with `ItemEventListener`
//! (AsyncCallbackSystem.lua), which calls the request and fires or clears the
//! callbacks from `ITEM_DATA_LOAD_RESULT(itemID, success)`.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

const CONTINUE_ON_LOAD: &str = r#"
    local loaded = {}
    for _, itemID in ipairs({6948, 2147480}) do
        Item:CreateFromItemID(itemID):ContinueOnItemLoad(function()
            loaded[#loaded + 1] = itemID
        end)
    end
    return #loaded, loaded[1]
"#;

fn env_with_object_api() -> WowLuaEnv {
    let env = common::env_with_shared_xml();
    let ui = common::blizzard_ui_dir();
    common::load_required_blizzard_addon(&env, &ui, "Blizzard_Colors");
    common::load_required_blizzard_addon(&env, &ui, "Blizzard_ObjectAPI");
    env
}

#[test]
fn item_load_callbacks_run_only_for_catalog_items() {
    let env = env_with_object_api();
    let (count, first): (i64, i64) = env.eval(CONTINUE_ON_LOAD).unwrap();
    // 6948 (Hearthstone) is in the item catalog; 2147480 is not.
    assert_eq!((count, first), (1, 6948));
}
