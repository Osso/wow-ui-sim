//! `AbbreviateNumbers`/`AbbreviateLargeNumbers` (LocalizationDocumentation.lua:10-39)
//! format through the abbreviated-number breakpoint model. The original
//! consumer is `FormattingUtil.GetCostString` (Blizzard_SharedXML/FormattingUtil.lua:191-201).
#![cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const COST_STRINGS: &str = r#"
    local function cost(quantity)
        return FormattingUtil.GetCostString(134400, quantity, "", true):match("^(%S+)")
    end
    assert(cost(123456) == "123k", cost(123456))
    assert(cost(2500000) == "2.5m", cost(2500000))
    assert(cost(999) == "999", cost(999))
"#;

const LARGE_NUMBERS: &str = r#"
    assert(AbbreviateLargeNumbers(1234) == "1.2k", AbbreviateLargeNumbers(1234))
    assert(AbbreviateLargeNumbers(12345678) == "12m", AbbreviateLargeNumbers(12345678))
    local options = { breakpointData = {
        { breakpoint = 100, abbreviation = "h", significandDivisor = 100,
          fractionDivisor = 1, abbreviationIsGlobal = false },
    } }
    assert(AbbreviateNumbers(512, options) == "5h", AbbreviateNumbers(512, options))
    assert(AbbreviateLargeNumbers(99, options) == "99")
"#;

#[cfg(feature = "client-retail")]
#[test]
fn player_health_text_uses_abbreviated_large_numbers() {
    crate::common::with_timeout(120, || {
        let env = WowLuaEnv::new().expect("Lua environment");
        crate::common::load_game_startup_inventory(&env);
        env.sync_addon_names_to_lua();
        env.apply_post_load_workarounds();
        wow_ui_sim::startup::fire_startup_events_for_screen(
            &env,
            wow_ui_sim::screen::ScreenKind::Game,
        );
        env.exec(
            r#"
            SetCVar("statusText", "1")
            SetCVar("statusTextDisplay", "NUMERIC")
            local first
            for _, pair in ipairs({ { 1234567, 9876543 }, { 2345678, 8765432 } }) do
                A_Admin.SetPlayerHealth(pair[1], pair[2])
                A_Admin.FireEvent("UNIT_HEALTH", "player")
                A_Admin.FireEvent("UNIT_MAXHEALTH", "player")
                local text = PlayerFrame.PlayerFrameContent.PlayerFrameContentMain.HealthBarsContainer.HealthBarText:GetText()
                assert(text and text ~= "", tostring(text))
                if first then assert(first ~= text, text) end
                first = text
            end
            "#,
        )
        .expect("player health display follows both large health pairs");
    });
}

#[test]
fn abbreviated_numbers_follow_the_breakpoint_rows() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_SharedXML"],
            &[],
            |env, _| {
                env.exec(COST_STRINGS).expect("GetCostString abbreviates");
                env.exec(LARGE_NUMBERS).expect("globals follow their rows");
            },
        );
    });
}
