//! Sandbox-parity probes for dangerous globals.
//!
//! Pins the behavioural expectation for five classically-restricted
//! globals, checked separately against the insecure `_G` surface and
//! the secure `__secureenv` surface:
//!
//! - `dofile`, `loadfile`, `require` — filesystem / module loaders.
//! - `string.dump` — bytecode dumper (can side-step fenv/taint).
//! - `math.randomseed` — mutates a process-global RNG.
//!
//! Policy split (matches Blizzard's restricted-execution model):
//!
//! - Insecure addon code reads globals through `_G`. These five MUST
//!   be `nil` there so addons cannot invoke them.
//! - Secure chunks (audited Blizzard code) run with fenv retargeted to
//!   `__secureenv`, which is shallow-copied from `_G` BEFORE the
//!   cleanup. `__secureenv` therefore retains the five entries so
//!   secure bootstrap machinery can still use them.
//!
//! `env_init::remove_sandbox_globals` enforces this split; the tests
//! below pin both sides so nobody accidentally re-orders or widens
//! the cleanup.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

fn probe(env: &WowLuaEnv, name: &str) -> (String, String) {
    let script = format!(
        r#"
        return type(rawget(_G, "{name}")),
               type(rawget(__secureenv, "{name}"))
        "#
    );
    env.eval::<(String, String)>(&script)
        .expect("probe eval should succeed")
}

#[test]
fn dofile_nil_on_g_retained_on_secureenv() {
    let env = env();
    let (g, secure) = probe(&env, "dofile");
    assert_eq!(g, "nil", "dofile must be nil on _G");
    assert_eq!(
        secure, "function",
        "dofile must remain on __secureenv for secure chunks"
    );
}

#[test]
fn loadfile_nil_on_g_retained_on_secureenv() {
    let env = env();
    let (g, secure) = probe(&env, "loadfile");
    assert_eq!(g, "nil", "loadfile must be nil on _G");
    assert_eq!(
        secure, "function",
        "loadfile must remain on __secureenv for secure chunks"
    );
}

#[cfg(not(feature = "client-wowforever"))]
#[test]
fn require_nil_on_g_retained_on_secureenv() {
    let env = env();
    let (g, secure) = probe(&env, "require");
    assert_eq!(g, "nil", "require must be nil on _G");
    assert_eq!(
        secure, "function",
        "require must remain on __secureenv for secure chunks"
    );
}

#[cfg(feature = "client-wowforever")]
#[test]
fn require_is_module_lookup_on_both_forever_environments() {
    let env = env();
    let (g, secure) = probe(&env, "require");
    assert_eq!(g, "function");
    assert_eq!(secure, "function");
    let (public_ok, secure_ok): (bool, bool) = env
        .eval("local public = pcall(require, 'os'); local secure = pcall(__secureenv.require, 'os'); return public, secure")
        .unwrap();
    assert!(!public_ok, "Forever does not expose host package loading");
    assert!(!secure_ok, "secure Forever imports use the same addon registry");
}

#[test]
fn string_dump_nil_on_g_retained_on_secureenv() {
    let env = env();
    let (g, secure): (String, String) = env
        .eval(
            r#"
            local g_string = rawget(_G, "string")
            local secure_string = rawget(__secureenv, "string")
            return type(g_string and g_string.dump),
                   type(secure_string and secure_string.dump)
            "#,
        )
        .unwrap();
    assert_eq!(g, "nil", "string.dump must be nil on _G.string");
    assert_eq!(
        secure, "function",
        "string.dump must remain on __secureenv.string"
    );
}

#[test]
fn dynamic_string_methods_use_sandboxed_string_table() {
    let env = env();
    env.restore_post_cleanup_globals();
    env.eval::<()>(
        r#"
        local stringMeta = debug.getmetatable("")
        stringMeta.__index = {}
        "#,
    )
    .unwrap();
    env.sync_string_metatable_to_global_string();
    let (same_index, result): (bool, String) = env
        .eval(
            r#"
            string.TEST_DYNAMIC_METHOD = function(str)
                return str .. ":ok"
            end
            return debug.getmetatable("").__index == string,
                   ("probe"):TEST_DYNAMIC_METHOD()
            "#,
        )
        .unwrap();

    assert!(
        same_index,
        "string method lookup must read the sandboxed _G.string table"
    );
    assert_eq!(result, "probe:ok");
}

#[test]
fn math_randomseed_nil_on_g_retained_on_secureenv() {
    let env = env();
    let (g, secure): (String, String) = env
        .eval(
            r#"
            local g_math = rawget(_G, "math")
            local secure_math = rawget(__secureenv, "math")
            return type(g_math and g_math.randomseed),
                   type(secure_math and secure_math.randomseed)
            "#,
        )
        .unwrap();
    assert_eq!(g, "nil", "math.randomseed must be nil on _G.math");
    assert_eq!(
        secure, "function",
        "math.randomseed must remain on __secureenv.math"
    );
}
