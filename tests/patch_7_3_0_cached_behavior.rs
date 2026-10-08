//! Bounded 7.3.0 input transition; no audio-output or historical catalog parity claim.
#![cfg(feature = "client-retail")]

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_7_3_0_cached_sound_ids_reject_names_and_preserve_request(env: &WowLuaEnv) {
    let lua_sound_id: i32 = env
        .eval("return SOUNDKIT.IG_INVENTORY_ROTATE_CHARACTER")
        .expect("numeric sound-kit entry");
    assert_eq!(lua_sound_id, 861);
    let sound_id = u32::try_from(lua_sound_id).expect("positive sound-kit ID");
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

prefork_full_ui_case! {
fn patch_7_3_0_cached_table_inspector_focuses_concrete_tables(env: &WowLuaEnv) {
    let addons = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    );
    load_addon(&env.loader_env(), &addons.join("Blizzard_DebugTools/Blizzard_DebugTools.toc"))
        .expect("load actual Blizzard debug tools");
    let focused: bool = env
        .eval(r#"
            local root = { p730Field = 17, child = { leaf = 23 } }
            local inspector = DisplayTableInspectorWindow(root, "Patch 7.3.0 fixture")
            assert(inspector:IsShown())
            assert(inspector.focusedTable == root)
            inspector:SelectTable(root.child, "child")
            assert(inspector.focusedTable == root.child)
            inspector:NavigateBackward()
            assert(inspector.focusedTable == root)
            inspector:Hide()
            return not inspector:IsShown()
        "#)
        .expect("real inspector window focus, navigation and close lifecycle");
    assert!(focused);
}
}
