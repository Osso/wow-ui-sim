#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};

#[test]
fn retail_minimap_refresh_stops_at_unmodeled_visibility_query() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    // Stop at the missing visibility predicate; do not claim full refresh success.
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
        C_Garrison.IsLandingPageMinimapButtonVisible = function(garrisonType)
            visibilityCalls = visibilityCalls + 1
            assert(type(garrisonType) == 'number')
            error('stop at unmodeled C_Garrison.IsLandingPageMinimapButtonVisible')
        end

        local button = setmetatable({
            hidden = false,
            Hide = function(self) self.hidden = true end,
        }, { __index = ExpansionLandingPageMinimapButtonMixin })

        local ok, err = pcall(function() button:RefreshButton() end)
        assert(not ok and tostring(err):find(
            'stop at unmodeled C_Garrison.IsLandingPageMinimapButtonVisible', 1, true
        ))
        assert(visibilityCalls == 1,
            "original minimap refresh must reach the unmodeled visibility query")

        local function resultShape(...)
            return select('#', ...), type((...))
        end
        local returnCount, returnType = resultShape(C_Garrison.GetLandingPageGarrisonType())
        assert(returnCount == 1 and returnType == 'number',
            "C_Garrison.GetLandingPageGarrisonType must return one number")
        "#,
    )
    .expect("Minimap must reach the numeric garrison type query before the known blocker");
}
