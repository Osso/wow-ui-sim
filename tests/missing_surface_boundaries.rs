use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn global_placeholder_tables_are_available_and_share_ui_special_frames_identity() {
    let env = WowLuaEnv::new().expect("WowLuaEnv init");
    let (static_popups, panel_windows, soundkit, special_frames, special_frames_alias):
        (String, String, String, String, bool) = env
        .eval(
            r#"
            return type(StaticPopupDialogs),
                   type(UIPanelWindows),
                   type(SOUNDKIT),
                   type(UISpecialFrames),
                   UISpecialFrames == UI_SPECIAL_FRAMES
            "#,
        )
        .unwrap();

    assert_eq!(static_popups, "table");
    assert_eq!(panel_windows, "table");
    assert_eq!(soundkit, "table");
    assert_eq!(special_frames, "table");
    assert!(
        special_frames_alias,
        "UI_SPECIAL_FRAMES should alias the UI special-frame registry"
    );
}
