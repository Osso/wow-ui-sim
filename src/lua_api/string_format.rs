//! WoW-compatible `string.format` patch — rilua port of the master-era
//! `src/lua_api/string_format.rs`.
//!
//! Handles WoW's verified `string.format` extensions:
//! - `%F`, `%t`, and `%T` are rewritten to `%f` with their format fields.
//! - Positional arguments select positions 0 through 99. Position 0 selects
//!   the format argument, and sequential arguments continue after the last
//!   positional selection.
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
const ORIGINAL_TONUMBER_REGISTRY_KEY: &str = "__original_string_tonumber";
const INVALID_OPTION_ERROR: &str = "invalid option in `format'";
const WIDTH_PRECISION_ERROR: &str = "invalid format (width or precision too long)";

/// Install the WoW `string.format` patch on the rilua VM.
///
/// Replaces `string.format` and the `format` global with a RustFn wrapper
/// that handles WoW's verified conversions and positional arguments before
/// delegating to the original implementation (stashed under registry keys).
pub fn patch_string_format(lua: &mut rilua::Lua) -> LuaResult<()> {
    let existing = registry_get(lua.state_mut(), ORIG_REGISTRY_KEY);
    if !matches!(existing, Val::Function(_)) {
        let original = read_string_format(lua)?;
        registry_set(lua.state_mut(), ORIG_REGISTRY_KEY, original);
    }

    let original_tonumber = registry_get(lua.state_mut(), ORIGINAL_TONUMBER_REGISTRY_KEY);
    if !matches!(original_tonumber, Val::Function(_)) {
        let tonumber = LuaApiMut::get_global_val(lua, "tonumber");
        if !matches!(tonumber, Val::Function(_)) {
            return Err(runtime_error("tonumber global missing"));
        }
        registry_set(lua.state_mut(), ORIGINAL_TONUMBER_REGISTRY_KEY, tonumber);
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
    let (original, tonumber) = read_original_format_functions(state)?;
    let args = read_stack_args(state);
    let Some((format_argument, fmt)) = read_format_argument(state, &args)? else {
        return delegate(state, original, &args);
    };
    if !validate_format_arguments(state, tonumber, &fmt, &args)? {
        return delegate(state, original, &args);
    }

    let (new_fmt, new_args) =
        process_wow_format(state, tonumber, &fmt, format_argument, &args[1..])?;
    let new_fmt_val = create_string_bytes(state, &new_fmt);
    let mut delegated = Vec::with_capacity(new_args.len() + 1);
    delegated.push(new_fmt_val);
    delegated.extend(new_args);
    delegate(state, original, &delegated)
}

fn read_original_format_functions(state: &mut LuaState) -> LuaResult<(Val, Val)> {
    let original = registry_get(state, ORIG_REGISTRY_KEY);
    let tonumber = registry_get(state, ORIGINAL_TONUMBER_REGISTRY_KEY);
    if !matches!(original, Val::Function(_)) {
        return Err(runtime_error("__original_string_format missing"));
    }
    if !matches!(tonumber, Val::Function(_)) {
        return Err(runtime_error("__original_string_tonumber missing"));
    }
    Ok((original, tonumber))
}

fn read_format_argument(state: &LuaState, args: &[Val]) -> LuaResult<Option<(Val, Vec<u8>)>> {
    let Some(format_argument) = args.first().copied() else {
        return Err(format_type_error(state, 0, "string"));
    };
    match format_argument {
        Val::Str(format_ref) => {
            Ok(read_string(state, format_ref).map(|bytes| (format_argument, bytes)))
        }
        Val::Num(_) => Ok(None),
        _ => Err(format_type_error(state, 0, "string")),
    }
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

fn validate_format_arguments(
    state: &mut LuaState,
    tonumber: Val,
    bytes: &[u8],
    args: &[Val],
) -> LuaResult<bool> {
    let mut needs_processing = false;
    let mut scan = FormatScan::new(bytes);
    while let Some(specifier) = scan.next_specifier()? {
        needs_processing |= specifier.positional || is_float_alias(specifier.conversion);
        let value = args
            .get(specifier.argument_position)
            .copied()
            .unwrap_or(Val::Nil);
        needs_processing |= validate_format_argument(
            state,
            tonumber,
            value,
            specifier.argument_position,
            specifier.conversion,
        )?;
    }
    Ok(needs_processing)
}

fn validate_format_argument(
    state: &mut LuaState,
    tonumber: Val,
    value: Val,
    argument_position: usize,
    conversion: u8,
) -> LuaResult<bool> {
    match conversion {
        b's' if matches!(value, Val::Str(_) | Val::Num(_)) => Ok(false),
        b's' => Err(format_type_error(state, argument_position, "string")),
        b'q' => match value {
            Val::Str(_) => Ok(false),
            Val::Num(_) => Ok(true),
            _ => Err(format_type_error(state, argument_position, "string")),
        },
        b'd' | b'i' => Ok(!numeric_conversion_succeeds(state, tonumber, value)?),
        b'c' | b'o' | b'u' | b'x' | b'X' | b'e' | b'E' | b'f' | b'F' | b'g' | b'G' | b't'
        | b'T' => {
            if numeric_conversion_succeeds(state, tonumber, value)? {
                Ok(false)
            } else {
                Err(numeric_type_error(state, argument_position))
            }
        }
        _ => Ok(false),
    }
}

fn numeric_conversion_succeeds(state: &mut LuaState, tonumber: Val, value: Val) -> LuaResult<bool> {
    match value {
        Val::Num(_) => Ok(true),
        Val::Str(_) => Ok(matches!(
            call_function_state(state, tonumber, &[value])?,
            Val::Num(_)
        )),
        _ => Ok(false),
    }
}

fn format_type_error(state: &LuaState, argument_index: usize, expected: &str) -> rilua::LuaError {
    let stack_index = state.base + argument_index;
    let actual = if stack_index < state.top {
        state.stack_get(stack_index).type_name()
    } else {
        "no value"
    };
    if rilua::vm::debug_info::getfuncname(state, state.ci, &state.gc.string_arena).is_none() {
        return runtime_error(format!(
            "bad argument #{} ({expected} expected, got {actual})",
            argument_index + 1
        ));
    }
    rilua::stdlib::type_error(state, argument_index, expected)
}

fn numeric_type_error(state: &LuaState, argument_index: usize) -> rilua::LuaError {
    let stack_index = state.base + argument_index;
    if stack_index < state.top
        && rilua::table_security::secret_payload_kind(state, state.stack_get(stack_index))
            == Some(rilua::table_security::SecretPayloadKind::String)
    {
        return format_argument_error(state, argument_index + 1, "number expected, got string");
    }
    format_type_error(state, argument_index, "number")
}

fn format_argument_error(
    state: &LuaState,
    argument_number: usize,
    message: &str,
) -> rilua::LuaError {
    if rilua::vm::debug_info::getfuncname(state, state.ci, &state.gc.string_arena).is_none() {
        return runtime_error(format!("bad argument #{argument_number} ({message})"));
    }
    rilua::stdlib::arg_error(state, argument_number, message)
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
    use super::patch_string_format;
    use crate::lua_api::WowLuaEnv;
    use rilua::LuaApiMut;

    #[test]
    fn patch_string_format_is_idempotent() {
        let env = WowLuaEnv::new().expect("env");
        {
            let loader = env.loader_env();
            let mut lua = loader.rilua_mut();
            patch_string_format(&mut lua).expect("second patch should succeed");
        }
        let matches_lowercase: bool = env
            .eval(
                r#"
                return string.format("%2$s %1$s %3$.1F", "first", "second", 3.25)
                    == string.format("%2$s %1$s %3$.1f", "first", "second", 3.25)
                "#,
            )
            .expect("patched format should still work");
        assert!(matches_lowercase);
    }

    #[test]
    fn format_preserves_literal_bytes() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            local bytes = string.char(255, 128)
            local fixed = string.format('%.1f', 2.5)
            assert(string.format(bytes .. ' %% %.1F ' .. bytes, 2.5)
                == bytes .. ' % ' .. fixed .. ' ' .. bytes)
            assert(string.format('é %% %1$d 雪', 7) == string.format('é %% %d 雪', 7))
        "#,
        )
        .expect("format must preserve literal bytes");
    }

    #[test]
    fn integer_conversions_return_zero_when_conversion_fails() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            assert(string.format('%d/%i/%d/%i', nil, 'not numeric', false)
                == '0/0/0/0')
            local ok, err = pcall(function() return string.format('%u', nil) end)
            assert(not ok)
            assert(err == "bad argument #2 to 'format' (number expected, got nil)")
        "#,
        )
        .expect("only %d and %i should replace conversion failures with zero");
    }

    #[test]
    fn signed_integer_formats_report_i32_overflow() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            assert(string.format('%d/%i', 2147483647, -2147483648)
                == '2147483647/-2147483648')

            local cases = {
                { '%d', 2147483648, '2147483648.000000' },
                { '%d', -2147483649, '-2147483649.000000' },
                { '%i', 2147483648, '2147483648.000000' },
                { '%i', -2147483649, '-2147483649.000000' },
            }
            for _, case in ipairs(cases) do
                local ok, err = pcall(function()
                    return string.format(case[1], case[2])
                end)
                assert(not ok, 'overflow must fail for ' .. case[1])
                assert(err == 'integer overflow attempting to store ' .. case[3], tostring(err))
            end
        "#,
        )
        .expect("%d and %i must reject values outside signed 32-bit range");
    }

    #[test]
    fn positional_formats_select_the_format_and_reset_the_cursor() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            assert(string.format('%0$s') == '%0$s')
            assert(string.format('%01$s', 'first') == 'first')
            assert(string.format('%3$s %1$s %s', 'one', 'two', 'three')
                == 'three one two')
        "#,
        )
        .expect("positional and sequential selectors should share a resettable cursor");
    }

    #[test]
    fn positional_prefixes_and_parser_errors_match_native_templates() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            local ok, err = pcall(function() return string.format('%001$s', 'first') end)
            assert(not ok and err == 'invalid format (width or precision too long)')
            ok, err = pcall(function() return string.format('%100$s', 'first') end)
            assert(not ok and err == 'invalid format (width or precision too long)')
            ok, err = pcall(function() return string.format('%123f', 1) end)
            assert(not ok and err == 'invalid format (width or precision too long)')
            ok, err = pcall(function() return string.format('%1.123f', 1) end)
            assert(not ok and err == 'invalid format (width or precision too long)')
            ok, err = pcall(function() return string.format('%z') end)
            assert(not ok and err == "invalid option in `format'")
            ok, err = pcall(function() return string.format('%a', 1) end)
            assert(not ok and err == "invalid option in `format'")
            ok, err = pcall(function() return string.format('%') end)
            assert(not ok and err == "invalid option in `format'")
        "#,
        )
        .expect("format parser errors should match the Retail templates");
    }

    #[test]
    fn format_argument_errors_use_caller_and_method_names() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            local ok, err = pcall(function() return string.format('%s') end)
            assert(not ok)
            assert(err == "bad argument #2 to 'format' (string expected, got no value)")

            ok, err = pcall(function() return string.format('%2$s', 'first') end)
            assert(not ok)
            assert(err == "bad argument #3 to 'format' (string expected, got no value)")

            ok, err = pcall(function() return string.format('%s', false) end)
            assert(not ok)
            assert(err == "bad argument #2 to 'format' (string expected, got boolean)")

            ok, err = pcall(function() return string.format('%t', 'not numeric') end)
            assert(not ok)
            assert(err == "bad argument #2 to 'format' (number expected, got string)")

            local fmt = string.format
            ok, err = pcall(function() return fmt('%s') end)
            assert(not ok)
            assert(err == "bad argument #2 to 'fmt' (string expected, got no value)")

            ok, err = pcall(function() return string:format('%s') end)
            assert(not ok)
            assert(err == "calling 'format' on bad self (string expected, got table)")
        "#,
        )
        .expect("format type errors should use the caller and method context");
    }

    #[test]
    fn verified_format_extensions_map_and_ignore_quoted_width() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            assert(string.format('%08.2t', 12.5) == string.format('%08.2f', 12.5))
            assert(string.format('%20.3q', 'text') == string.format('%q', 'text'))
            assert(string.format('%q', 12) == string.format('%q', '12'))
        "#,
        )
        .expect("verified conversion mapping and quote conversion should match");
    }

    #[cfg(feature = "client-retail")]
    #[test]
    fn numeric_conversion_rejects_secret_marked_strings() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            local ok, err = pcall(function() return string.format('%t', secretwrap('12')) end)
            assert(not ok)
            assert(err == "bad argument #2 to 'format' (number expected, got string)")
        "#,
        )
        .expect("numeric conversion should reject a secret-marked string without reading it");
    }

    #[cfg(feature = "client-retail")]
    #[test]
    fn numeric_conversion_accepts_secret_numbers() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            local function assert_secret_format(format, value, expected)
                local result = string.format(format, secretwrap(value))
                assert(issecretvalue(result), format .. ' must keep secret input private')
                assert(secretunwrap(result) == expected, tostring(secretunwrap(result)))
            end

            assert_secret_format('%t', 12.5, string.format('%f', 12.5))
            assert_secret_format('%T', 0, string.format('%f', 0))
            assert_secret_format('%08.2F', -3.25, string.format('%08.2f', -3.25))
            assert_secret_format('%1$t', 7, string.format('%f', 7))

            local ok, err = pcall(function() return string.format('%t') end)
            assert(not ok)
            assert(err == "bad argument #2 to 'format' (number expected, got no value)")
        "#,
        )
        .expect("numeric conversion should format secret numbers into secret results");
    }

    #[test]
    fn host_format_errors_omit_function_name_without_a_debug_caller() {
        let env = WowLuaEnv::new().expect("env");
        let format_string = {
            let loader = env.loader_env();
            let mut lua = loader.rilua_mut();
            LuaApiMut::create_string(lua.state_mut(), b"%s")
        };
        let error = env
            .call_global("format", &[format_string])
            .expect_err("missing format values should fail");
        assert_eq!(
            error.to_string(),
            "Lua error: bad argument #2 (string expected, got no value)"
        );
    }

    #[cfg(feature = "client-retail")]
    #[test]
    fn secure_string_format_uses_retail_globalstring_templates() {
        let env = WowLuaEnv::new().expect("env");
        let (same_function, positional, plain): (bool, String, String) = env
            .eval(
                r#"
                local secure_format = rawget(rawget(__secureenv, "string"), "format")
                local positional = secure_format(
                    TEXT_MODE_A_STRING_1,
                    "source", "spell", "action", "destination", "value", "result",
                    "school", "power", "amount", "extra", "points"
                )
                local plain = secure_format(TEXT_MODE_A_STRING_TIMESTAMP, "time", "line")
                return rawequal(secure_format, string.format), positional, plain
                "#,
            )
            .expect("secure string.format should use the WoW wrapper");

        assert!(same_function);
        assert_eq!(
            positional,
            "source spell action destination value. result points"
        );
        assert_eq!(plain, "time> line");
    }
}

