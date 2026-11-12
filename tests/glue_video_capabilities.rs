//! R63 verified exact-member publication and Forever no-argument return shape.
//! Inferred byte projections and unknown return tails are not asserted here.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

const METHODS: [&str; 5] = [
    "IsLinearEnabledOnStart",
    "IsOutlineModeSupported",
    "IsPBRWaterSupported",
    "IsSecondaryLightingSupported",
    "IsSpellVisualDensitySystemSupported",
];

const SCREENS: [ScreenKind; 4] = [
    ScreenKind::Game,
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

fn published(index: usize) -> bool {
    ACTIVE != ClientProfile::Retail || index == 4
}

#[test]
fn verified_video_capability_publication_matches_profile_in_game_and_glue() {
    let env = WowLuaEnv::new().expect("create capability registration environment");
    for screen in SCREENS {
        env.set_screen_mode(screen);
        for (index, method) in METHODS.iter().enumerate() {
            let kind: String = env
                .eval(&format!("return type(rawget(C_VideoOptions, '{method}'))"))
                .expect("inspect exact capability member");
            assert_eq!(
                kind,
                if published(index) { "function" } else { "nil" },
                "{ACTIVE:?} {screen:?} {method}"
            );
        }
    }
}

#[cfg(feature = "client-wowforever")]
#[test]
fn verified_forever_video_capabilities_return_one_boolean_in_game_and_glue() {
    let env = WowLuaEnv::new().expect("create Forever capability environment");
    for screen in SCREENS {
        env.set_screen_mode(screen);
        for method in METHODS {
            let (count, kind): (i64, String) = env
                .eval(&format!(
                    "local query = rawget(C_VideoOptions, '{method}'); \
                     return select('#', query()), type(query())"
                ))
                .expect("inspect verified Forever capability return shape");
            assert_eq!(count, 1, "{screen:?} {method}");
            assert_eq!(kind, "boolean", "{screen:?} {method}");
        }
    }
}
