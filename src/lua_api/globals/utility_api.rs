//! Utility functions for WoW API.
//!
//! Contains table manipulation functions (wipe, tinsert, tremove, tContains, etc.),
//! string utilities (strsplit, strjoin), and other general-purpose functions.

use mlua::{Lua, Result, Value};

/// Register all utility API functions.
pub fn register_utility_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // strsplit(delimiter, str, limit) - WoW string utility
    let strsplit = lua.create_function(|lua, args: mlua::MultiValue| {
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

        let limit = args.get(2).and_then(|v| {
            if let Value::Integer(n) = v {
                Some(*n as usize)
            } else if let Value::Number(n) = v {
                Some(*n as usize)
            } else {
                None
            }
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
    })?;
    globals.set("strsplit", strsplit)?;

    // getglobal(name) - Get a global variable by name (old WoW API)
    let getglobal_fn = lua.create_function(|lua, name: String| {
        let globals = lua.globals();
        let value: Value = globals.get(name.as_str()).unwrap_or(Value::Nil);
        Ok(value)
    })?;
    globals.set("getglobal", getglobal_fn)?;

    // setglobal(name, value) - Set a global variable by name (old WoW API)
    let setglobal_fn = lua.create_function(|lua, (name, value): (String, Value)| {
        lua.globals().set(name.as_str(), value)?;
        Ok(())
    })?;
    globals.set("setglobal", setglobal_fn)?;

    // loadstring(code, name) - Compile a string of Lua code and return it as a function
    // This is a Lua 5.1 function that WoW uses (replaced by load() in Lua 5.2+)
    let loadstring_fn = lua.create_function(|lua, (code, name): (String, Option<String>)| {
        let chunk_name = name.unwrap_or_else(|| "=(loadstring)".to_string());
        match lua.load(&code).set_name(&chunk_name).into_function() {
            Ok(func) => Ok((Value::Function(func), Value::Nil)),
            Err(e) => Ok((Value::Nil, Value::String(lua.create_string(&e.to_string())?))),
        }
    })?;
    globals.set("loadstring", loadstring_fn)?;

    // wipe(table) - Clear a table in place
    let wipe = lua.create_function(|_, table: mlua::Table| {
        // Get all keys first to avoid modification during iteration
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

    // Also set table.wipe for convenience
    let table_lib: mlua::Table = globals.get("table")?;
    table_lib.set("wipe", wipe)?;

    // tinsert - alias for table.insert
    let tinsert = lua.create_function(|lua, args: mlua::MultiValue| {
        let table_insert: mlua::Function =
            lua.globals().get::<mlua::Table>("table")?.get("insert")?;
        table_insert.call::<()>(args)?;
        Ok(())
    })?;
    globals.set("tinsert", tinsert)?;

    // tremove - alias for table.remove
    let tremove = lua.create_function(|lua, args: mlua::MultiValue| {
        let table_remove: mlua::Function =
            lua.globals().get::<mlua::Table>("table")?.get("remove")?;
        table_remove.call::<Value>(args)
    })?;
    globals.set("tremove", tremove)?;

    // tInvert - invert table (swap keys and values)
    let tinvert = lua.create_function(|lua, tbl: mlua::Table| {
        let result = lua.create_table()?;
        for pair in tbl.pairs::<Value, Value>() {
            let (k, v) = pair?;
            result.set(v, k)?;
        }
        Ok(result)
    })?;
    globals.set("tInvert", tinvert)?;

    // tContains - check if table contains value
    let tcontains = lua.create_function(|_, (tbl, value): (mlua::Table, Value)| {
        for pair in tbl.pairs::<Value, Value>() {
            let (_, v) = pair?;
            if v == value {
                return Ok(true);
            }
        }
        Ok(false)
    })?;
    globals.set("tContains", tcontains)?;

    // tIndexOf - get index of value in array-like table
    let tindexof = lua.create_function(|_, (tbl, value): (mlua::Table, Value)| {
        for pair in tbl.pairs::<i32, Value>() {
            let (k, v) = pair?;
            if v == value {
                return Ok(Value::Integer(k as i64));
            }
        }
        Ok(Value::Nil)
    })?;
    globals.set("tIndexOf", tindexof)?;

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
                        // Recursively copy
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

    // SecureCmdOptionParse - parse secure command option strings
    globals.set(
        "SecureCmdOptionParse",
        lua.create_function(|lua, options: String| {
            // Returns the result of parsing a secure option string like "[mod:shift] action1; action2"
            // In simulation, just return the default (last) option
            if let Some(last) = options.split(';').last() {
                Ok(Value::String(lua.create_string(last.trim())?))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;

    // issecure() - check if current execution is in secure context
    globals.set("issecure", lua.create_function(|_, ()| Ok(false))?)?;

    // issecurevariable(table, variable) - check if variable is secure
    globals.set(
        "issecurevariable",
        lua.create_function(|_, (_table, _var): (Option<Value>, String)| {
            // Returns: isSecure, taint
            Ok((true, Value::Nil))
        })?,
    )?;

    // securecall(func, ...) - call a function in secure context
    globals.set(
        "securecall",
        lua.create_function(|_, (func, args): (mlua::Function, mlua::MultiValue)| {
            func.call::<mlua::MultiValue>(args)
        })?,
    )?;

    // forceinsecure() - mark current execution as insecure
    globals.set("forceinsecure", lua.create_function(|_, ()| Ok(()))?)?;

    // hooksecurefunc(name, hook) or hooksecurefunc(table, name, hook)
    let hooksecurefunc = lua.create_function(|lua, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();

        let (table, name, hook) = if args.len() == 2 {
            // hooksecurefunc("FuncName", hookFunc)
            let name = if let Value::String(s) = &args[0] {
                s.to_string_lossy().to_string()
            } else {
                String::new()
            };
            let hook = args[1].clone();
            (lua.globals(), name, hook)
        } else if args.len() >= 3 {
            // hooksecurefunc(someTable, "FuncName", hookFunc)
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
            let hook = args[2].clone();
            (table, name, hook)
        } else {
            return Ok(());
        };

        // Get the original function
        let original: Value = table.get::<Value>(name.as_str())?;

        if let (Value::Function(orig_fn), Value::Function(hook_fn)) = (original, hook) {
            // Create a wrapper that calls original then hook
            let wrapper = lua.create_function(move |_, args: mlua::MultiValue| {
                // Call original
                let result = orig_fn.call::<mlua::MultiValue>(args.clone())?;
                // Call hook (ignoring its result)
                let _ = hook_fn.call::<mlua::MultiValue>(args);
                Ok(result)
            })?;

            table.set(name.as_str(), wrapper)?;
        }

        Ok(())
    })?;
    globals.set("hooksecurefunc", hooksecurefunc)?;

    // nop() - no-operation function
    let nop = lua.create_function(|_, _: mlua::MultiValue| Ok(()))?;
    globals.set("nop", nop)?;

    // securecallfunction(func, ...) - calls a function in protected mode
    let securecallfunction =
        lua.create_function(|_, (func, args): (mlua::Function, mlua::MultiValue)| {
            // In WoW this provides taint protection, but for simulation we just call it
            func.call::<mlua::MultiValue>(args)
        })?;
    globals.set("securecallfunction", securecallfunction)?;

    // secureexecuterange(tbl, func, ...) - calls func(key, value, ...) for each entry in tbl
    let secureexecuterange = lua.create_function(
        |_lua, (tbl, func, args): (mlua::Table, mlua::Function, mlua::MultiValue)| {
            // Iterate through the table and call func(key, value, ...) for each entry
            for pair in tbl.pairs::<mlua::Value, mlua::Value>() {
                if let Ok((key, value)) = pair {
                    let mut call_args = mlua::MultiValue::new();
                    call_args.push_front(value);
                    call_args.push_front(key);
                    // Append the extra arguments
                    for arg in args.iter() {
                        call_args.push_back(arg.clone());
                    }
                    if let Err(e) = func.call::<()>(call_args) {
                        // Log but don't propagate errors (WoW behavior)
                        tracing::warn!("secureexecuterange callback error: {}", e);
                    }
                }
            }
            Ok(())
        },
    )?;
    globals.set("secureexecuterange", secureexecuterange)?;

    // geterrorhandler() - returns error handler function
    let geterrorhandler = lua.create_function(|lua, ()| {
        // Return a simple error handler that just prints
        let handler = lua.create_function(|_, msg: String| {
            println!("Lua error: {}", msg);
            Ok(())
        })?;
        Ok(handler)
    })?;
    globals.set("geterrorhandler", geterrorhandler)?;

    // seterrorhandler(func) - sets the error handler function
    let seterrorhandler = lua.create_function(|_, _handler: mlua::Function| {
        // Accept the handler but don't actually use it (stub)
        Ok(())
    })?;
    globals.set("seterrorhandler", seterrorhandler)?;

    // SecureHandler functions (secure frame management stubs)
    let secure_handler_set_frame_ref = lua.create_function(
        |_, (_frame, _name, _target): (mlua::Value, String, mlua::Value)| Ok(()),
    )?;
    globals.set("SecureHandlerSetFrameRef", secure_handler_set_frame_ref)?;

    let secure_handler_execute = lua.create_function(
        |_, (_frame, _body, _args): (mlua::Value, String, mlua::MultiValue)| Ok(()),
    )?;
    globals.set("SecureHandlerExecute", secure_handler_execute)?;

    let secure_handler_wrap_script =
        lua.create_function(|_, (_frame, _script, _body): (mlua::Value, String, String)| Ok(()))?;
    globals.set("SecureHandlerWrapScript", secure_handler_wrap_script)?;

    // Secure state driver functions
    globals.set(
        "RegisterStateDriver",
        lua.create_function(|_, (_frame, _attribute, _state_driver): (Value, String, String)| {
            // Secure state drivers are not fully implemented in simulation
            Ok(())
        })?,
    )?;
    globals.set(
        "UnregisterStateDriver",
        lua.create_function(|_, (_frame, _attribute): (Value, String)| Ok(()))?,
    )?;
    globals.set(
        "RegisterAttributeDriver",
        lua.create_function(|_, (_frame, _attribute, _driver): (Value, String, String)| Ok(()))?,
    )?;
    globals.set(
        "UnregisterAttributeDriver",
        lua.create_function(|_, (_frame, _attribute): (Value, String)| Ok(()))?,
    )?;

    // GetCurrentEnvironment() - returns the current global environment table
    let get_current_environment = lua.create_function(|lua, ()| {
        // Return _G (the global environment table)
        Ok(lua.globals())
    })?;
    globals.set("GetCurrentEnvironment", get_current_environment)?;

    // Lua stdlib global aliases (WoW compatibility)
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
        -- WoW uses bit.* functions for bitwise operations
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

    // Mixin system (WoW C++ intrinsics)
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
