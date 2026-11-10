use super::load_game_ui_without_player_choice;

/// Proves cursor helpers use the pinned PTR InputUtil API and its retained wrappers.
#[test]
fn cursor_helpers_match_ptr_input_util_publications() {
    let env = load_game_ui_without_player_choice();

    let publications_match: bool = env
        .eval(
            r#"
            return type(CursorOnUpdate) == "function"
                and type(CursorUpdate) == "function"
                and type(ShowInspectCursor) == "function"
                and GetScaledCursorDelta == nil
                and GetScaledCursorPosition == nil
                and GetScaledCursorPositionForFrame == nil
                and MouseIsOver == nil
                and type(InputUtil.CursorOnUpdate) == "function"
                and type(InputUtil.CursorUpdate) == "function"
                and type(InputUtil.GetCursorPosition) == "function"
                and type(InputUtil.GetAnchorPositionAtCursor) == "function"
                and type(InputUtil.GetCursorDelta) == "function"
                and type(InputUtil.IsMouseOver) == "function"
                and type(InputUtil.ShowInspectCursor) == "function"
            "#,
        )
        .expect("input utility publication probe succeeds");
    assert!(publications_match);

    let (frame_x, frame_y, delta_x, delta_y): (f64, f64, f64, f64) = env
        .eval(
            r#"
            GetCursorPosition = function() return 100, 200 end
            GetCursorDelta = function() return 10, 20 end
            UIParent:SetScale(2)
            local frameX, frameY = InputUtil.GetCursorPosition(UIParent)
            local deltaX, deltaY = InputUtil.GetCursorDelta(UIParent)
            return frameX, frameY, deltaX, deltaY
            "#,
        )
        .expect("scaled InputUtil coordinate probe succeeds");

    assert_eq!(
        (frame_x, frame_y, delta_x, delta_y),
        (50.0, 100.0, 5.0, 10.0)
    );

    let (mouse_result, cursor): (bool, String) = env
        .eval(
            r#"
            local region = CreateFrame("Frame")
            region.IsMouseOver = function(_, top, bottom, left, right)
                return top == 1 and bottom == 2 and left == 3 and right == 4
            end
            local mouseResult = InputUtil.IsMouseOver(region, 1, 2, 3, 4)
            local cursor
            SetCursor = function(value) cursor = value end
            ShowInspectCursor()
            return mouseResult, cursor
            "#,
        )
        .expect("InputUtil mouse and inspect cursor probes succeed");

    assert!(mouse_result);
    assert_eq!(cursor, "INSPECT_CURSOR");
}