/// Parse positional arguments and apply the conversions that rilua does not
/// implement with the verified WoW semantics.
fn process_wow_format(
    state: &mut LuaState,
    tonumber: Val,
    bytes: &[u8],
    format_argument: Val,
    args: &[Val],
) -> LuaResult<(Vec<u8>, Vec<Val>)> {
    let mut output = Vec::with_capacity(bytes.len());
    let mut reordered = Vec::new();
    let mut has_positional = false;
    let mut has_argument_rewrite = false;
    let mut previous_end = 0;
    let mut scan = FormatScan::new(bytes);

    while let Some(specifier) = scan.next_specifier()? {
        output.extend_from_slice(&bytes[previous_end..specifier.start_index]);
        has_positional |= specifier.positional;
        append_format_conversion(&mut output, bytes, specifier);
        let value = format_argument_at(format_argument, args, specifier.argument_position);
        let (value, rewritten) =
            rewrite_format_argument(state, tonumber, value, specifier.conversion)?;
        reordered.push(value);
        has_argument_rewrite |= rewritten;
        previous_end = specifier.next_index;
    }
    output.extend_from_slice(&bytes[previous_end..]);

    if has_positional || has_argument_rewrite {
        Ok((output, reordered))
    } else {
        Ok((output, args.to_vec()))
    }
}

