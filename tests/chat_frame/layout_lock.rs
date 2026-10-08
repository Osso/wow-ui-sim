use super::*;

prefork_full_ui_case! {
fn chat_frame_layout_stays_locked(env: &WowLuaEnv) {

        let result: String = env
            .eval(include_str!("layout_lock.lua"))
            .expect("chat frame lock eval failed");

        assert_eq!(
            result, "ok",
            "ChatFrame1 layout should remain fully locked: {result}"
        );

}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn chat_frame_layout_stays_locked() {
    let env = setup_env();
    chat_frame_layout_stays_locked::run(&env);
}
