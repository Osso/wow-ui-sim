//! Bare-environment sound request model; cached Blizzard alias coverage is separate.
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_7_3_0_sound_namespace_and_global_share_request_state() {
    let env = WowLuaEnv::new().expect("bare environment");
    env.exec("C_Sound.PlaySound(861)")
        .expect("namespace accepts numeric sound-kit ID");
    assert_eq!(env.state().borrow().last_sound_kit_requested, Some(861));
    env.exec("PlaySound(839)")
        .expect("legacy global accepts numeric sound-kit ID");
    assert_eq!(env.state().borrow().last_sound_kit_requested, Some(839));
    for function in ["PlaySound", "C_Sound.PlaySound"] {
        assert!(
            env.exec(&format!("{function}('igInventoryRotateCharacter')"))
                .is_err(),
            "{function} must reject legacy sound names"
        );
        assert_eq!(env.state().borrow().last_sound_kit_requested, Some(839));
    }
}
