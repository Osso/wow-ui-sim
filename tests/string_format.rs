//! WoW-compatible `string.format` extensions, rilua port of the master-era
//! `string_format.rs`. Installed via `crate::lua_api::string_format::patch_string_format`
//! during env init.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[test]
fn plain_format_string_unchanged() {
    let env = env();
    let out: String = env
        .eval(r#"return string.format("%d-%s", 42, "hi")"#)
        .unwrap();
    assert_eq!(out, "42-hi");
}

#[test]
fn uppercase_f_converts_to_lowercase_f() {
    // %F is invalid in plain Lua 5.1; WoW's patched runtime treats it as %f.
    let env = env();
    let out: String = env.eval(r#"return string.format("%.2F", 1.5)"#).unwrap();
    assert_eq!(out, "1.50");
}

#[test]
fn format_global_mirrors_string_format() {
    let env = env();
    let out: String = env.eval(r#"return format("%.1F", 3.25)"#).unwrap();
    assert_eq!(out, "3.2");
}

#[test]
fn positional_args_reorder_output() {
    let env = env();
    let out: String = env
        .eval(r#"return string.format("%2$s then %1$s", "first", "second")"#)
        .unwrap();
    assert_eq!(out, "second then first");
}

#[test]
fn positional_args_with_numeric_specifiers() {
    let env = env();
    let out: String = env
        .eval(r#"return string.format("%2$d / %1$d", 10, 20)"#)
        .unwrap();
    assert_eq!(out, "20 / 10");
}

#[test]
fn mixed_positional_and_sequential_are_both_supported() {
    // Retail LuaJIT lets positional + sequential coexist; keep that shape.
    let env = env();
    let out: String = env
        .eval(r#"return string.format("%s %1$s %s", "a", "b")"#)
        .unwrap();
    // Sequential %s picks args[1] on the first encounter and args[2] on the
    // second — %1$s always picks args[1].
    assert_eq!(out, "a a b");
}

#[test]
fn literal_percent_preserved() {
    let env = env();
    let out: String = env
        .eval(r#"return string.format("100%% of %d", 5)"#)
        .unwrap();
    assert_eq!(out, "100% of 5");
}

#[test]
fn repeated_numeric_specifiers_are_all_substituted() {
    let env = env();
    let out: String = env
        .eval(r#"return string.format("Page %d/%d", 1, 2)"#)
        .unwrap();
    assert_eq!(out, "Page 1/2");
}

#[test]
fn non_string_fmt_delegates_to_original() {
    // When the first arg isn't a string, the patched implementation
    // forwards straight through to the native string.format. Lua 5.1's
    // string.format coerces numbers to their decimal form when used as
    // the format source, so passing 42 yields "42" (no specifiers).
    let env = env();
    let out: String = env.eval(r#"return string.format(42)"#).unwrap();
    assert_eq!(out, "42");
}

#[test]
fn width_and_precision_are_preserved() {
    let env = env();
    let out: String = env
        .eval(r#"return string.format("[%10.3F]", 1.23456)"#)
        .unwrap();
    assert_eq!(out, "[     1.235]");
}

#[test]
fn flag_characters_are_preserved() {
    let env = env();
    let out: String = env
        .eval(r#"return string.format("%-5d|%+d|%05d", 1, 2, 3)"#)
        .unwrap();
    assert_eq!(out, "1    |+2|00003");
}

// Retail 12.1.5 reports a missing positional argument as `no value`
// (VERIFIED R88af.3, docs/specs/wow-lua-stdlib-deviations.md; 8d65b8b3b9,
// 24120fa41d). The other profiles keep the legacy formatter's nil.
#[cfg(not(all(feature = "client-retail", feature = "retail-12-1-5")))]
#[test]
fn positional_index_out_of_range_yields_nil_arg() {
    // args[3] is nil; string.format called with nil for %s prints "nil".
    let env = env();
    let out: String = env
        .eval(r#"return string.format("%3$s", "a", "b")"#)
        .unwrap();
    assert_eq!(out, "nil");
}

#[test]
fn excessive_positional_index_errors() {
    // master rejects indices >= 100 with "invalid format (width or precision too long)".
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            local ok = pcall(string.format, "%100$s", "x")
            return ok
            "#,
        )
        .unwrap();
    assert!(!ok, "index 100 must be rejected");
}

#[test]
fn patched_format_appears_as_a_function() {
    // The rilua port registers the WoW wrapper as a RustFn, so `type(fmt)`
    // is still "function" (same as retail where string.format is a C fn).
    let env = env();
    let kind: String = env.eval(r#"return type(string.format)"#).unwrap();
    assert_eq!(kind, "function");
}
