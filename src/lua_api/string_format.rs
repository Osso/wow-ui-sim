//! WoW-compatible `string.format` patch — rilua port of the master-era
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

use crate::lua_api::methods::{call_function_state, create_string, registry_get, registry_set};
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
    if !fmt.contains('F') && !fmt.contains('$') {
        let normalized_args = normalize_nil_numeric_args(&fmt, &args);
        return delegate(state, original, &normalized_args);
    }

    let rest: Vec<Val> = args.iter().skip(1).copied().collect();
    let (new_fmt, new_rest) = process_wow_format(&fmt, &rest)?;
    let new_fmt_val = create_string(state, &new_fmt);
    let mut delegated: Vec<Val> = Vec::with_capacity(new_rest.len() + 1);
    delegated.push(new_fmt_val);
    delegated.extend(new_rest);
    let normalized_args = normalize_nil_numeric_args(&new_fmt, &delegated);
    delegate(state, original, &normalized_args)
}

fn read_stack_args(state: &LuaState) -> Vec<Val> {
    let nargs = (state.top as i32 - state.base as i32).max(0) as i32;
    (0..nargs).map(|i| stack_val(state, i + 1)).collect()
}

fn read_string(
    state: &LuaState,
    s: rilua::vm::gc::arena::GcRef<rilua::vm::string::LuaString>,
) -> Option<String> {
    let lua_str = state.gc.string_arena.get(s)?;
    std::str::from_utf8(lua_str.data()).ok().map(str::to_owned)
}

fn format_requires_string_arg(fmt: &str) -> bool {
    let bytes = fmt.as_bytes();
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

fn validate_required_string_arg(fmt: &str, args: &[Val]) -> LuaResult<()> {
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

fn normalize_nil_numeric_args(fmt: &str, args: &[Val]) -> Vec<Val> {
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

fn numeric_format_arg_indices(fmt: &str) -> Vec<usize> {
    let bytes = fmt.as_bytes();
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

        let mut j = i + 1;
        while j < bytes.len()
            && matches!(
                bytes[j] as char,
                '-' | '+' | ' ' | '#' | '0' | '.' | '1'..='9'
            )
        {
            j += 1;
        }
        if j >= bytes.len() {
            break;
        }

        if matches!(
            bytes[j] as char,
            'c' | 'd' | 'i' | 'u' | 'o' | 'x' | 'X' | 'f' | 'e' | 'E' | 'g' | 'G'
        ) {
            indices.push(arg_index);
        }

        arg_index += 1;
        i = j + 1;
    }

    indices
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

#[cfg(test)]
mod tests {
    use super::{patch_string_format, process_wow_format};
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
    fn literal_utf8_in_the_format_string_survives() {
        let (out, _) = process_wow_format("Version %s \u{b7} Ärger %d%%", &[]).unwrap();
        assert_eq!(out, "Version %s \u{b7} Ärger %d%%");
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
fn process_wow_format(fmt: &str, args: &[Val]) -> LuaResult<(String, Vec<Val>)> {
    let bytes = fmt.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    let mut reordered: Vec<Val> = Vec::new();
    let mut seq: usize = 0;
    let mut has_positional = false;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'%' {
            // Copy the literal run as-is. `%` is ASCII, so the run ends on a
            // char boundary; pushing byte-by-byte as `char` would re-encode
            // every non-ASCII byte and turn "·" into "Â·".
            let start = i;
            while i < bytes.len() && bytes[i] != b'%' {
                i += 1;
            }
            out.push_str(&fmt[start..i]);
        } else if i + 1 < bytes.len() && bytes[i + 1] == b'%' {
            out.push_str("%%");
            i += 2;
        } else {
            i = parse_format_specifier(
                bytes,
                i,
                args,
                &mut out,
                &mut reordered,
                &mut seq,
                &mut has_positional,
            )?;
        }
    }

    if has_positional {
        Ok((out, reordered))
    } else {
        Ok((out, args.to_vec()))
    }
}

/// Parse one format specifier starting at `%`, appending to `out` and
/// collecting the matched arg. Returns the index after the specifier.
fn parse_format_specifier(
    bytes: &[u8],
    start: usize,
    args: &[Val],
    out: &mut String,
    reordered: &mut Vec<Val>,
    seq: &mut usize,
    has_positional: &mut bool,
) -> LuaResult<usize> {
    let mut i = start + 1; // skip the '%'

    if let Some((n, after)) = parse_positional_index(bytes, i) {
        if n >= 100 {
            return Err(runtime_error(
                "invalid format (width or precision too long)",
            ));
        }
        *has_positional = true;
        // A positional spec `%N$` bumps `seq` to max(seq, N) without
        // consuming. Later sequential specs then pick from `seq+1` onward,
        // matching WoW's patched LuaJIT where sequential consumes the
        // lowest arg slot not yet "seen" by positional refs.
        *seq = std::cmp::max(*seq, n);
        reordered.push(args.get(n - 1).copied().unwrap_or(Val::Nil));
        out.push('%');
        i = after;
    } else {
        *seq += 1;
        reordered.push(args.get(*seq - 1).copied().unwrap_or(Val::Nil));
        out.push('%');
    }

    i = skip_flags_width_precision(bytes, i, out);
    if i < bytes.len() && is_format_conversion(bytes[i]) {
        out.push(if bytes[i] == b'F' {
            'f'
        } else {
            bytes[i] as char
        });
        i += 1;
    }
    Ok(i)
}

/// Skip flags (`-+ #0`), width digits, and precision (`.N`) — appending
/// to `out` as we go.
fn skip_flags_width_precision(bytes: &[u8], start: usize, out: &mut String) -> usize {
    let mut i = start;
    while i < bytes.len() && matches!(bytes[i], b'-' | b'+' | b' ' | b'#' | b'0') {
        out.push(bytes[i] as char);
        i += 1;
    }
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        out.push(bytes[i] as char);
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        out.push('.');
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            out.push(bytes[i] as char);
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
