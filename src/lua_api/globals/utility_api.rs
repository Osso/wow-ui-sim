//! Utility functions for WoW API.
//!
//! Contains table manipulation functions (wipe, tinsert, tremove, tContains, etc.),
//! string utilities (strsplit, strjoin), and other general-purpose functions.

use mlua::{Lua, Result, Value};

/// Register all utility API functions.
pub fn register_utility_api(lua: &Lua) -> Result<()> {
    register_table_functions(lua)?;
    register_string_functions(lua)?;
    register_global_access(lua)?;
    register_security_functions(lua)?;
    register_error_handlers(lua)?;
    register_misc_stubs(lua)?;
    register_lua_stdlib_aliases(lua)?;
    register_mixin_system(lua)?;
    Ok(())
}

/// Table manipulation: wipe, tinsert, tremove, tInvert, tContains, tIndexOf,
/// tFilter, CopyTable, MergeTable.
fn register_table_functions(lua: &Lua) -> Result<()> {
    register_wipe_and_aliases(lua)?;
    register_table_search(lua)?;
    register_table_transform(lua)?;
    Ok(())
}

/// wipe, tinsert, tremove - core table mutation functions.
fn register_wipe_and_aliases(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    let wipe = lua.create_function(|_, table: mlua::Table| {
        let keys: Vec<Value> = table
            .pairs::<Value, Value>()
            .filter_map(|r| r.ok().map(|(k, _)| k))
            .collect();
        for key in keys {
            table.set(key, Value::Nil)?;
        }
        Ok(table)
    })?;
    globals.set("wipe", wipe.clone())?;
    let table_lib: mlua::Table = globals.get("table")?;
    table_lib.set("wipe", wipe)?;

    globals.set(
        "tinsert",
        lua.create_function(|lua, args: mlua::MultiValue| {
            let table_insert: mlua::Function =
                lua.globals().get::<mlua::Table>("table")?.get("insert")?;
            table_insert.call::<()>(args)?;
            Ok(())
        })?,
    )?;

    globals.set(
        "tremove",
        lua.create_function(|lua, args: mlua::MultiValue| {
            let table_remove: mlua::Function =
                lua.globals().get::<mlua::Table>("table")?.get("remove")?;
            table_remove.call::<Value>(args)
        })?,
    )?;

    Ok(())
}

/// tInvert, tContains, tIndexOf, tFilter - table search/filter functions.
fn register_table_search(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "tInvert",
        lua.create_function(|lua, tbl: mlua::Table| {
            let result = lua.create_table()?;
            for pair in tbl.pairs::<Value, Value>() {
                let (k, v) = pair?;
                result.set(v, k)?;
            }
            Ok(result)
        })?,
    )?;

    globals.set(
        "tContains",
        lua.create_function(|_, (tbl, value): (mlua::Table, Value)| {
            for pair in tbl.pairs::<Value, Value>() {
                let (_, v) = pair?;
                if v == value {
                    return Ok(true);
                }
            }
            Ok(false)
        })?,
    )?;

    globals.set(
        "tIndexOf",
        lua.create_function(|_, (tbl, value): (mlua::Table, Value)| {
            for pair in tbl.pairs::<i32, Value>() {
                let (k, v) = pair?;
                if v == value {
                    return Ok(Value::Integer(k as i64));
                }
            }
            Ok(Value::Nil)
        })?,
    )?;

    globals.set(
        "tFilter",
        lua.create_function(
            |_, (tbl, pred, _keep_order): (mlua::Table, mlua::Function, Option<bool>)| {
                let mut to_remove = Vec::new();
                for pair in tbl.pairs::<Value, Value>() {
                    let (k, v) = pair?;
                    let keep: bool = pred.call((v.clone(),))?;
                    if !keep {
                        to_remove.push(k);
                    }
                }
                for k in to_remove {
                    tbl.set(k, Value::Nil)?;
                }
                Ok(tbl)
            },
        )?,
    )?;

    Ok(())
}

