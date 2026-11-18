//! temporary: master `4f073e8ea` `string.format` patch, kept unchanged for every
//! profile except Retail 12.1.5 (decision A). Those profiles have no R88ag or Evidence: pr46-49-decomp-answers.md, sections R88ag.
//! R88af.3 evidence; their native formatter behavior is UNKNOWN. Retail 12.1.5 Evidence: pr46-49-decomp-answers.md, sections R88af.3.
//! uses `string_format`. Retire per profile once native evidence exists.
//!
//! Original header: WoW-compatible `string.format` patch — rilua port of the master-era
//! `src/lua_api/string_format.rs`.
//!
//! Handles the two dialect extensions real WoW's patched LuaJIT ships:
//! - `%F` (uppercase float) — plain Lua 5.1 would error on this; we
//!   rewrite to `%f` before delegating to the underlying `string.format`.
//! - Positional arguments (`%1$s`, `%2$d`, …) — the Nth arg (1-based)
//!   is spliced in place; mixing positional and sequential specifiers in
//!   the same format string is supported (matches retail behaviour).
//!
//! Being a Rust RustFn, the patched `format` appears as a C function to
//! Lua's `coroutine.create` — matching WoW's real runtime where
//! `string.format` is a builtin C function, not a Lua-defined wrapper.

use crate::lua_api::methods::{
    call_function_state, create_string_bytes, registry_get, registry_set,
};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

const ORIG_REGISTRY_KEY: &str = "__original_string_format";

/// Install the WoW `string.format` patch on the rilua VM.
///
/// Replaces `string.format` and the `format` global with a RustFn wrapper
/// that handles `%F` and positional args before delegating to the
/// original implementation (stashed under a registry key).
pub fn patch_string_format(lua: &mut rilua::Lua) -> LuaResult<()> {
    let existing = registry_get(lua.state_mut(), ORIG_REGISTRY_KEY);
    if !matches!(existing, Val::Function(_)) {
        let original = read_string_format(lua)?;
        registry_set(lua.state_mut(), ORIG_REGISTRY_KEY, original);
    }

    LuaApiMut::register_function(lua, "format", wow_string_format)?;
    install_on_string_table(lua)?;
    Ok(())
}

