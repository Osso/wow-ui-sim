use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(not(feature = "retail-12-1-0"))]
#[test]
fn ui_parent_manage_frame_positions_is_callable_noop() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            UIParent_ManageFramePositions()
            return type(UIParent_ManageFramePositions)
            "#,
        )
        .unwrap();

    assert_eq!(result, "function");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn ui_parent_manage_frame_positions_is_absent_in_retail_12_1() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval("return type(UIParent_ManageFramePositions)")
        .unwrap();
    assert_eq!(result, "nil");
}