/// CopyTable, MergeTable - table copy/merge functions.
fn register_table_transform(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "CopyTable",
        lua.create_function(|lua, (tbl, seen): (mlua::Table, Option<mlua::Table>)| {
            let seen = seen.unwrap_or_else(|| lua.create_table().unwrap());
            let result = lua.create_table()?;
            seen.set(tbl.clone(), result.clone())?;
            for pair in tbl.pairs::<Value, Value>() {
                let (k, v) = pair?;
                let new_v = if let Value::Table(inner) = v.clone() {
                    if let Ok(cached) = seen.get::<mlua::Table>(inner.clone()) {
                        Value::Table(cached)
                    } else {
                        let copy_table: mlua::Function = lua.globals().get("CopyTable")?;
                        copy_table.call((inner, seen.clone()))?
                    }
                } else {
                    v
                };
                result.set(k, new_v)?;
            }
            Ok(result)
        })?,
    )?;

    globals.set(
        "MergeTable",
        lua.create_function(|_, (dest, source): (mlua::Table, mlua::Table)| {
            for pair in source.pairs::<Value, Value>() {
                let (k, v) = pair?;
                dest.set(k, v)?;
            }
            Ok(dest)
        })?,
    )?;

    Ok(())
}

/// String functions: strsplit.
fn register_string_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // strsplit(delimiter, str, limit) - WoW string utility
    globals.set(
        "strsplit",
        lua.create_function(|lua, args: mlua::MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();

            let delimiter = args
                .first()
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| " ".to_string());

            let input = args
                .get(1)
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let limit = args.get(2).and_then(|v| match v {
                Value::Integer(n) => Some(*n as usize),
                Value::Number(n) => Some(*n as usize),
                _ => None,
            });

            let parts: Vec<&str> = if let Some(limit) = limit {
                input.splitn(limit, &delimiter).collect()
            } else {
                input.split(&delimiter).collect()
            };

            let mut result = mlua::MultiValue::new();
            for part in parts {
                result.push_back(Value::String(lua.create_string(part)?));
            }
            Ok(result)
        })?,
    )?;

    Ok(())
}

/// Global access: getglobal, setglobal, loadstring, GetCurrentEnvironment.
fn register_global_access(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "getglobal",
        lua.create_function(|lua, name: String| {
            let value: Value = lua.globals().get(name.as_str()).unwrap_or(Value::Nil);
            Ok(value)
        })?,
    )?;

    globals.set(
        "setglobal",
        lua.create_function(|lua, (name, value): (String, Value)| {
            lua.globals().set(name.as_str(), value)?;
            Ok(())
        })?,
    )?;

    // loadstring(code, name) - Compile a string of Lua code and return it as a function
    globals.set(
        "loadstring",
        lua.create_function(|lua, (code, name): (String, Option<String>)| {
            let chunk_name = name.unwrap_or_else(|| "=(loadstring)".to_string());
            match lua.load(&code).set_name(&chunk_name).into_function() {
                Ok(func) => Ok((Value::Function(func), Value::Nil)),
                Err(e) => Ok((Value::Nil, Value::String(lua.create_string(e.to_string())?))),
            }
        })?,
    )?;

    globals.set(
        "GetCurrentEnvironment",
        lua.create_function(|lua, ()| Ok(lua.globals()))?,
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

/// Security functions: issecure, issecurevariable, securecall, securecallfunction,
/// secureexecuterange, forceinsecure, hooksecurefunc, SecureHandler stubs,
/// state/attribute driver stubs, SecureCmdOptionParse.
fn register_security_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("issecure", lua.create_function(|_, ()| Ok(true))?)?;

    globals.set(
        "issecurevariable",
        lua.create_function(|_, (_table, _var): (Option<Value>, String)| Ok((true, Value::Nil)))?,
    )?;

    globals.set("securecall", lua.create_function(securecall_impl)?)?;
    globals.set("securecallfunction", lua.create_function(securecall_impl)?)?;

    globals.set("forceinsecure", lua.create_function(|_, ()| Ok(()))?)?;

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

fn arg_to_i32(v: &Value) -> Option<i32> {
    match v {
        Value::Integer(n) => Some(*n as i32),
        Value::Number(n) => Some(*n as i32),
        _ => None,
    }
}

/// debuglocals(level, skipFunctionsAndUserdata) - returns a string of local variables.
/// Stub: returns empty string. Only used by Blizzard_ScriptErrors for error display.
fn register_debuglocals(lua: &Lua) -> Result<()> {
    lua.globals().set(
        "debuglocals",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(String::new()))?,
    )
}

