//! Direct unchanged historical deprecation Lua against current headless Era state.
//! Not proof of full addon loading, native numeric values, or C_Seasons state.

use sha2::{Digest, Sha256};
use wow_ui_sim::lua_api::WowLuaEnv;

const DEPRECATED: &str = include_str!(
    "../data/patch-api/evidence/1.15.1-session-2026-10-09/external-primary-source/Deprecated_1_15_1.lua"
);

#[test]
fn official_public_build_deprecation_preserves_numeric_season_alias() {
    assert_eq!(
        format!("{:x}", Sha256::digest(DEPRECATED.as_bytes())),
        "aa34e2aa7b244efde499f8eb4767cb3b29bc7cbb4a583947c90996256ce512b3",
        "unchanged main-pinned Blizzard Lua at Gethe commit 967711a33ee9d3db2e7262e0bc0b49f4ef5a0013"
    );
    let env = WowLuaEnv::new().expect("headless Era environment initializes");
    let initial: (bool, String, f64, bool) = env
        .eval(
            r#"
            local season = rawget(Enum, "SeasonID")
            local discovery = rawget(season, "SeasonOfDiscovery")
            return IsPublicBuild(), type(discovery), discovery,
                rawget(season, "Placeholder") == nil
            "#,
        )
        .expect("read actual initialized public-build and enum state");
    assert_eq!(initial, (true, "number".to_string(), 2.0, true));

    env.eval::<()>(DEPRECATED)
        .expect("unchanged official deprecation executes without synthetic flags");
    let aliases: (String, String, f64, f64, bool) = env
        .eval(
            r#"
            local season = rawget(Enum, "SeasonID")
            local discovery = rawget(season, "SeasonOfDiscovery")
            local placeholder = rawget(season, "Placeholder")
            return type(discovery), type(placeholder), discovery, placeholder,
                discovery == placeholder
            "#,
        )
        .expect("read raw numeric named values after official deprecation");
    assert_eq!(
        aliases,
        ("number".to_string(), "number".to_string(), 2.0, 2.0, true),
        "existing simulator numeric value is retained; equality cannot be nil == nil"
    );
}
