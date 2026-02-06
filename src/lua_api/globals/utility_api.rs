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
    let globals = lua.globals();

    // wipe(table) - Clear a table in place
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

    // tinsert - alias for table.insert
    globals.set(
        "tinsert",
        lua.create_function(|lua, args: mlua::MultiValue| {
            let table_insert: mlua::Function =
                lua.globals().get::<mlua::Table>("table")?.get("insert")?;
            table_insert.call::<()>(args)?;
            Ok(())
        })?,
    )?;

    // tremove - alias for table.remove
    globals.set(
        "tremove",
        lua.create_function(|lua, args: mlua::MultiValue| {
            let table_remove: mlua::Function =
                lua.globals().get::<mlua::Table>("table")?.get("remove")?;
            table_remove.call::<Value>(args)
        })?,
    )?;

    // tInvert - invert table (swap keys and values)
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

    // tContains - check if table contains value
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

    // tIndexOf - get index of value in array-like table
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

    // tFilter - filter table with predicate (in-place)
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

    // CopyTable - deep copy a table
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

    // MergeTable - merge source into dest
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
                Err(e) => Ok((Value::Nil, Value::String(lua.create_string(&e.to_string())?))),
            }
        })?,
    )?;

    globals.set(
        "GetCurrentEnvironment",
        lua.create_function(|lua, ()| Ok(lua.globals()))?,
    )?;

    Ok(())
}

/// Security functions: issecure, issecurevariable, securecall, securecallfunction,
/// secureexecuterange, forceinsecure, hooksecurefunc, SecureHandler stubs,
/// state/attribute driver stubs, SecureCmdOptionParse.
fn register_security_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set("issecure", lua.create_function(|_, ()| Ok(false))?)?;

    globals.set(
        "issecurevariable",
        lua.create_function(|_, (_table, _var): (Option<Value>, String)| Ok((true, Value::Nil)))?,
    )?;

    globals.set(
        "securecall",
        lua.create_function(|_, (func, args): (mlua::Function, mlua::MultiValue)| {
            func.call::<mlua::MultiValue>(args)
        })?,
    )?;

    globals.set(
        "securecallfunction",
        lua.create_function(|_, (func, args): (mlua::Function, mlua::MultiValue)| {
            func.call::<mlua::MultiValue>(args)
        })?,
    )?;

    globals.set("forceinsecure", lua.create_function(|_, ()| Ok(()))?)?;

    register_hooksecurefunc(lua)?;
    register_secureexecuterange(lua)?;
    register_secure_handler_stubs(lua)?;

    // SecureCmdOptionParse - returns the default (last) option
    globals.set(
        "SecureCmdOptionParse",
        lua.create_function(|lua, options: String| {
            if let Some(last) = options.split(';').last() {
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
                for pair in tbl.pairs::<Value, Value>() {
                    if let Ok((key, value)) = pair {
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

/// Error handler functions: geterrorhandler, seterrorhandler.
fn register_error_handlers(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "geterrorhandler",
        lua.create_function(|lua, ()| {
            let handler = lua.create_function(|_, msg: String| {
                println!("Lua error: {}", msg);
                Ok(())
            })?;
            Ok(handler)
        })?,
    )?;

    globals.set(
        "seterrorhandler",
        lua.create_function(|_, _handler: mlua::Function| Ok(()))?,
    )?;

    Ok(())
}

/// Misc stubs: nop, sound functions.
fn register_misc_stubs(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let nop = lua.create_function(|_, _: mlua::MultiValue| Ok(()))?;
    globals.set("nop", nop.clone())?;
    globals.set("PlaySound", nop.clone())?;
    globals.set("StopSound", nop.clone())?;
    globals.set("PlaySoundFile", nop)?;
    Ok(())
}

/// Lua stdlib global aliases (string, math, table, bit) for WoW compatibility.
fn register_lua_stdlib_aliases(lua: &Lua) -> Result<()> {
    lua.load(
        r##"
        -- String library aliases
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

        -- Add string:split method (WoW extension)
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

        -- Math library aliases
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

        -- Table library aliases
        sort = table.sort
        getn = function(t) return #t end
        tconcat = table.concat

        -- Bitwise operations (Lua 5.1 bit library compatibility)
        bit = bit or {}
        bit.band = function(a, b)
            local result = 0
            local bitval = 1
            while a > 0 and b > 0 do
                if a % 2 == 1 and b % 2 == 1 then
                    result = result + bitval
                end
                bitval = bitval * 2
                a = math.floor(a / 2)
                b = math.floor(b / 2)
            end
            return result
        end
        bit.bor = function(a, b)
            local result = 0
            local bitval = 1
            while a > 0 or b > 0 do
                if a % 2 == 1 or b % 2 == 1 then
                    result = result + bitval
                end
                bitval = bitval * 2
                a = math.floor(a / 2)
                b = math.floor(b / 2)
            end
            return result
        end
        bit.bxor = function(a, b)
            local result = 0
            local bitval = 1
            while a > 0 or b > 0 do
                if (a % 2 == 1) ~= (b % 2 == 1) then
                    result = result + bitval
                end
                bitval = bitval * 2
                a = math.floor(a / 2)
                b = math.floor(b / 2)
            end
            return result
        end
        bit.bnot = function(a)
            -- 32-bit not
            return 4294967295 - a
        end
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
