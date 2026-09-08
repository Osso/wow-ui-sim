use super::Result;
use rilua::{Lua, LuaApi, LuaApiMut, StdLib, Val};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

const CAPTURE: &str = r#"
__documents = {}
APIDocumentation = { AddDocumentationTable = function(self, document)
    __documents[#__documents + 1] = document
end }
local symbolic
local function expression(a, operator, b)
    local function text(value)
        if type(value) == 'table' then return value.__symbol end
        return tostring(value)
    end
    return symbolic('(' .. text(a) .. ' ' .. operator .. ' ' .. text(b) .. ')')
end
symbolic = function(text)
    return setmetatable({__symbol=text}, {
        __add=function(a,b) return expression(a,'+',b) end,
        __sub=function(a,b) return expression(a,'-',b) end,
    })
end
local function symbols(root)
    return setmetatable({}, {__index=function(_, namespace)
        return setmetatable({}, {__index=function(_, member)
            return symbolic(root .. '.' .. namespace .. '.' .. member)
        end})
    end})
end
Enum = symbols('Enum')
Constants = symbols('Constants')
dofile = nil
loadfile = nil
loadstring = nil
load = nil
require = nil
"#;

pub(super) fn documents(bytes: &[u8], path: &str) -> Result<Vec<Value>> {
    // No simulator, package loader, filesystem, networking, or OS libraries.
    // These are trusted generated data files, not a sandbox for hostile Lua.
    let mut lua = Lua::new_with(StdLib::BASE)?;
    lua.exec(CAPTURE)?;
    lua.exec_bytes(bytes, path)
        .map_err(|error| format!("{path}: {error}"))?;
    let capture = lua.get_global_val("__documents");
    let value = convert(&lua, capture, 0)?;
    let documents = value
        .as_array()
        .ok_or("documentation capture is not an array")?;
    if documents.is_empty() {
        return Err(format!("{path}: no documentation table captured").into());
    }
    Ok(documents.clone())
}

fn convert(lua: &Lua, value: Val, depth: usize) -> Result<Value> {
    if depth > 64 {
        return Err("cyclic or excessively nested documentation table".into());
    }
    match value {
        Val::Nil => Ok(Value::Null),
        Val::Bool(value) => Ok(Value::Bool(value)),
        Val::Num(number) if number.fract() == 0.0 && number.abs() <= 9_007_199_254_740_991.0 => {
            Ok(Value::from(number as i64))
        }
        Val::Num(number) => serde_json::Number::from_f64(number)
            .map(Value::Number)
            .ok_or_else(|| "non-finite documentation number".into()),
        Val::Str(_) => Ok(Value::String(
            std::str::from_utf8(lua.val_as_bytes(value).ok_or("missing string")?)?.to_owned(),
        )),
        Val::Table(reference) => {
            let table = lua
                .state()
                .gc
                .tables
                .get(reference)
                .ok_or("collected documentation table")?;
            let mut pairs = Vec::new();
            let mut key = Val::Nil;
            while let Some((next, value)) = table.next(key, &lua.state().gc.string_arena)? {
                key = next;
                if lua.val_as_bytes(key) == Some(b"Documentation") {
                    continue;
                }
                pairs.push((key, value));
            }
            convert_table(lua, pairs, depth + 1)
        }
        _ => Err("unsupported non-data documentation value".into()),
    }
}

fn convert_table(lua: &Lua, pairs: Vec<(Val, Val)>, depth: usize) -> Result<Value> {
    let mut object = Map::new();
    let mut array = BTreeMap::new();
    for (key, value) in pairs {
        match key {
            Val::Str(_) => {
                let name = std::str::from_utf8(lua.val_as_bytes(key).ok_or("missing key")?)?;
                object.insert(name.to_owned(), convert(lua, value, depth)?);
            }
            Val::Num(n) if n >= 1.0 && n.fract() == 0.0 => {
                array.insert(n as usize, convert(lua, value, depth)?);
            }
            _ => return Err("unsupported documentation key".into()),
        }
    }
    if array.is_empty() && !object.is_empty() {
        if object.len() == 1 && object.contains_key("__symbol") {
            return Ok(object.remove("__symbol").unwrap());
        }
        return Ok(Value::Object(object));
    }
    if !object.is_empty() || array.keys().copied().ne(1..=array.len()) {
        return Err("mixed or non-contiguous documentation table".into());
    }
    Ok(Value::Array(array.into_values().collect()))
}
