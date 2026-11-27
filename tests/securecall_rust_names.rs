//! `securecall` is the Rust primitive behind a Rust origin-recording wrapper.
//! R88f.D.32 (Retail 12.1.5.70688): a string or number first argument is
//! converted to a string and looked up as a global before the call.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const NAMED_AND_DIRECT_CALLS: &str = r#"
    assert(securecall("tostring", 42) == "42")
    assert(securecall("type", {}) == "table")
    assert(securecall(math.floor, 2.5) == 2)
    assert(securecall(math.ceil, 2.5) == 3)
    local first, second = securecall(strsplit, ",", "a,b")
    assert(first == "a" and second == "b")
"#;

#[test]
fn securecall_resolves_names_and_returns_the_callee_results() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(NAMED_AND_DIRECT_CALLS)
        .expect("securecall follows its inputs");
}
