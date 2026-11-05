use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};

const ROOT: &str = "Blizzard_AzeriteRespecUI";
const RESPEC_COST: i64 = 50_000;
const ENOUGH_MONEY: i64 = 100_000;

const INSTALL_TOOLTIP_SPY_LUA: &str = r#"
_G.__azerite_respec_tooltip_texts = {}
local originalSetText = GameTooltip.SetText

GameTooltip.SetText = function(self, text, ...)
    table.insert(_G.__azerite_respec_tooltip_texts, text)
    return originalSetText(self, text, ...)
end
"#;

const PREPARE_BUTTON_STATE_LUA: &str = r#"
AzeriteRespecFrame.respecCost = 50000
AzeriteRespecFrame.respecItemLocation = { itemID = 158041, bagID = 0, slotIndex = 9 }
AzeriteRespecFrame:UpdateAzeriteRespecButtonState()
"#;

// AzeriteRespecUI vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
timerunning_util_panel_fixture! {
#[test]
fn blizzard_azerite_respec_ui_button_disabled_tooltip_depends_on_money() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                seed_respec_money(env, 0);
                clear_recorded_lua_errors(env);
                load_azerite_respec_ui(env);
                install_tooltip_spy(env);

                prepare_button_state(env);
                assert_disabled_hover_shows_not_enough_money(env);

                seed_respec_money(env, ENOUGH_MONEY);
                prepare_button_state(env);
                assert_enabled_hover_hides_tooltip(env);
                assert_no_lua_errors(env);
            });
        });
    });
}
}

fn seed_respec_money(env: &wow_ui_sim::lua_api::WowLuaEnv, player_money: i64) {
    let mut state = env.state().borrow_mut();
    state.azerite_empowered.respec_cost = RESPEC_COST;
    state.player.money = player_money;
}

fn load_azerite_respec_ui(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (loaded, reason): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AzeriteRespecUI")"#)
        .expect("C_AddOns.LoadAddOn should return for Blizzard_AzeriteRespecUI");
    assert!(loaded, "`{ROOT}` should load: {reason:?}");
}

fn install_tooltip_spy(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(INSTALL_TOOLTIP_SPY_LUA)
        .expect("AzeriteRespec tooltip spy should install");
}

fn prepare_button_state(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(PREPARE_BUTTON_STATE_LUA)
        .expect("AzeriteRespec button state should update");
}

fn assert_disabled_hover_shows_not_enough_money(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let tooltip_text: Option<String> = env
        .eval(
            r#"
            _G.__azerite_respec_tooltip_texts = {}
            AzeriteRespecFrame.ButtonFrame.AzeriteRespecButton:OnMouseEnter()
            return _G.__azerite_respec_tooltip_texts[1]
            "#,
        )
        .expect("disabled AzeriteRespecButton hover should run");
    let expected: String = env
        .eval("return NOT_ENOUGH_GOLD_FOR_AZERITE_RESPEC")
        .expect("AzeriteRespec not-enough-money string should exist");
    assert_eq!(tooltip_text.as_deref(), Some(expected.as_str()));
}

fn assert_enabled_hover_hides_tooltip(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let tooltip_count: i64 = env
        .eval(
            r#"
            _G.__azerite_respec_tooltip_texts = {}
            AzeriteRespecFrame.ButtonFrame.AzeriteRespecButton:OnMouseEnter()
            return #_G.__azerite_respec_tooltip_texts
            "#,
        )
        .expect("enabled AzeriteRespecButton hover should run");
    assert_eq!(
        tooltip_count, 0,
        "`{ROOT}` enabled respec button hover should not show disabled tooltip"
    );
}

fn assert_no_lua_errors(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let errors = recorded_lua_errors(env);
    assert!(
        errors.is_empty(),
        "`{ROOT}` emitted Lua errors while checking button tooltip:\n{}",
        errors.join("\n")
    );
}
