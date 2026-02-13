//! Security-related WoW API functions.
//!
//! Contains securecall, securecallmethod, securecallfunction, hooksecurefunc,
//! secureexecuterange, SecureHandler stubs, state/attribute driver stubs,
//! and SecureCmdOptionParse.

use mlua::{Lua, Result, Value};

/// Register all security-related API functions.
pub fn register_security_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("issecure", lua.create_function(|_, ()| Ok(true))?)?;

    globals.set(
        "issecurevariable",
        lua.create_function(|_, (_table, _var): (Option<Value>, String)| Ok((true, Value::Nil)))?,
    )?;

    globals.set("securecall", lua.create_function(securecall_impl)?)?;
    globals.set("securecallfunction", lua.create_function(securecall_impl)?)?;
    globals.set("securecallmethod", lua.create_function(securecallmethod_impl)?)?;

    globals.set("forceinsecure", lua.create_function(|_, ()| Ok(()))?)?;
    globals.set("issecretvalue", lua.create_function(|_, _val: Value| Ok(false))?)?;
    globals.set("canaccessvalue", lua.create_function(|_, _val: Value| Ok(true))?)?;
    globals.set(
        "canaccessallvalues",
        lua.create_function(|_, _vals: mlua::MultiValue| Ok(true))?,
    )?;
    globals.set("canaccesstable", lua.create_function(|_, _val: Value| Ok(true))?)?;

    register_hooksecurefunc(lua)?;
    register_secureexecuterange(lua)?;
    register_secure_handler_stubs(lua)?;

    // SecureCmdOptionParse - returns the default (last) option
    globals.set(
        "SecureCmdOptionParse",
        lua.create_function(|lua, options: String| {
            if let Some(last) = options.split(';').next_back() {
                Ok(Value::String(lua.create_string(last.trim())?))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;

    Ok(())
}

/// securecall/securecallfunction implementation.
/// Accepts a function or a string name (resolved from _G).
fn securecall_impl(lua: &Lua, args: mlua::MultiValue) -> Result<mlua::MultiValue> {
    let mut args_iter = args.into_iter();
    let func_or_name = args_iter.next().unwrap_or(Value::Nil);
    let remaining = mlua::MultiValue::from_vec(args_iter.collect());
    match func_or_name {
        Value::Function(f) => f.call::<mlua::MultiValue>(remaining),
        Value::String(s) => {
            let name = s.to_str()?;
            match lua.globals().get::<Value>(name)? {
                Value::Function(f) => f.call::<mlua::MultiValue>(remaining),
                _ => Ok(mlua::MultiValue::new()),
            }
        }
        _ => Ok(mlua::MultiValue::new()),
    }
}

/// securecallmethod(object, methodName, ...) → object:methodName(...)
fn securecallmethod_impl(_lua: &Lua, args: mlua::MultiValue) -> Result<mlua::MultiValue> {
    let mut it = args.into_iter();
    let obj = match it.next() {
        Some(Value::Table(t)) => t,
        _ => return Ok(mlua::MultiValue::new()),
    };
    let method_name = match it.next() {
        Some(Value::String(s)) => s,
        _ => return Ok(mlua::MultiValue::new()),
    };
    let remaining: Vec<Value> = it.collect();
    match obj.get::<Value>(method_name)? {
        Value::Function(f) => {
            let mut call_args = vec![Value::Table(obj)];
            call_args.extend(remaining);
            f.call::<mlua::MultiValue>(mlua::MultiValue::from_iter(call_args))
        }
        _ => Ok(mlua::MultiValue::new()),
    }
}

/// hooksecurefunc(name, hook) or hooksecurefunc(table, name, hook).
fn register_hooksecurefunc(lua: &Lua) -> Result<()> {
    lua.globals().set(
        "hooksecurefunc",
        lua.create_function(|lua, args: mlua::MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();

            let (table, name, hook) = if args.len() == 2 {
                let name = if let Value::String(s) = &args[0] {
                    s.to_string_lossy().to_string()
                } else {
                    String::new()
                };
                (lua.globals(), name, args[1].clone())
            } else if args.len() >= 3 {
                let table = if let Value::Table(t) = &args[0] {
                    t.clone()
                } else {
                    lua.globals()
                };
                let name = if let Value::String(s) = &args[1] {
                    s.to_string_lossy().to_string()
                } else {
                    String::new()
                };
                (table, name, args[2].clone())
            } else {
                return Ok(());
            };

            let original: Value = table.get::<Value>(name.as_str())?;
            if let (Value::Function(orig_fn), Value::Function(hook_fn)) = (original, hook) {
                let wrapper = lua.create_function(move |_, args: mlua::MultiValue| {
                    let result = orig_fn.call::<mlua::MultiValue>(args.clone())?;
                    let _ = hook_fn.call::<mlua::MultiValue>(args);
                    Ok(result)
                })?;
                table.set(name.as_str(), wrapper)?;
            }

            Ok(())
        })?,
    )?;
    Ok(())
}

/// secureexecuterange(tbl, func, ...) - calls func(key, value, ...) for each entry.
fn register_secureexecuterange(lua: &Lua) -> Result<()> {
    lua.globals().set(
        "secureexecuterange",
        lua.create_function(
            |_, (tbl, func, args): (mlua::Table, mlua::Function, mlua::MultiValue)| {
                for (key, value) in tbl.pairs::<Value, Value>().flatten() {
                    let mut call_args = mlua::MultiValue::new();
                    call_args.push_front(value);
                    call_args.push_front(key);
                    for arg in args.iter() {
                        call_args.push_back(arg.clone());
                    }
                    if let Err(e) = func.call::<()>(call_args) {
                        tracing::warn!("secureexecuterange callback error: {}", e);
                    }
                }
                Ok(())
            },
        )?,
    )?;
    Ok(())
}

/// SecureHandler stubs and state/attribute driver stubs.
fn register_secure_handler_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "SecureHandlerSetFrameRef",
        lua.create_function(|_, (_frame, _name, _target): (Value, String, Value)| Ok(()))?,
    )?;
    globals.set(
        "SecureHandlerExecute",
        lua.create_function(|_, (_frame, _body, _args): (Value, String, mlua::MultiValue)| {
            Ok(())
        })?,
    )?;
    globals.set(
        "SecureHandlerWrapScript",
        lua.create_function(|_, (_frame, _script, _body): (Value, String, String)| Ok(()))?,
    )?;

    globals.set(
        "RegisterStateDriver",
        lua.create_function(|_, (_frame, _attr, _driver): (Value, String, String)| Ok(()))?,
    )?;
    globals.set(
        "UnregisterStateDriver",
        lua.create_function(|_, (_frame, _attr): (Value, String)| Ok(()))?,
    )?;
    globals.set(
        "RegisterAttributeDriver",
        lua.create_function(|_, (_frame, _attr, _driver): (Value, String, String)| Ok(()))?,
    )?;
    globals.set(
        "UnregisterAttributeDriver",
        lua.create_function(|_, (_frame, _attr): (Value, String)| Ok(()))?,
    )?;

    Ok(())
}
