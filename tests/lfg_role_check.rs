//! `GetLFGRoleUpdate` reports the supplied role check through Blizzard's
//! `LFGUtilShared.lua` `GetLFGMode`, which returns `"rolecheck"` only for the
//! role check's category and lfgID.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

const MODES: &str = r#"
local category, lfgID, otherID, battleground = ...
local before = tostring(GetLFGMode(category, lfgID))
A_Admin.SetLFGRoleCheck(category, lfgID, 2, 3, battleground)
local inProgress, slots, members, roleCategory, roleID, isBattleground = GetLFGRoleUpdate()
local active = table.concat({
    tostring(GetLFGMode(category, lfgID)),
    tostring(GetLFGMode(category, otherID)),
    tostring(GetLFGMode(category % 6 + 1, lfgID)),
    tostring(inProgress), slots, members, roleCategory, roleID, tostring(isBattleground),
}, ",")
A_Admin.ClearLFGRoleCheck()
return before .. "|" .. active .. "|" .. tostring(GetLFGMode(category, lfgID))
"#;

fn modes(category: i32, lfg_id: i32, other_id: i32, battleground: bool) -> String {
    let env = WowLuaEnv::new().expect("create Retail Lua environment");
    let chunk = format!(
        "return (function(...) {MODES} end)({category}, {lfg_id}, {other_id}, {battleground})"
    );
    env.eval(&chunk)
        .expect("GetLFGMode should read the supplied role check")
}

#[test]
fn get_lfg_mode_reports_rolecheck_for_supplied_category_and_id() {
    // RF (category 3) compares lfgID; other categories ignore it.
    assert_eq!(
        modes(3, 417, 418, false),
        "nil|rolecheck,nil,nil,true,2,3,3,417,false|nil"
    );
    assert_eq!(
        modes(1, 1203, 1204, true),
        "nil|rolecheck,rolecheck,nil,true,2,3,1,1203,true|nil"
    );
}
