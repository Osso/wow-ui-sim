//! 12.0.5 retained contract: secret `%s` ignores width and precision.
//! Tainted formatting success is an inferred opaque operation from the
//! addon-focused notes, not native-verified caller policy. Display consumers
//! such as SetFormattedText need a separate provenance contract.

use rilua::api::state_is_secure;
use rilua::table_security::{is_secret_value, unwrap_secret, wrap_host_secret_string};
use rilua::{LuaApiMut, Val};
use wow_ui_sim::lua_api::WowLuaEnv;

const PAYLOAD: &str = "abcdefgh";

fn inject_host_secret_string() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("simulator environment");
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let state = lua.state_mut();
        let secret = wrap_host_secret_string(state, PAYLOAD);
        // Root before global-name allocation or any subsequent GC safe point.
        state.push(secret);
        let inserted = lua.set_global_val("HostSecretString", secret);
        lua.state_mut().pop();
        inserted.expect("inject host-created secret fixture");
    }
    env.exec("debug.settaintmode(true); collectgarbage('collect'); collectgarbage('collect')")
        .expect("collect while fixture is globally rooted");
    assert_secret_payload(&env, "HostSecretString");
    env
}

fn assert_secret_payload(env: &WowLuaEnv, global_name: &str) {
    // Trusted test boundary only: not exported to Lua, and never clears taint
    // or bypasses the runtime's secure-caller guard to inspect a result.
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let value = lua.get_global_val(global_name);
    let state = lua.state_mut();
    assert!(state_is_secure(state), "inspection requires a secure host");
    assert!(
        is_secret_value(state, value),
        "{global_name}: formatting must retain secrecy, got {}",
        value.type_name()
    );
    let payload = unwrap_secret(state, value).expect("guarded secure inspection");
    let Val::Str(string_ref) = payload else {
        panic!("{global_name}: secret payload must be a string");
    };
    let bytes = state
        .gc
        .string_arena
        .get(string_ref)
        .expect("rooted secret payload survives GC")
        .data();
    assert_eq!(
        bytes,
        PAYLOAD.as_bytes(),
        "{global_name}: full secret payload"
    );
}

#[test]
fn public_strings_keep_precision_and_width_direct_and_positional() {
    let env = WowLuaEnv::new().expect("simulator environment");
    env.exec(
        r#"
        assert(string.format('%.5s', 'abcdefgh') == 'abcde')
        assert(string.format('%8s', 'abc') == '     abc')
        assert(string.format('%2$.5s', 'unused', 'abcdefgh') == 'abcde')
        assert(string.format('%2$8s', 'unused', 'abc') == '     abc')
        "#,
    )
    .expect("public strings retain normal formatting through both wrapper paths");
}

#[test]
fn secret_precision_direct_preserves_full_payload_and_secrecy() {
    let env = inject_host_secret_string();
    env.exec("SecretFormatResult = string.format('%.5s', HostSecretString)")
        .expect("direct secret precision formatting succeeds");
    assert_secret_payload(&env, "SecretFormatResult");
}

#[test]
fn secret_width_direct_does_not_pad_or_declassify() {
    let env = inject_host_secret_string();
    env.exec("SecretFormatResult = string.format('%12s', HostSecretString)")
        .expect("direct secret width formatting succeeds");
    assert_secret_payload(&env, "SecretFormatResult");
}

#[test]
fn secret_positional_formatting_ignores_precision_and_width() {
    let env = inject_host_secret_string();
    env.exec(
        r#"
        SecretPrecisionResult = string.format('%2$.5s', 'unused', HostSecretString)
        SecretWidthResult = string.format('%2$12s', 'unused', HostSecretString)
        "#,
    )
    .expect("positional wrapper formats the selected secret argument");
    assert_secret_payload(&env, "SecretPrecisionResult");
    assert_secret_payload(&env, "SecretWidthResult");
}

fn assert_inferred_tainted_formatting(precision: &str, width: &str) {
    let env = inject_host_secret_string();
    // Policy inference: addon formatting is an allowed opaque operation;
    // addon payload reads remain forbidden. Do not weaken unwrap_secret's
    // production guard to make this expected formatting behavior succeed.
    let source = format!(
        r#"
        assert(debug.getstacktaint() == nil)
        local function addon_format(fmt)
            assert(debug.getstacktaint() == 'SecretFormattingAddon')
            assert(issecretvalue(HostSecretString))
            assert(not pcall(secretunwrap, HostSecretString), 'addon cannot read input')
            local result = string.format(fmt, HostSecretString)
            assert(debug.getstacktaint() == 'SecretFormattingAddon', 'format preserves taint')
            assert(issecretvalue(result), 'opaque formatting retains secrecy')
            assert(not pcall(secretunwrap, result), 'addon cannot declassify output')
            assert(debug.getstacktaint() == 'SecretFormattingAddon', 'rejection preserves taint')
            return result
        end
        debug.setobjecttaint(addon_format, 'SecretFormattingAddon')
        SecretPrecisionResult = addon_format('{precision}')
        assert(debug.getstacktaint() == nil, 'caller taint restored after precision')
        SecretWidthResult = addon_format('{width}')
        assert(debug.getstacktaint() == nil, 'caller taint restored after width')
        collectgarbage('collect')
        "#
    );
    env.exec(&source)
        .expect("inferred addon opaque formatting succeeds without public payload access");
    assert_secret_payload(&env, "SecretPrecisionResult");
    assert_secret_payload(&env, "SecretWidthResult");
}

#[test]
fn inferred_tainted_direct_formatting_preserves_secrecy_and_stack_taint() {
    assert_inferred_tainted_formatting("%.5s", "%12s");
}

#[test]
fn inferred_tainted_positional_formatting_preserves_secrecy_and_stack_taint() {
    assert_inferred_tainted_formatting("%1$.5s", "%1$12s");
}
