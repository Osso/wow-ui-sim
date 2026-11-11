//! R63 verified exact-member publication, followed by explicitly supplied host
//! byte tests of INFERRED projections. No default boolean is a native assertion.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use wow_ui_sim::c_api::video_capabilities::VideoCapabilityBytes;
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

fn host_bytes(values: [u8; 5]) -> VideoCapabilityBytes {
    VideoCapabilityBytes {
        linear_enabled_on_start: values[0],
        outline_disabled: values[1],
        pbr_water_supported: values[2],
        secondary_lighting_supported: values[3],
        spell_visual_density_supported: values[4],
    }
}

fn assert_host_projection(env: &WowLuaEnv, values: [u8; 5]) {
    let expected = [
        values[0] != 0,
        values[1] == 0,
        values[2] != 0,
        values[3] != 0,
        values[4] != 0,
    ];
    let pending = env.state().borrow().events.pending().len();
    for (index, method) in METHODS.iter().enumerate() {
        if !published(index) {
            continue;
        }
        let (count, kind, value): (i64, String, bool) = env
            .eval(&format!(
                "local query = rawget(C_VideoOptions, '{method}'); \
                 return select('#', query()), type(query()), query()"
            ))
            .expect("project explicitly supplied host capability byte");
        assert_eq!(count, 1, "INFERRED host return count: {method}");
        assert_eq!(kind, "boolean", "INFERRED host return type: {method}");
        assert_eq!(value, expected[index], "INFERRED host projection: {method}");
    }
    assert_eq!(
        env.state().borrow().video_capability_bytes,
        host_bytes(values)
    );
    assert_eq!(env.state().borrow().events.pending().len(), pending);
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

#[test]
fn host_supplied_capability_bytes_project_independently_in_game_and_glue() {
    let env = WowLuaEnv::new().expect("create host capability environment");
    // Non-unit bytes distinguish a nonzero projection from an equality-to-one
    // shortcut. The outline field is an inverted disabled byte.
    let cases = [
        [0, 1, 0, 0, 0],
        [2, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 1, 128, 0, 0],
        [0, 1, 0, 255, 0],
        [0, 1, 0, 0, 3],
        [2, 0, 128, 255, 3],
        [0, 255, 0, 0, 0],
    ];
    for screen in SCREENS {
        env.set_screen_mode(screen);
        for values in cases {
            env.state().borrow_mut().video_capability_bytes = host_bytes(values);
            assert_host_projection(&env, values);
        }
    }
}

#[test]
fn host_capability_bytes_are_environment_local_and_not_cvar_or_adapter_inputs() {
    let env = WowLuaEnv::new().expect("create first host capability environment");
    let other = WowLuaEnv::new().expect("create second host capability environment");
    let supplied = [2, 0, 128, 255, 3];
    let other_supplied = [0, 1, 0, 0, 0];
    env.state().borrow_mut().video_capability_bytes = host_bytes(supplied);
    other.state().borrow_mut().video_capability_bytes = host_bytes(other_supplied);
    // Ordinary host CVar input, not a capability discovery mechanism.
    for (name, value) in [
        ("linear", "0"),
        ("graphicsOutlineMode", "0"),
        ("graphicsPBRLiquidDetail", "0"),
        ("graphicsLightMode", "0"),
        ("graphicsSpellDensity", "0"),
        ("useHighResTextures", "1"),
    ] {
        env.state().borrow_mut().cvars.set(name, value);
    }
    env.exec(
        r#"
        C_VideoOptions._state.gxAdapterInfo = {
            { name = 'Unrelated host adapter fixture', index = 99 },
        }
        C_VideoOptions.SetGameWindowSize(640, 480)
        "#,
    )
    .expect("change independent temporary display inputs");
    assert_host_projection(&env, supplied);
    assert_host_projection(&other, other_supplied);
}