fn append_format_conversion(output: &mut Vec<u8>, bytes: &[u8], specifier: ParsedFormatSpecifier) {
    output.push(b'%');
    if specifier.conversion != b'q' {
        output.extend_from_slice(&bytes[specifier.body_start..specifier.conversion_index]);
    }
    output.push(if is_float_alias(specifier.conversion) {
        b'f'
    } else {
        specifier.conversion
    });
}

fn format_argument_at(format_argument: Val, args: &[Val], position: usize) -> Val {
    if position == 0 {
        format_argument
    } else {
        args.get(position - 1).copied().unwrap_or(Val::Nil)
    }
}

fn rewrite_format_argument(
    state: &mut LuaState,
    tonumber: Val,
    value: Val,
    conversion: u8,
) -> LuaResult<(Val, bool)> {
    if matches!(conversion, b'd' | b'i') && !numeric_conversion_succeeds(state, tonumber, value)? {
        return Ok((Val::Num(0.0), true));
    }
    if conversion == b'q'
        && let Val::Num(_) = value
    {
        let string = format!("{value}");
        return Ok((create_string_bytes(state, string.as_bytes()), true));
    }
    Ok((value, false))
}

#[derive(Clone, Copy)]
struct ParsedFormatSpecifier {
    start_index: usize,
    next_index: usize,
    argument_position: usize,
    body_start: usize,
    conversion_index: usize,
    conversion: u8,
    positional: bool,
}

