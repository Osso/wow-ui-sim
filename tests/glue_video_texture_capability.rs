//! R60 verified publication coverage. The high-resolution return convention
//! and fixed capability value are inferred and are not asserted here.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

#[test]
fn high_res_texture_capability_publication_matches_profile_in_game_and_glue() {
    let env = WowLuaEnv::new().expect("create texture capability environment");
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        env.set_screen_mode(screen);
        let expected_type = if ACTIVE == ClientProfile::Retail {
            "nil"
        } else {
            "function"
        };
        let actual_type: String = env
            .eval("return type(rawget(C_VideoOptions, 'AreHighResTexturesAvailable'))")
            .expect("inspect raw texture capability registration");
        assert_eq!(actual_type, expected_type, "{ACTIVE:?} {screen:?}");
    }
}
