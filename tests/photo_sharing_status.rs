//! C_PhotoSharing account-link status over A_Admin flags. Moved from
//! patch_12_0_5_sweep_globals.rs: Forever's PhotoSharingUIDocumentation.lua
//! declares GetStatus/ClearAuthorization byte-identically to Retail's.
#![cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create Lua environment")
}

fn drained_event_names(env: &WowLuaEnv) -> Vec<String> {
    let events = env.state().borrow_mut().events.drain();
    events.into_iter().map(|event| event.name).collect()
}

#[test]
fn photo_sharing_status_and_clear_authorization_follow_account_flags() {
    let env = env();
    env.exec(
        r#"
        local status = Enum.PhotoSharingStatus
        assert(C_PhotoSharing.GetStatus() == status.Disabled)
        A_Admin.SetPhotoSharingEnabled(true)
        assert(C_PhotoSharing.GetStatus() == status.NotConfigured)
        A_Admin.SetPhotoSharingAuthorized(true)
        assert(C_PhotoSharing.GetStatus() == status.Configured)
        "#,
    )
    .expect("status derives from the enabled and authorized flags");
    drained_event_names(&env);
    env.exec(
        r#"
        C_PhotoSharing.ClearAuthorization()
        C_PhotoSharing.ClearAuthorization()
        assert(C_PhotoSharing.IsAuthorized() == false)
        assert(C_PhotoSharing.GetStatus() == Enum.PhotoSharingStatus.NotConfigured)
        "#,
    )
    .expect("clearing unlinks the account");
    assert_eq!(
        drained_event_names(&env),
        ["PHOTO_SHARING_AUTHORIZATION_UPDATED"]
    );
}