/// Fetch the current `string.format` function so we can delegate to it.
fn read_string_format(lua: &mut rilua::Lua) -> LuaResult<Val> {
    let state = lua.state_mut();
    let global = state.global;
    let string_key = state.gc.intern_string_static(b"string");
    let string_tbl = state
        .gc
        .tables
        .get(global)
        .map(|t| t.get_str(string_key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    let Val::Table(string_ref) = string_tbl else {
        return Err(runtime_error("string table missing"));
    };
    let format_key = state.gc.intern_string_static(b"format");
    let format_val = state
        .gc
        .tables
        .get(string_ref)
        .map(|t| t.get_str(format_key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    if !matches!(format_val, Val::Function(_)) {
        return Err(runtime_error("string.format missing"));
    }
    Ok(format_val)
}

/// Write our RustFn into `string.format`.
fn install_on_string_table(lua: &mut rilua::Lua) -> LuaResult<()> {
    let format_val = LuaApiMut::get_global_val(lua, "format");
    if !matches!(format_val, Val::Function(_)) {
        return Err(runtime_error("format global missing after registration"));
    }
    let state = lua.state_mut();
    let global = state.global;
    let string_key = state.gc.intern_string_static(b"string");
    let Val::Table(string_ref) = state
        .gc
        .tables
        .get(global)
        .map(|t| t.get_str(string_key, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
    else {
        return Err(runtime_error("string table missing"));
    };
    let format_key = state.gc.intern_string_static(b"format");
    if let Some(tbl) = state.gc.tables.get_mut(string_ref) {
        let _ = tbl.raw_set(Val::Str(format_key), format_val, &state.gc.string_arena);
    }
    state.gc.barrier_back(string_ref);
    Ok(())
}

/// Rust implementation of WoW's extended `string.format`.
fn wow_string_format(state: &mut LuaState) -> LuaResult<u32> {
    let original = registry_get(state, ORIG_REGISTRY_KEY);
    if !matches!(original, Val::Function(_)) {
        return Err(runtime_error("__original_string_format missing"));
    }
    let args = read_stack_args(state);

    // Non-string first arg: pass through.
    let Some(Val::Str(first)) = args.first().copied() else {
        return delegate(state, original, &args);
    };
    let fmt = match read_string(state, first) {
        Some(s) => s,
        None => return delegate(state, original, &args),
    };

    validate_required_string_arg(&fmt, &args)?;

    // Fast path: plain format string.
    if !fmt.contains(&b'F') && !fmt.contains(&b'$') {
        let normalized_args = normalize_nil_numeric_args(&fmt, &args);
        return delegate(state, original, &normalized_args);
    }

    delegate_rewritten_format(state, original, &fmt, &args)
}

fn delegate_rewritten_format(
    state: &mut LuaState,
    original: Val,
    format: &[u8],
    args: &[Val],
) -> LuaResult<u32> {
    let rest: Vec<Val> = args.iter().skip(1).copied().collect();
    let (new_format, new_rest) = rewrite_wow_format(format, &rest)?;
    let format_value = create_string_bytes(state, &new_format);
    let mut delegated = Vec::with_capacity(new_rest.len() + 1);
    delegated.push(format_value);
    delegated.extend(new_rest);
    let normalized_args = normalize_nil_numeric_args(&new_format, &delegated);
    delegate(state, original, &normalized_args)
}

fn read_stack_args(state: &LuaState) -> Vec<Val> {
    let nargs = (state.top as i32 - state.base as i32).max(0) as i32;
    (0..nargs).map(|i| stack_val(state, i + 1)).collect()
}

fn read_string(
    state: &LuaState,
    s: rilua::vm::gc::arena::GcRef<rilua::vm::string::LuaString>,
) -> Option<Vec<u8>> {
    let lua_str = state.gc.string_arena.get(s)?;
    Some(lua_str.data().to_vec())
}

fn format_requires_string_arg(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'%' {
            i += 1;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i + 1] == b'%' {
            i += 2;
            continue;
        }
        let mut j = i + 1;
        while j < bytes.len()
            && matches!(
                bytes[j] as char,
                '-' | '+' | ' ' | '#' | '0' | '.' | '1'..='9'
            )
        {
            j += 1;
        }
        if j < bytes.len() && matches!(bytes[j] as char, 's' | 'q') {
            return true;
        }
        i = j.saturating_add(1);
    }
    false
}

fn validate_required_string_arg(fmt: &[u8], args: &[Val]) -> LuaResult<()> {
    if !format_requires_string_arg(fmt) {
        return Ok(());
    }

    match args.get(1).copied() {
        None => Err(runtime_error(
            "bad argument #2 to '?' (string expected, got no value)",
        )),
        Some(Val::Nil) => Err(runtime_error(
            "bad argument #2 to '?' (string expected, got nil)",
        )),
        _ => Ok(()),
    }
}

fn normalize_nil_numeric_args(fmt: &[u8], args: &[Val]) -> Vec<Val> {
    if !args.iter().any(|arg| matches!(arg, Val::Nil)) {
        return args.to_vec();
    }

    let mut normalized = args.to_vec();
    for index in numeric_format_arg_indices(fmt) {
        if matches!(normalized.get(index), Some(Val::Nil)) {
            normalized[index] = Val::Num(0.0);
        }
    }
    normalized
}

fn numeric_format_arg_indices(bytes: &[u8]) -> Vec<usize> {
    let mut indices = Vec::new();
    let mut arg_index = 1;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'%' {
            i += 1;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i + 1] == b'%' {
            i += 2;
            continue;
        }
        let Some(conversion_index) = format_conversion_index(bytes, i) else {
            break;
        };
        if is_numeric_format_conversion(bytes[conversion_index]) {
            indices.push(arg_index);
        }
        arg_index += 1;
        i = conversion_index + 1;
    }
    indices
}

fn format_conversion_index(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start + 1;
    while index < bytes.len()
        && matches!(
            bytes[index] as char,
            '-' | '+' | ' ' | '#' | '0' | '.' | '1'..='9'
        )
    {
        index += 1;
    }
    (index < bytes.len()).then_some(index)
}

fn is_numeric_format_conversion(byte: u8) -> bool {
    matches!(
        byte as char,
        'c' | 'd' | 'i' | 'u' | 'o' | 'x' | 'X' | 'f' | 'e' | 'E' | 'g' | 'G'
    )
}

fn delegate(state: &mut LuaState, original: Val, args: &[Val]) -> LuaResult<u32> {
    match call_function_state(state, original, args) {
        Ok(value) => {
            state.push(value);
            Ok(1)
        }
        Err(e) => {
            trace_format_error(state, args);
            Err(e)
        }
    }
}

fn trace_format_error(state: &LuaState, args: &[Val]) {
    if std::env::var_os("WOW_SIM_TRACE_STRING_FORMAT_ERRORS").is_none() {
        return;
    }

    let fmt = args
        .first()
        .and_then(|value| describe_string_value(state, *value))
        .unwrap_or_else(|| "<missing format>".to_string());
    let arg_types = args
        .iter()
        .enumerate()
        .skip(1)
        .map(|(index, value)| format!("#{index}:{}", describe_value(state, *value)))
        .collect::<Vec<_>>()
        .join(", ");
    eprintln!("[string.format] failed fmt={fmt:?} args=[{arg_types}]");
}

fn describe_value(state: &LuaState, value: Val) -> String {
    match value {
        Val::Nil => "nil".to_string(),
        Val::Bool(value) => format!("boolean:{value}"),
        Val::Num(value) => format!("number:{value}"),
        Val::Str(_) => describe_string_value(state, value)
            .map(|value| format!("string:{value:?}"))
            .unwrap_or_else(|| "string:<missing>".to_string()),
        _ => value.type_name().to_string(),
    }
}

fn describe_string_value(state: &LuaState, value: Val) -> Option<String> {
    let Val::Str(string_ref) = value else {
        return None;
    };
    let bytes = state.gc.string_arena.get(string_ref)?.data();
    Some(String::from_utf8_lossy(bytes).to_string())
}

#[cfg(all(test, not(all(feature = "client-retail", feature = "retail-12-1-5"))))]
mod tests {
    use super::patch_string_format;
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn patch_string_format_is_idempotent() {
        let env = WowLuaEnv::new().expect("env");
        {
            let loader = env.loader_env();
            let mut lua = loader.rilua_mut();
            patch_string_format(&mut lua).expect("second patch should succeed");
        }
        let out: String = env
            .eval(r#"return string.format("%2$s %1$s %.1F", "first", "second", 3.25)"#)
            .expect("patched format should still work");
        assert_eq!(out, "second first 3.2");
    }

    #[test]
    fn format_preserves_literal_bytes() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            local bytes = string.char(255, 128)
            assert(string.format(bytes .. ' %% %.1F ' .. bytes, 2.5) == bytes .. ' % 2.5 ' .. bytes)
            assert(string.format('é %% %1$d 雪', 7) == 'é % 7 雪')
        "#,
        )
        .expect("format must preserve literal bytes");
    }

    #[test]
    fn nil_numeric_format_args_are_zero() {
        let env = WowLuaEnv::new().expect("env");
        let out: String = env
            .eval(r#"return string.format("%d %.1f %2$d", nil, nil)"#)
            .expect("nil numeric args should format as zero");
        assert_eq!(out, "0 0.0 0");
    }
}

/// Parse a WoW-dialect format string: replace `%F` → `%f` and reorder
/// positional args (`%1$s`). Returns the cleaned format + the arg slice
/// in positional order (or the original args when no positional spec
/// was found — matching master behaviour and avoiding unnecessary
/// cloning on the hot path).
fn rewrite_wow_format(bytes: &[u8], args: &[Val]) -> LuaResult<(Vec<u8>, Vec<Val>)> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut reordered = Vec::new();
    let mut seq = 0;
    let mut has_positional = false;
    let mut index = 0;
    while index < bytes.len() {
        index = append_next_format_item(
            bytes,
            index,
            args,
            &mut out,
            &mut reordered,
            &mut seq,
            &mut has_positional,
        )?;
    }
    let args = if has_positional {
        reordered
    } else {
        args.to_vec()
    };
    Ok((out, args))
}

fn append_next_format_item(
    bytes: &[u8],
    index: usize,
    args: &[Val],
    output: &mut Vec<u8>,
    reordered: &mut Vec<Val>,
    sequence: &mut usize,
    has_positional: &mut bool,
) -> LuaResult<usize> {
    if bytes[index] != b'%' {
        output.push(bytes[index]);
        return Ok(index + 1);
    }
    if index + 1 < bytes.len() && bytes[index + 1] == b'%' {
        output.extend_from_slice(b"%%");
        return Ok(index + 2);
    }
    parse_format_specifier(
        bytes,
        index,
        args,
        output,
        reordered,
        sequence,
        has_positional,
    )
}

/// Parse one format specifier starting at `%`, appending to `out` and
/// collecting the matched arg. Returns the index after the specifier.
fn parse_format_specifier(
    bytes: &[u8],
    start: usize,
    args: &[Val],
    out: &mut Vec<u8>,
    reordered: &mut Vec<Val>,
    seq: &mut usize,
    has_positional: &mut bool,
) -> LuaResult<usize> {
    let index =
        append_format_specifier_argument(bytes, start, args, out, reordered, seq, has_positional)?;
    let index = skip_flags_width_precision(bytes, index, out);
    append_format_conversion(bytes, index, out)
}

fn append_format_specifier_argument(
    bytes: &[u8],
    start: usize,
    args: &[Val],
    out: &mut Vec<u8>,
    reordered: &mut Vec<Val>,
    seq: &mut usize,
    has_positional: &mut bool,
) -> LuaResult<usize> {
    let index = start + 1;
    if let Some((position, after)) = parse_positional_index(bytes, index) {
        if position >= 100 {
            return Err(runtime_error(
                "invalid format (width or precision too long)",
            ));
        }
        *has_positional = true;
        // Positional references mark slots seen for later sequential conversions.
        *seq = std::cmp::max(*seq, position);
        reordered.push(args.get(position - 1).copied().unwrap_or(Val::Nil));
        out.push(b'%');
        return Ok(after);
    }
    *seq += 1;
    reordered.push(args.get(*seq - 1).copied().unwrap_or(Val::Nil));
    out.push(b'%');
    Ok(index)
}

fn append_format_conversion(bytes: &[u8], index: usize, out: &mut Vec<u8>) -> LuaResult<usize> {
    if index < bytes.len() && is_format_conversion(bytes[index]) {
        out.push(if bytes[index] == b'F' {
            b'f'
        } else {
            bytes[index]
        });
        return Ok(index + 1);
    }
    Ok(index)
}

/// Skip flags (`-+ #0`), width digits, and precision (`.N`) — appending
/// to `out` as we go.
fn skip_flags_width_precision(bytes: &[u8], start: usize, out: &mut Vec<u8>) -> usize {
    let mut i = start;
    while i < bytes.len() && matches!(bytes[i], b'-' | b'+' | b' ' | b'#' | b'0') {
        out.push(bytes[i]);
        i += 1;
    }
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        out.push(bytes[i]);
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        out.push(b'.');
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            out.push(bytes[i]);
            i += 1;
        }
    }
    i
}

/// Try to parse `N$` (digits followed by `$`) at `start`. Returns
/// `(N, index_after_$)` on match, `None` otherwise.
fn parse_positional_index(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    let mut i = start;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start || i >= bytes.len() || bytes[i] != b'$' {
        return None;
    }
    let n: usize = std::str::from_utf8(&bytes[start..i]).ok()?.parse().ok()?;
    Some((n, i + 1))
}

/// Valid printf conversion characters. `%F` is accepted here and
/// rewritten to `%f` by the caller.
fn is_format_conversion(b: u8) -> bool {
    matches!(
        b,
        b'd' | b'i'
            | b'o'
            | b'u'
            | b'x'
            | b'X'
            | b'e'
            | b'E'
            | b'f'
            | b'F'
            | b'g'
            | b'G'
            | b'a'
            | b'A'
            | b'c'
            | b's'
            | b'p'
            | b'q'
            | b'n'
    )
}
