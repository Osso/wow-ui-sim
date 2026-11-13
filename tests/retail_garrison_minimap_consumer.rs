#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};

#[test]
fn retail_minimap_refresh_uses_the_landing_page_garrison_type() {
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

        local visibilityCalls = 0
        C_Garrison.IsLandingPageMinimapButtonVisible = function()
            visibilityCalls = visibilityCalls + 1
            return false
        end

        local button = setmetatable({
            hidden = false,
            Hide = function(self) self.hidden = true end,
        }, { __index = ExpansionLandingPageMinimapButtonMixin })

        button:RefreshButton()
        assert(visibilityCalls == 1, "original minimap refresh must query landing-page visibility")
        assert(button.hidden, "the original consumer must keep a hidden garrison button hidden")
        "#,
    )
    .expect("original Blizzard Minimap refresh must query the landing-page garrison type");
}
