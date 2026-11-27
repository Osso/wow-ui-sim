//! `C_Item.RequestLoadItemDataByID` through the original Blizzard_ObjectAPI
//! consumer: `Item:ContinueOnItemLoad` registers with `ItemEventListener`
//! (AsyncCallbackSystem.lua), which calls the request and fires or clears the
//! callbacks from `ITEM_DATA_LOAD_RESULT(itemID, success)`; the callback
//! receives that `success` flag.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

const CONTINUE_ON_LOAD: &str = r#"
    local fired = {}
    for _, itemID in ipairs({6948, 2147480}) do
        Item:CreateFromItemID(itemID):ContinueOnItemLoad(function(id, success)
            fired[#fired + 1] = id .. ":" .. tostring(success)
        end)
    end
    return table.concat(fired, ",")
"#;

fn env_with_object_api() -> WowLuaEnv {
    let env = common::env_with_shared_xml();
    let ui = common::blizzard_ui_dir();
    common::load_required_blizzard_addon(&env, &ui, "Blizzard_Colors");
    common::load_required_blizzard_addon(&env, &ui, "Blizzard_ObjectAPI");
    env
}

#[test]
fn item_load_callbacks_report_catalog_membership() {
    let env = env_with_object_api();
    let fired: String = env.eval(CONTINUE_ON_LOAD).unwrap();
    // 6948 (Hearthstone) is in the item catalog; 2147480 is not. Forever's
    // AsyncCallbackSystem.lua fires item callbacks on failure too; Retail's
    // clears them.
    let expected = if cfg!(feature = "client-wowforever") {
        "6948:true,2147480:false"
    } else {
        "6948:true"
    };
    assert_eq!(fired, expected);
}
