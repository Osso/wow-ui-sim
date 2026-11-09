use super::load_game_ui_without_player_choice;

/// Proves PTR keeps the UI-parent positioning entry point and applies its supplied inputs.
#[test]
fn ui_parent_position_updates_for_debug_bar_and_notch() {
    let env = load_game_ui_without_player_choice();

    let types: (String, String, String) = env
        .eval(
            r#"
            return type(UpdateUIParentPosition),
                type(GetNotchHeight),
                type(UIDoFramesIntersect)
            "#,
        )
        .expect("UI parent position publication probe succeeds");
    assert_eq!(types, ("function".into(), "nil".into(), "nil".into()));

    let (plain_offset, debug_offset, notch_offset): (f64, f64, f64) = env
        .eval(
            r#"
            local originalShouldAvoidNotch = C_UI.ShouldUIParentAvoidNotch
            local originalSafeRegion = C_UI.GetTopLeftNotchSafeRegion
            local originalPhysicalSize = GetPhysicalScreenSize
            local originalGetHeight = UIParent.GetHeight
            local originalDebugHeight = DebugBarManager.GetTotalHeight
            local function currentOffset()
                return -select(5, UIParent:GetPoint(1))
            end

            C_UI.ShouldUIParentAvoidNotch = function() return false end
            DebugBarManager.GetTotalHeight = function() return 40 end
            UpdateUIParentPosition()
            local plainOffset = currentOffset()

            C_UI.ShouldUIParentAvoidNotch = function() return true end
            C_UI.GetTopLeftNotchSafeRegion = function() return 0, 0, 0, 120 end
            GetPhysicalScreenSize = function() return 1920, 1080 end
            UIParent.GetHeight = function() return 720 end
            DebugBarManager.GetTotalHeight = function() return 100 end
            UpdateUIParentPosition()
            local debugOffset = currentOffset()

            DebugBarManager.GetTotalHeight = function() return 40 end
            UpdateUIParentPosition()
            local notchOffset = currentOffset()

            C_UI.ShouldUIParentAvoidNotch = originalShouldAvoidNotch
            C_UI.GetTopLeftNotchSafeRegion = originalSafeRegion
            GetPhysicalScreenSize = originalPhysicalSize
            UIParent.GetHeight = originalGetHeight
            DebugBarManager.GetTotalHeight = originalDebugHeight
            return plainOffset, debugOffset, notchOffset
            "#,
        )
        .expect("UpdateUIParentPosition should apply supplied offsets");

    assert_eq!(plain_offset, 40.0);
    assert_eq!(debug_offset, 100.0);
    assert_eq!(notch_offset, 80.0);
}
