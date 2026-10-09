//! Current retail factory publication only, not cached Game/native historical parity.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_2_4_2_factory_publication() {
    let env = WowLuaEnv::new().unwrap();
    let register = include_str!("../data/patch-api/sources/2.4.2-wikitext-register.json");
    sweep::run_publication_sweep(
        &env,
        &sweep::SweepSpec {
            register,
            known_gaps: include_str!("data/patch_2_4_2_factory_known_gaps.json"),
            row_count: serde_json::from_str::<serde_json::Value>(register).unwrap()["entries"]
                .as_array()
                .unwrap()
                .len(),
            register_env: "P242_SWEEP_REGISTER",
            out_env: "P242_SWEEP_OUT",
            later_registers: &[
                include_str!("../data/patch-api/sources/3.2.0-wikitext-register.json"),
                include_str!("../data/patch-api/sources/3.3.0-wikitext-register.json"),
                include_str!("../data/patch-api/sources/3.3.3-wikitext-register.json"),
                include_str!("../data/patch-api/sources/3.3.5-wikitext-register.json"),
                include_str!("../data/patch-api/sources/4.0.1-wikitext-register.json"),
            ],
        },
    );
}

#[test]
fn patch_2_4_2_current_currency_separator_model() {
    let env = WowLuaEnv::new().unwrap();
    // Current namespace is a real amount decomposition; no legacy alias is installed.
    let result: String = env
        .eval(
            r#"return C_CurrencyInfo.GetCoinText(12345, ' / ') .. ';' ..
                C_CurrencyInfo.GetCoinText(10101, ':') .. ';' ..
                C_CurrencyInfo.GetCoinText(0, '-')"#,
        )
        .unwrap();
    assert_eq!(
        result,
        "1 Gold / 23 Silver / 45 Copper;1 Gold:1 Silver:1 Copper;0 Copper"
    );
}
