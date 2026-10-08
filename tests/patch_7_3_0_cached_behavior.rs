//! Bounded 7.3.0 input transition; no audio-output or historical catalog parity claim.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_7_3_0_cached_sound_ids_reject_names_and_preserve_request(env: &WowLuaEnv) {
    let sound_id: u32 = env
        .eval("return SOUNDKIT.IG_INVENTORY_ROTATE_CHARACTER")
        .expect("numeric sound-kit entry");
    assert_eq!(sound_id, 861);
    env.exec("PlaySound(SOUNDKIT.IG_INVENTORY_ROTATE_CHARACTER)")
        .expect("PlaySound accepts sound-kit IDs");
    assert_eq!(env.state().borrow().last_sound_kit_requested, Some(sound_id));

    let rejected: bool = env
        .eval("return not pcall(PlaySound, 'igInventoryRotateCharacter')")
        .expect("legacy sound-name rejection");
    assert!(rejected, "pre-7.3 string sound names must not be accepted");
    assert_eq!(
        env.state().borrow().last_sound_kit_requested,
        Some(sound_id),
        "rejected string must not replace the last sound-kit request"
    );
    env.exec("PlaySound(839)").expect("second numeric sound-kit ID");
    assert_eq!(env.state().borrow().last_sound_kit_requested, Some(839));
}
}
