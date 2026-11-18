#[test]
fn loaded_global_table_owners_publish_tables() {
    // VERIFIED — the Retail panel fixture loads the original table owners.
    // UIParentPanelManager.lua:21 replaces UISpecialFrames after bootstrap.
    let env = crate::common::panel_fixtures::setup_env();
    let (static_popups, panel_windows, soundkit, special_frames): (String, String, String, String) =
        env.eval(
            r#"
            return type(StaticPopupDialogs),
                   type(UIPanelWindows),
                   type(SOUNDKIT),
                   type(UISpecialFrames)
            "#,
        )
        .unwrap();

    assert_eq!(static_popups, "table");
    assert_eq!(panel_windows, "table");
    assert_eq!(soundkit, "table");
    assert_eq!(special_frames, "table");
}

#[test]
fn global_special_frame_bootstrap_aliases_share_identity() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().expect("Failed to create Lua environment");
    let (special_frames, special_frames_alias): (String, bool) = env
        .eval(
            r#"
            return type(UISpecialFrames), UISpecialFrames == UI_SPECIAL_FRAMES
            "#,
        )
        .unwrap();

    assert_eq!(special_frames, "table");
    assert!(
        special_frames_alias,
        "UI_SPECIAL_FRAMES should alias the UI special-frame registry"
    );
}
