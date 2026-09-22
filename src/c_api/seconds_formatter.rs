//! C API-owned opaque SecondsFormatter handles and private native rendering.
#[cfg(feature = "native-duration-formatting")]
mod render;
#[cfg(feature = "native-duration-formatting")]
mod units;

use crate::lua_api::methods::{create_table, table_set, table_set_static};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::closure::{Closure, RustClosure};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

fn publish_values(
    state: &mut LuaState,
    name: &'static str,
    fields: &[(&'static str, f64)],
) -> LuaResult<()> {
    let enums = super::helpers::ensure_namespace(state, "Enum")?;
    let values = create_table(state);
    state.push(values);
    for &(key, value) in fields {
        table_set_static(state, values, key, Val::Num(value));
    }
    table_set_static(state, Val::Table(enums), name, values);
    state.pop();
    Ok(())
}

fn publish_metadata(
    state: &mut LuaState,
    namespace: &'static str,
    name: &'static str,
) -> LuaResult<()> {
    let namespace = super::helpers::ensure_namespace(state, namespace)?;
    let metadata = create_table(state);
    state.push(metadata);
    for (key, value) in [("MinValue", 0.0), ("MaxValue", 2.0), ("NumValues", 3.0)] {
        table_set_static(state, metadata, key, Val::Num(value));
    }
    table_set_static(state, Val::Table(namespace), name, metadata);
    state.pop();
    Ok(())
}

pub(crate) fn register_enums(state: &mut LuaState) -> LuaResult<()> {
    if cfg!(any(
        feature = "retail-12-1-0",
        feature = "client-wowforever"
    )) {
        publish_values(
            state,
            "SecondsFormatterIntervalWhitespace",
            &[
                ("Preserve", 0.0),
                ("Strip", 1.0),
                ("StripIgnoreLocale", 2.0),
            ],
        )?;
        publish_metadata(state, "EnumMeta", "SecondsFormatterIntervalWhitespace")?;
    }
    #[cfg(feature = "native-duration-formatting")]
    {
        publish_values(
            state,
            "SecondsFormatterAbbreviation",
            &[("None", 0.0), ("Truncate", 1.0), ("OneLetter", 2.0)],
        )?;
        publish_metadata(state, "Enum", "SecondsFormatterAbbreviationMeta")?;
    }
    Ok(())
}

fn private_callback(state: &mut LuaState, name: &'static str, function: rilua::RustFn) -> Val {
    let closure = Closure::Rust(RustClosure::new(function, name));
    let callback = Val::Function(state.gc.alloc_closure(closure));
    state.push(callback);
    callback
}

fn new_configuration(state: &mut LuaState) -> LuaResult<u32> {
    let values = create_table(state);
    state.push(values);
    for (key, value) in [
        ("approximationSeconds", 0.0),
        ("millisecondsThreshold", 0.0),
        ("stripIntervalWhitespace", 0.0),
        ("minInterval", 0.0),
        ("maxInterval", 3.0),
        ("desiredUnitCount", 1.0),
    ] {
        table_set_static(state, values, key, Val::Num(value));
    }
    Ok(1)
}

fn write_configuration(state: &mut LuaState) -> LuaResult<u32> {
    let values @ Val::Table(_) = stack_val(state, 1) else {
        return Err(runtime_error(
            "SecondsFormatter requires private configuration storage",
        ));
    };
    let key = String::from_stack(state, 2)?;
    let valid = matches!(
        key.as_str(),
        "approximationSeconds"
            | "millisecondsThreshold"
            | "stripIntervalWhitespace"
            | "minInterval"
            | "maxInterval"
            | "desiredUnitCount"
            | "defaultAbbreviation"
            | "rounding"
            | "canRoundUpLastUnit"
            | "maxIntervalCurve"
    );
    if !valid {
        return Err(runtime_error(
            "unknown SecondsFormatter configuration field",
        ));
    }
    let value = stack_val(state, 3);
    if rilua::table_security::is_secret_value(state, value) {
        return Err(runtime_error(
            "secret SecondsFormatter configuration is not modeled",
        ));
    }
    // Native-owned configuration is not an addon Lua slot. This private write
    // never unwraps values or clears stack/closure taint; callback objects keep
    // their original identity and taint. Format input has its own checked path.
    table_set(state, values, &key, value);
    Ok(0)
}

fn read_number(state: &mut LuaState) -> LuaResult<u32> {
    let input = stack_val(state, 1);
    let secret = rilua::table_security::is_secret_value(state, input);
    let value = rilua::table_security::unwrap_secret(state, input)?;
    let Val::Num(number) = value else {
        return Err(runtime_error("SecondsFormatter requires a finite number"));
    };
    if !number.is_finite() {
        return Err(runtime_error("SecondsFormatter requires a finite number"));
    }
    state.push(Val::Num(number));
    state.push(Val::Bool(secret));
    Ok(2)
}

fn wrap_value(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::wrap_secret(state, stack_val(state, 1))?;
    state.push(value);
    Ok(1)
}

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    let source = concat!(
        include_str!("seconds_formatter/format.lua"),
        "\n",
        include_str!("seconds_formatter/configuration.lua"),
    );
    let function = lua.load_bytes(source.as_bytes(), "@seconds-formatter-bootstrap")?;
    let state = lua.state_mut();
    let saved_top = state.top;
    state.push(Val::Function(function.gc_ref()));
    let arguments = [
        private_callback(
            state,
            "SecondsFormatter.NewConfiguration",
            new_configuration,
        ),
        private_callback(
            state,
            "SecondsFormatter.WriteConfiguration",
            write_configuration,
        ),
        private_callback(state, "SecondsFormatter.ReadNumber", read_number),
        private_callback(state, "SecondsFormatter.WrapValue", wrap_value),
        renderer(state),
    ];
    let result = lua.call_function(&function, &arguments);
    lua.state_mut().top = saved_top;
    result?;
    Ok(())
}

fn renderer(state: &mut LuaState) -> Val {
    #[cfg(feature = "native-duration-formatting")]
    {
        render::callback(state)
    }
    #[cfg(not(feature = "native-duration-formatting"))]
    {
        let _ = state;
        Val::Nil
    }
}
