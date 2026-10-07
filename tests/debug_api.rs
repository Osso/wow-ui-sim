#![cfg(not(feature = "retail-12-0-0"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn debug_window_methods_append_rendered_lines_to_console_output() {
    let env = env();
    env.exec(
        r#"
        C_Debug.PrintToDebugWindow("alpha")
        C_Debug.ViewInDebugWindow("beta", 42, true, nil)
        "#,
    )
    .unwrap();

    let output = &env.state().borrow().console_output;
    assert_eq!(output.len(), 2, "debug output should append two lines");
    assert_eq!(output[0], "alpha");
    assert_eq!(output[1], "beta\t42\ttrue\tnil");
}
