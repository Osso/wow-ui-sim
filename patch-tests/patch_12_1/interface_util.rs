use super::load_game_ui_without_player_choice;

/// Proves screen-scale helpers use the pinned PTR InterfaceUtil namespace.
#[test]
fn screen_scale_helpers_match_ptr_interface_util_publications() {
    let env = load_game_ui_without_player_choice();

    let (global_height, global_width, height_type, width_type, height, width): (
        String,
        String,
        String,
        String,
        f64,
        f64,
    ) = env
        .eval(
            r#"
            return type(GetScreenHeightScale),
                type(GetScreenWidthScale),
                type(InterfaceUtil.GetScreenHeightScale),
                type(InterfaceUtil.GetScreenWidthScale),
                InterfaceUtil.GetScreenHeightScale(),
                InterfaceUtil.GetScreenWidthScale()
            "#,
        )
        .expect("InterfaceUtil screen scale probe succeeds");

    assert_eq!(global_height, "nil");
    assert_eq!(global_width, "nil");
    assert_eq!(height_type, "function");
    assert_eq!(width_type, "function");
    assert!((height - 1.0).abs() < f64::EPSILON);
    assert!((width - 1.0).abs() < f64::EPSILON);
}
