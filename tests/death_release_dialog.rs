#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! `GetReleaseTimeRemaining` and `C_DeathRecap.HasRecapEvents` read the host
//! release timer and recap list. Original consumers: the `PLAYER_DEAD` handler
//! `GameEvent.HandlePlayerDead` (Blizzard_Game Mainline/EventImplementation.lua:
//! 221-231 on Retail, Camelot/EventImplementation.lua:17-29 on Forever) shows
//! the DEATH dialog only for a positive or `-1` timer, and its `OnShow`
//! (Blizzard_StaticPopup_Game/Mainline/GameDialogDefs.lua:192-221) copies the
//! timer to `dialog.timeleft` and enables the recap button only with recap events.
//! `C_DeathInfo.GetSelfResurrectOptions` feeds `GameDialogDefsUtil.
//! GetSelfResurrectDialogOptions` (GameDialogDefsUtil.lua:3-22): the DEATH
//! dialog shows button 2 only with an option, and its `OnUpdate` labels and
//! enables that button from the first option after the usable-first sort.

use wow_ui_sim::lua_api::WowLuaEnv;

const DIE: &str = r#"
local seconds, recapID = ...
StaticPopup_Hide("DEATH")
A_Admin.ClearDeathRecaps()
if recapID then A_Admin.AddDeathRecap(recapID, true, 100, 40, 60) end
A_Admin.SetReleaseTimeRemaining(seconds)
A_Admin.FireEvent("PLAYER_DEAD")
local _, dialog = StaticPopup_Visible("DEATH")
if not dialog then return false, 0, false end
return true, dialog.timeleft, dialog:GetButton4():IsEnabled()
"#;

fn die(env: &WowLuaEnv, seconds: f64, recap: &str) -> (bool, f64, bool) {
    let chunk = format!("return (function(...) {DIE} end)({seconds}, {recap})");
    env.eval(&chunk)
        .expect("the original death handler should run")
}

const RESURRECT: &str = r#"
local options = ...
StaticPopup_Hide("DEATH")
A_Admin.ClearDeathRecaps()
A_Admin.ClearSelfResurrectOptions()
for _, o in ipairs(options) do
    A_Admin.AddSelfResurrectOption(o[1], Enum.SelfResurrectOptionType.Spell, o[2], o[3], false, o[4])
end
A_Admin.SetReleaseTimeRemaining(360)
A_Admin.FireEvent("PLAYER_DEAD")
local _, dialog = StaticPopup_Visible("DEATH")
return dialog:GetButton2():IsShown()
"#;

const BUTTON2: &str = r#"
local _, dialog = StaticPopup_Visible("DEATH")
local button = dialog:GetButton2()
return button:GetText(), button:IsEnabled(), button.option and button.option.id or 0
"#;

fn offer_resurrects(env: &WowLuaEnv, options: &str) -> bool {
    let chunk = format!("return (function(...) {RESURRECT} end)({options})");
    env.eval(&chunk)
        .expect("the original death handler should run")
}

fn first_resurrect_button(env: &WowLuaEnv) -> (String, bool, f64) {
    env.fire_on_update(0.1).expect("the DEATH dialog OnUpdate should run");
    env.eval(BUTTON2).expect("button 2 should be readable")
}

fn assert_resurrect_options_follow_inputs(env: &WowLuaEnv) {
    assert!(!offer_resurrects(env, "{}"));
    let usable_second = r#"{{"Soulstone", 20707, false, 1}, {"Reincarnation", 20608, true, 2}}"#;
    assert!(offer_resurrects(env, usable_second));
    let expected = ("Reincarnation".to_owned(), true, 20608.0);
    assert_eq!(first_resurrect_button(env), expected);
    assert!(offer_resurrects(env, r#"{{"Soulstone", 20707, false, 1}}"#));
    let expected = ("Soulstone".to_owned(), false, 20707.0);
    assert_eq!(first_resurrect_button(env), expected);
    env.exec("A_Admin.ClearSelfResurrectOptions()").expect("clear options");
}

fn assert_death_dialog_follows_inputs(env: &WowLuaEnv) {
    assert_eq!(die(env, 360.0, "11"), (true, 360.0, true));
    assert_eq!(die(env, -1.0, "nil"), (true, -1.0, false));
    assert_eq!(die(env, 0.0, "11"), (false, 0.0, false));
    assert_resurrect_options_follow_inputs(env);
}

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn death_dialog_follows_the_release_timer_and_recaps(env: &WowLuaEnv) {
    assert_death_dialog_follows_inputs(env);
}
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_death_dialog_follows_the_release_timer_and_recaps() {
    crate::common::with_timeout(120, || {
        let env = WowLuaEnv::new().expect("create Forever Lua environment");
        crate::common::load_game_startup_inventory(&env);
        env.sync_addon_names_to_lua();
        env.apply_post_load_workarounds();
        let game = wow_ui_sim::screen::ScreenKind::Game;
        wow_ui_sim::startup::fire_startup_events_for_screen(&env, game);
        assert_death_dialog_follows_inputs(&env);
    });
}