/// debugstack(start, count1, count2) - returns a stack trace string.
/// WoW's debugstack is used by error handlers and BugSack.
fn register_debugstack(lua: &Lua) -> Result<()> {
    lua.globals().set(
        "debugstack",
        lua.create_function(|lua, args: mlua::MultiValue| {
            let start = args.front().and_then(arg_to_i32).unwrap_or(2);
            let count1 = args.get(1).and_then(arg_to_i32).unwrap_or(12) as usize;
            let count2 = args.get(2).and_then(arg_to_i32).unwrap_or(10) as usize;
            let tb: mlua::Function =
                lua.globals().get::<mlua::Table>("debug")?.get("traceback")?;
            let trace: String = tb.call(("", start))?;
            let lines: Vec<&str> = trace.lines().filter(|l| !l.is_empty()).collect();
            let total = lines.len();
            if total <= count1 + count2 {
                Ok(lines.join("\n"))
            } else {
                let top = &lines[..count1];
                let bottom = &lines[total - count2..];
                Ok(format!("{}\n...\n{}", top.join("\n"), bottom.join("\n")))
            }
        })?,
    )
}

/// Error handler functions: geterrorhandler, seterrorhandler.
///
/// Stores the handler in the Lua registry under `__wow_error_handler`.
/// Script dispatch errors are routed through this handler (see script_helpers).
fn register_error_handlers(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    register_debugstack(lua)?;
    register_debuglocals(lua)?;

    globals.set(
        "geterrorhandler",
        lua.create_function(|lua, ()| {
            let handler: Value =
                lua.named_registry_value("__wow_error_handler").unwrap_or(Value::Nil);
            if let Value::Function(f) = handler {
                Ok(Value::Function(f))
            } else {
                // Return default handler that prints to stderr
                let default = lua.create_function(|_, msg: String| {
                    eprintln!("Lua error: {}", msg);
                    Ok(())
                })?;
                Ok(Value::Function(default))
            }
        })?,
    )?;

    globals.set(
        "seterrorhandler",
        lua.create_function(|lua, handler: mlua::Function| {
            lua.set_named_registry_value("__wow_error_handler", handler)?;
            Ok(())
        })?,
    )?;

    Ok(())
}

/// Misc stubs: nop function.
fn register_misc_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let nop = lua.create_function(|_, _: mlua::MultiValue| Ok(()))?;
    globals.set("nop", nop)?;
    Ok(())
}

/// Lua stdlib global aliases (string, math, table, bit, os) for WoW compatibility.
fn register_lua_stdlib_aliases(lua: &Lua) -> Result<()> {
    register_string_aliases(lua)?;
    register_math_aliases(lua)?;
    register_table_aliases(lua)?;
    register_bit_library(lua)?;
    register_os_aliases(lua)?;
    Ok(())
}