struct FormatScan<'a> {
    bytes: &'a [u8],
    index: usize,
    argument_cursor: usize,
}

impl<'a> FormatScan<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            index: 0,
            argument_cursor: 0,
        }
    }

    fn next_specifier(&mut self) -> LuaResult<Option<ParsedFormatSpecifier>> {
        while self.index < self.bytes.len() {
            let start = self.index;
            if self.bytes[start] != b'%' {
                self.index += 1;
                continue;
            }
            if start + 1 < self.bytes.len() && self.bytes[start + 1] == b'%' {
                self.index += 2;
                continue;
            }

            let specifier =
                parse_next_format_specifier(self.bytes, start, &mut self.argument_cursor)?;
            self.index = specifier.next_index;
            return Ok(Some(specifier));
        }
        Ok(None)
    }
}

fn parse_next_format_specifier(
    bytes: &[u8],
    start: usize,
    argument_cursor: &mut usize,
) -> LuaResult<ParsedFormatSpecifier> {
    let (argument_position, body_start, positional) =
        parse_argument_position(bytes, start + 1, argument_cursor)?;
    let conversion_index = skip_flags_width_precision(bytes, body_start)?;
    let conversion = parse_conversion(bytes, conversion_index)?;
    Ok(ParsedFormatSpecifier {
        start_index: start,
        next_index: conversion_index + 1,
        argument_position,
        body_start,
        conversion_index,
        conversion,
        positional,
    })
}

