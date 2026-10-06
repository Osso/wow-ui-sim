//! Name shortening with the 12.0.5 opaque secret-string boundary.

pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(feature = "retail-12-0-5")]
    rilua::LuaApiMut::register_function(lua, "Ambiguate", ambiguate)?;
    #[cfg(not(feature = "retail-12-0-5"))]
    lua.exec(
        r#"
function Ambiguate(fullName, context)
    if context == "none" then return fullName end
    return string.match(fullName, "^(.-)%-.+$") or fullName
end
"#,
    )?;
    Ok(())
}

#[cfg(feature = "retail-12-0-5")]
fn ambiguate(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    use crate::lua_bridge::stack_val;
    use rilua::table_security::{is_secret_value, transform_host_secret_string};
    use rilua::{Val, runtime_error};

    let full_name = stack_val(state, 1);
    let context = stack_val(state, 2);
    if is_secret_value(state, context) {
        return Err(runtime_error("Ambiguate argument #2 must not be secret"));
    }
    let Val::Str(context) = context else {
        return Err(runtime_error("Ambiguate argument #2 must be a string"));
    };
    let keep_realm = state
        .gc
        .string_arena
        .get(context)
        .is_some_and(|s| s.data() == b"none");
    let result = if is_secret_value(state, full_name) {
        transform_host_secret_string(state, full_name, |bytes| {
            Ok(shorten_name(bytes, keep_realm))
        })?
    } else {
        let Val::Str(name) = full_name else {
            return Err(runtime_error("Ambiguate argument #1 must be a string"));
        };
        let bytes = state.gc.string_arena.get(name).expect("rooted name").data();
        let output = shorten_name(bytes, keep_realm);
        Val::Str(state.gc.intern_string(&output))
    };
    state.push(result);
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
fn shorten_name(bytes: &[u8], keep_realm: bool) -> Vec<u8> {
    if keep_realm {
        return bytes.to_vec();
    }
    // Preserve the former Lua pattern: a realm separator needs a nonempty suffix.
    match bytes.iter().position(|byte| *byte == b'-') {
        Some(separator) if separator + 1 < bytes.len() => bytes[..separator].to_vec(),
        _ => bytes.to_vec(),
    }
}
