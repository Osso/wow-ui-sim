//! WoW's `string.upper` / `string.lower`.
//!
//! The client's lstrlib decodes UTF-8 and maps ASCII, U+00C0–U+00DE ↔
//! U+00E0–U+00FE (a flat 0x20 offset, so × ↔ ÷), U+0410–U+042F ↔
//! U+0430–U+044F and Ё ↔ ё; bytes that are not valid UTF-8 are dropped
//! (3.3.5a lstrlib.c decomp via aso; Retail 12.1.5 observation in
//! Meorawr/elune#10).

use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    let string_key = state.gc.intern_string_static(b"string");
    let Some(global) = state.gc.tables.get(state.global) else {
        return Err(runtime_error("global table has been collected"));
    };
    let Val::Table(string) = global.get_str(string_key, &state.gc.string_arena) else {
        return Err(runtime_error("string table is unavailable"));
    };
    table_set_rust_fn_static(state, string, "upper", upper)?;
    table_set_rust_fn_static(state, string, "lower", lower)?;
    Ok(())
}

fn upper(state: &mut LuaState) -> LuaResult<u32> {
    map_case(state, "upper", to_upper)
}

fn lower(state: &mut LuaState) -> LuaResult<u32> {
    map_case(state, "lower", to_lower)
}

fn map_case(state: &mut LuaState, name: &str, map: fn(char) -> char) -> LuaResult<u32> {
    let input = string_argument(state, name)?;
    let mapped = map_utf8(&input, map);
    let interned = state.gc.intern_string(&mapped);
    state.push(Val::Str(interned));
    Ok(1)
}

fn string_argument(state: &LuaState, name: &str) -> LuaResult<Vec<u8>> {
    match stack_val(state, 1) {
        Val::Str(s) => Ok(state
            .gc
            .string_arena
            .get(s)
            .map(|s| s.data().to_vec())
            .unwrap_or_default()),
        number @ Val::Num(_) => Ok(number.to_string().into_bytes()),
        other => {
            let got = if state.top <= state.base {
                "no value"
            } else {
                other.type_name()
            };
            Err(runtime_error(format!(
                "bad argument #1 to '{name}' (string expected, got {got})"
            )))
        }
    }
}

fn map_utf8(input: &[u8], map: fn(char) -> char) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    for chunk in input.utf8_chunks() {
        for ch in chunk.valid().chars() {
            let mut buf = [0u8; 4];
            out.extend_from_slice(map(ch).encode_utf8(&mut buf).as_bytes());
        }
    }
    out
}

fn to_upper(ch: char) -> char {
    match ch {
        'a'..='z' | '\u{E0}'..='\u{FE}' | '\u{430}'..='\u{44F}' => offset(ch, -0x20),
        '\u{451}' => '\u{401}',
        _ => ch,
    }
}

fn to_lower(ch: char) -> char {
    match ch {
        'A'..='Z' | '\u{C0}'..='\u{DE}' | '\u{410}'..='\u{42F}' => offset(ch, 0x20),
        '\u{401}' => '\u{451}',
        _ => ch,
    }
}

fn offset(ch: char, delta: i32) -> char {
    char::from_u32((ch as i32 + delta) as u32).unwrap_or(ch)
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    fn eval(code: &str) -> String {
        WowLuaEnv::new()
            .expect("lua env should initialize")
            .eval(code)
            .expect("probe should run")
    }

    #[test]
    fn case_maps_latin1_and_cyrillic_like_the_client() {
        let result = eval(
            r#"
            local cases = {
                { string.upper("été"), "ÉTÉ" },
                { string.lower("ÀÉÖ"), "àéö" },
                { string.upper("привет"), "ПРИВЕТ" },
                { string.lower("Ё"), "ё" },
                { string.lower("×"), "÷" },
                { string.lower("a\233b"), "ab" },
                { string.upper("straße"), "STRAßE" },
                { string.upper("ÿ"), "ÿ" },
                { ("MiXeD 12"):lower(), "mixed 12" },
                { string.upper(12), "12" },
            }
            for i, case in ipairs(cases) do
                if case[1] ~= case[2] then return i .. ":" .. case[1] end
            end
            return "ok"
            "#,
        );
        assert_eq!(result, "ok");
    }

    #[test]
    fn rejects_non_string_arguments() {
        let result = eval(
            r#"
            local ok, err = pcall(string.upper, {})
            return tostring(ok) .. "|" .. tostring(err)
            "#,
        );
        assert_eq!(
            result,
            "false|bad argument #1 to 'upper' (string expected, got table)"
        );
    }
}