fn parse_argument_position(
    bytes: &[u8],
    start: usize,
    argument_cursor: &mut usize,
) -> LuaResult<(usize, usize, bool)> {
    if let Some((position, after_prefix)) = parse_positional_index(bytes, start)? {
        *argument_cursor = position;
        return Ok((position, after_prefix, true));
    }
    *argument_cursor += 1;
    Ok((*argument_cursor, start, false))
}

fn parse_conversion(bytes: &[u8], index: usize) -> LuaResult<u8> {
    let Some(conversion) = bytes.get(index).copied() else {
        return Err(runtime_error(INVALID_OPTION_ERROR));
    };
    if !is_format_conversion(conversion) {
        return Err(runtime_error(INVALID_OPTION_ERROR));
    }
    Ok(conversion)
}

/// Parse flags, width and precision. Each numeric field has at most two
/// digits, and the whole section has at most sixteen characters.
fn skip_flags_width_precision(bytes: &[u8], start: usize) -> LuaResult<usize> {
    let flags_end = skip_format_flags(bytes, start);
    let width_end = skip_format_width(bytes, flags_end)?;
    let specifier_end = skip_format_precision(bytes, width_end)?;
    if specifier_end - start > 16 {
        return Err(runtime_error(WIDTH_PRECISION_ERROR));
    }
    Ok(specifier_end)
}

fn skip_format_flags(bytes: &[u8], start: usize) -> usize {
    let mut index = start;
    while index < bytes.len() && matches!(bytes[index], b'-' | b'+' | b' ' | b'#' | b'0') {
        index += 1;
    }
    index
}

fn skip_format_width(bytes: &[u8], start: usize) -> LuaResult<usize> {
    let mut index = start;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index - start > 2 {
        return Err(runtime_error(WIDTH_PRECISION_ERROR));
    }
    Ok(index)
}

fn skip_format_precision(bytes: &[u8], start: usize) -> LuaResult<usize> {
    if start >= bytes.len() || bytes[start] != b'.' {
        return Ok(start);
    }
    let mut index = start + 1;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index - start - 1 > 2 {
        return Err(runtime_error(WIDTH_PRECISION_ERROR));
    }
    Ok(index)
}

/// Parse the one- or two-digit positional prefix `N$` at `start`.
fn parse_positional_index(bytes: &[u8], start: usize) -> LuaResult<Option<(usize, usize)>> {
    let mut index = start;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index == start || index >= bytes.len() || bytes[index] != b'$' {
        return Ok(None);
    }
    if index - start > 2 {
        return Err(runtime_error(WIDTH_PRECISION_ERROR));
    }

    let position = std::str::from_utf8(&bytes[start..index])
        .ok()
        .and_then(|digits| digits.parse().ok())
        .ok_or_else(|| runtime_error(WIDTH_PRECISION_ERROR))?;
    Ok(Some((position, index + 1)))
}

fn is_float_alias(conversion: u8) -> bool {
    matches!(conversion, b'F' | b't' | b'T')
}

/// WoW's native formatter accepts exactly these conversion letters.
fn is_format_conversion(byte: u8) -> bool {
    matches!(
        byte,
        b'c' | b'd'
            | b'i'
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
            | b'q'
            | b's'
            | b't'
            | b'T'
    )
}
