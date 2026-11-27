//! `GetLFGRoleUpdate` reports the supplied role check through Blizzard's
//! `LFGUtilShared.lua` `GetLFGMode`, which returns `"rolecheck"` only for the
//! role check's category and lfgID.
#![cfg(feature = "client-retail")]

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

const MODES: &str = r#"
local category, lfgID, otherID, battleground = ...
local function mode(c, id) return tostring((GetLFGMode(c, id))) end
local before = mode(category, lfgID)
A_Admin.SetLFGRoleCheck(category, lfgID, 2, 3, battleground)
local inProgress, slots, members, roleCategory, roleID, isBattleground = GetLFGRoleUpdate()
local active = table.concat({
    mode(category, lfgID),
    mode(category, otherID),
    mode(category % 6 + 1, lfgID),
    tostring(inProgress), slots, members, roleCategory, roleID, tostring(isBattleground),
}, ",")
A_Admin.ClearLFGRoleCheck()
return before .. "|" .. active .. "|" .. mode(category, lfgID)
"#;

fn blizzard_ui_dir() -> std::path::PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

/// The simulator's own `GetLFGMode` stands in until Blizzard_LFGUtil loads;
/// the test loads the runtime-selected original so its `GetLFGMode` is used.
fn load_lfg_util() -> WowLuaEnv {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    assert!(addons.iter().any(|(name, _)| name == "Blizzard_LFGUtil"));
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &blizzard_ui_dir(),
        &["Blizzard_LFGUtil"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_LFGUtil"));
    env
}

fn modes(category: i32, lfg_id: i32, other_id: i32, battleground: bool) -> String {
    let env = load_lfg_util();
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