/// String library global aliases.
fn register_string_aliases(lua: &Lua) -> Result<()> {
    lua.load(
        r##"
        strlen = string.len
        strsub = string.sub
        strfind = string.find
        strmatch = string.match
        strbyte = string.byte
        strchar = string.char
        strrep = string.rep
        strrev = string.reverse
        strlower = string.lower
        strupper = string.upper
        strtrim = function(s) return (s:gsub("^%s*(.-)%s*$", "%1")) end
        strsplittable = function(del, str) local t = {} for v in string.gmatch(str, "([^"..del.."]+)") do t[#t+1] = v end return t end
        strjoin = function(delimiter, ...) return table.concat({...}, delimiter) end
        string.join = strjoin
        format = string.format

        function string:split(delimiter)
            local result = {}
            local from = 1
            local delim_from, delim_to = string.find(self, delimiter, from, true)
            while delim_from do
                table.insert(result, string.sub(self, from, delim_from - 1))
                from = delim_to + 1
                delim_from, delim_to = string.find(self, delimiter, from, true)
            end
            table.insert(result, string.sub(self, from))
            return result
        end
        gsub = string.gsub
        gmatch = string.gmatch
    "##,
    )
    .exec()?;
    Ok(())
}

/// Math library global aliases.
fn register_math_aliases(lua: &Lua) -> Result<()> {
    lua.load(
        r##"
        abs = math.abs
        ceil = math.ceil
        floor = math.floor
        max = math.max
        min = math.min
        mod = math.fmod
        sqrt = math.sqrt
        sin = math.sin
        cos = math.cos
        tan = math.tan
        asin = math.asin
        acos = math.acos
        atan = math.atan
        atan2 = math.atan2
        deg = math.deg
        rad = math.rad
        random = math.random
        exp = math.exp
        log = math.log
        log10 = math.log10
        pow = math.pow
        frexp = math.frexp
        ldexp = math.ldexp

        sort = table.sort
        getn = function(t) return #t end
        tconcat = table.concat
    "##,
    )
    .exec()?;
    Ok(())
}

/// Table library global aliases.
fn register_table_aliases(_lua: &Lua) -> Result<()> {
    // Already registered in register_math_aliases (sort, getn, tconcat)
    Ok(())
}

/// OS library global aliases: date, time, difftime, clock.
fn register_os_aliases(lua: &Lua) -> Result<()> {
    lua.load(
        r##"
        date = os.date
        time = os.time
        difftime = os.difftime
        clock = os.clock
    "##,
    )
    .exec()?;
    Ok(())
}

/// Bitwise operations (Lua 5.1 bit library compatibility).
fn register_bit_library(lua: &Lua) -> Result<()> {
    register_bit_logic_ops(lua)?;
    register_bit_shift_ops(lua)?;
    Ok(())
}

/// bit.band, bit.bor, bit.bxor - bitwise logic operations.
fn register_bit_logic_ops(lua: &Lua) -> Result<()> {
    lua.load(
        r##"
        bit = bit or {}
        bit.band = function(a, b)
            local result, bitval = 0, 1
            while a > 0 and b > 0 do
                if a % 2 == 1 and b % 2 == 1 then result = result + bitval end
                bitval = bitval * 2
                a = math.floor(a / 2)
                b = math.floor(b / 2)
            end
            return result
        end
        bit.bor = function(a, b)
            local result, bitval = 0, 1
            while a > 0 or b > 0 do
                if a % 2 == 1 or b % 2 == 1 then result = result + bitval end
                bitval = bitval * 2
                a = math.floor(a / 2)
                b = math.floor(b / 2)
            end
            return result
        end
        bit.bxor = function(a, b)
            local result, bitval = 0, 1
            while a > 0 or b > 0 do
                if (a % 2 == 1) ~= (b % 2 == 1) then result = result + bitval end
                bitval = bitval * 2
                a = math.floor(a / 2)
                b = math.floor(b / 2)
            end
            return result
        end
    "##,
    )
    .exec()?;
    Ok(())
}

/// bit.bnot, bit.lshift, bit.rshift - bitwise not and shift operations.
fn register_bit_shift_ops(lua: &Lua) -> Result<()> {
    lua.load(
        r##"
        bit.bnot = function(a) return 4294967295 - a end
        bit.lshift = function(a, n) return a * (2 ^ n) end
        bit.rshift = function(a, n) return math.floor(a / (2 ^ n)) end
    "##,
    )
    .exec()?;
    Ok(())
}

/// Mixin system: Mixin, CreateFromMixins, CreateAndInitFromMixin.
fn register_mixin_system(lua: &Lua) -> Result<()> {
    lua.load(
        r##"
        function Mixin(object, ...)
            for i = 1, select("#", ...) do
                local mixin = select(i, ...)
                if mixin then
                    for k, v in pairs(mixin) do
                        object[k] = v
                    end
                end
            end
            return object
        end

        function CreateFromMixins(...)
            return Mixin({}, ...)
        end

        function CreateAndInitFromMixin(mixin, ...)
            local object = CreateFromMixins(mixin)
            if object.Init then
                object:Init(...)
            end
            return object
        end
    "##,
    )
    .exec()?;
    Ok(())
}
