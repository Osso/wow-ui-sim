//! Behavior pin: successful action-bar transitions refresh bars then relayout UIParent.

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ActionBarController";

#[test]
fn validate_transition_successful_paths_update_multibars_then_relayout_uiparent() {
    test_timeout! {
    with_blizzard_addon_smoke_shape(
        &[
            "Blizzard_ActionBar",
            "Blizzard_OverrideActionBar",
            ROOT,
        ],
        &[],
        |env, _loaded| {
        env.exec(
            r#"
            _G.transitionRelayoutLog = {}
            _G.transitionRelayoutCases = {}

            function Settings.GetValue()
                return true
            end

            local function logMultiActionBarUpdate()
                if not _G.transitionRelayoutMultiLogged then
                    _G.transitionRelayoutMultiLogged = true
                    table.insert(_G.transitionRelayoutLog, "multi")
                end
            end

            for _, bar in ipairs({
                MultiBarBottomLeft,
                MultiBarBottomRight,
                MultiBarRight,
                MultiBarLeft,
                MultiBar5,
                MultiBar6,
                MultiBar7,
            }) do
                function bar:SetShown()
                    logMultiActionBarUpdate()
                end
            end

            function ManageFramePositions()
                table.insert(_G.transitionRelayoutLog, "uiparent")
            end

            function StanceBar:ShouldShow()
                return false
            end

            function MicroMenu:ResetMicroMenuPosition()
            end

            function OverrideActionBar.slideOut:Play()
            end

            function _G.runTransitionRelayoutCase(caseName)
                _G.transitionRelayoutMultiLogged = false
                local startIndex = #_G.transitionRelayoutLog + 1
                ValidateActionBarTransition()
                local callCount = #_G.transitionRelayoutLog - startIndex + 1
                _G.transitionRelayoutCases[caseName] = {
                    _G.transitionRelayoutLog[startIndex + callCount - 2],
                    _G.transitionRelayoutLog[startIndex + callCount - 1],
                    callCount,
                }
            end

            MainActionBar:Hide()
            StanceBar:Hide()
            OverrideActionBar:Hide()
            _G.runTransitionRelayoutCase("mainHiddenOverride")
            "#,
        )
        .expect("main-state relayout probe must run cleanly");

        {
            let mut state = env.state().borrow_mut();
            state.has_override_action_bar = true;
            state.override_bar_skin = Some(1);
        }

        env.exec(
            r#"
            function OverrideActionBar:UpdateSkin()
            end

            ActionBarController_UpdateAll()
            _G.transitionRelayoutLog = {}

            MainActionBar:Show()
            StanceBar:Show()
            OverrideActionBar:Hide()
            _G.runTransitionRelayoutCase("overrideHiddenOverride")
            "#,
        )
        .expect("override-state relayout probe must run cleanly");

        {
            let mut state = env.state().borrow_mut();
            state.has_override_action_bar = false;
            state.override_bar_skin = None;
        }

        env.exec(
            r#"
            ActionBarController_UpdateAll()
            _G.transitionRelayoutLog = {}

            MainActionBar:Hide()
            StanceBar:Hide()
            OverrideActionBar:Show()
            _G.runTransitionRelayoutCase("mainShownOverride")
            "#,
        )
        .expect("main-state visible-override relayout probe must run cleanly");

        let (
            main_hidden_multi,
            main_hidden_uiparent,
            main_hidden_count,
            override_multi,
            override_uiparent,
            override_count,
            main_shown_multi,
            main_shown_uiparent,
            main_shown_count,
        ): (
            String,
            String,
            i32,
            String,
            String,
            i32,
            String,
            String,
            i32,
        ) = env
            .eval(
                r#"
                local mainHidden = _G.transitionRelayoutCases.mainHiddenOverride
                local override = _G.transitionRelayoutCases.overrideHiddenOverride
                local mainShown = _G.transitionRelayoutCases.mainShownOverride
                return mainHidden[1], mainHidden[2], mainHidden[3],
                    override[1], override[2], override[3],
                    mainShown[1], mainShown[2], mainShown[3]
                "#,
            )
            .expect("post transition relayout probe must run cleanly");

        assert_transition_relayout(
            &main_hidden_multi,
            &main_hidden_uiparent,
            main_hidden_count,
            "main state with hidden override",
        );
        assert_transition_relayout(
            &override_multi,
            &override_uiparent,
            override_count,
            "override state with hidden override",
        );
        assert_transition_relayout(
            &main_shown_multi,
            &main_shown_uiparent,
            main_shown_count,
            "main state with visible override",
        );
        },
    );
    }
}

fn assert_transition_relayout(
    first_call: &str,
    second_call: &str,
    call_count: i32,
    case_name: &str,
) {
    assert_eq!(
        first_call, "multi",
        "{case_name} must call MultiActionBar_Update before relayout"
    );
    assert_eq!(
        second_call, "uiparent",
        "{case_name} must call ManageFramePositions after MultiActionBar_Update"
    );
    assert!(
        call_count >= 2,
        "{case_name} must include the two terminal relayout calls"
    );
}
