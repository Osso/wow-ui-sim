#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};

#[test]
fn retail_minimap_refresh_uses_landing_page_visibility_records() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();

    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FrameXML", "Blizzard_Minimap"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_Minimap"));

    env.exec(
        r#"
        GameRulesUtil = GameRulesUtil or {}
        GameRulesUtil.ShouldShowExpansionLandingPageButton = function() return true end
        ExpansionLandingPage = { IsOverlayApplied = function() return false end }
        FrameUtil.RegisterFrameForEvents = function() end

        MinimapTestButton = setmetatable({
            hidden = false,
            Hide = function(self) self.hidden = true end,
            Show = function(self) self.hidden = false; self.shown = true end,
            UpdateIcon = function(self) self.iconUpdated = true end,
        }, { __index = ExpansionLandingPageMinimapButtonMixin })

        assert(C_Garrison.GetLandingPageGarrisonType() == 0)
        MinimapTestButton:RefreshButton()
        assert(MinimapTestButton.hidden,
            "the original refresh must hide the button when no record matches")
        assert(not MinimapTestButton.shown,
            "a missing landing-page record must not show the button")
        "#,
    )
    .expect("default-state Minimap refresh must complete and hide the button");

    {
        let mut state = env.state().borrow_mut();
        let landing_page = &mut state.garrison_landing_page;
        landing_page.garrison_type = Some(6);
        landing_page.minimap_button_records.insert(6, true);
        landing_page.minimap_button_records.insert(7, false);
    }

    env.exec(
        r#"
        MinimapTestButton:RefreshButton(true)
        assert(MinimapTestButton.shown and not MinimapTestButton.hidden,
            "the original refresh must show the button for a matching true record")
        assert(MinimapTestButton.iconUpdated,
            "the visible path must continue through the original refresh")
        assert(C_Garrison.IsLandingPageMinimapButtonVisible(262),
            "record lookup must use the low byte of the garrison type")
        assert(not C_Garrison.IsLandingPageMinimapButtonVisible(7),
            "a matched non-111 record must return its false state flag")
        assert(not C_Garrison.IsLandingPageMinimapButtonVisible(8),
            "a missing record must return false")
        "#,
    )
    .expect("a matching landing-page record must show the Minimap button");

    {
        let mut state = env.state().borrow_mut();
        let landing_page = &mut state.garrison_landing_page;
        landing_page.minimap_button_records.insert(111, false);
        landing_page.player_state_value = 12;
    }
    env.exec("assert(not C_Garrison.IsLandingPageMinimapButtonVisible(111))")
        .expect("PlayerCondition 105386 must gate type 111");

    {
        let mut state = env.state().borrow_mut();
        let landing_page = &mut state.garrison_landing_page;
        landing_page.player_condition_105386 = true;
        landing_page.player_state_value = 0;
    }
    env.exec("assert(not C_Garrison.IsLandingPageMinimapButtonVisible(111))")
        .expect("type 111 must require a nonzero player-state value");

    {
        let mut state = env.state().borrow_mut();
        let landing_page = &mut state.garrison_landing_page;
        landing_page.player_state_value = 12;
    }
    env.exec("assert(C_Garrison.IsLandingPageMinimapButtonVisible(367))")
        .expect("type 111 must use PlayerCondition and nonzero player state");
}
