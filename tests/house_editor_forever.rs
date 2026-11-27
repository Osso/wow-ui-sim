#![cfg(feature = "client-wowforever")]
//! Forever publishes the Retail `C_HouseEditor` request model (the pinned
//! HouseEditorUIDocumentation.lua is byte-identical to Retail's). The Forever
//! housing addons declare `AllowLoadGameType: standard` and are not selected
//! for the camelot/mainline game types, so the test drives the API with
//! `A_Admin` inputs and observes `HOUSE_EDITOR_MODE_CHANGED` with the global
//! `RegisterEventCallback`.

use wow_ui_sim::lua_api::WowLuaEnv;

const ENTER: &str = r#"
local mode, playerType = ...
local E = C_HouseEditor
E.LeaveHouseEditor()
A_Admin.SetHouseEditorStatus(true, Enum.HousingResult.Success, mode, playerType)
A_Admin.SetHouseEditorModeAvailability(mode, Enum.HousingResult.Success)
local changed
RegisterEventCallback("HOUSE_EDITOR_MODE_CHANGED", function(_, newMode) changed = newMode end)
local entered = E.EnterHouseEditor()
local activeBeforeReply = E.IsHouseEditorActive()
A_Admin.CompleteHouseEditorModeChange(Enum.HousingResult.Success)
return entered, activeBeforeReply, changed, E.GetActiveHouseEditorMode(),
    E.IsHouseEditorModeActive(mode), E.GetHouseEditorPlayerType()
"#;

type Entered = (f64, bool, f64, f64, bool, f64);

fn enter(env: &WowLuaEnv, mode: i32, player_type: i32) -> Entered {
    let chunk = format!("return (function(...) {ENTER} end)({mode}, {player_type})");
    env.eval(&chunk).expect("the editor request should complete")
}

#[test]
fn forever_house_editor_enters_the_supplied_default_mode() {
    let env = WowLuaEnv::new().expect("create Forever Lua environment");
    assert_eq!(enter(&env, 1, 1), (0.0, false, 1.0, 1.0, true, 1.0));
    assert_eq!(enter(&env, 2, 2), (0.0, false, 2.0, 2.0, true, 2.0));
    let gate: (bool, f64, f64) = env
        .eval(
            "A_Admin.SetHouseEditorStatus(false, 37, 1); return C_HouseEditor.IsHouseEditorStatusAvailable(), \
             C_HouseEditor.GetHouseEditorAvailability(), C_HouseEditor.GetHouseEditorModeAvailability(2)",
        )
        .expect("the supplied gate should be readable");
    assert_eq!(gate, (false, 37.0, 0.0));
}
