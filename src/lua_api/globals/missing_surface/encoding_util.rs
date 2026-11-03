use super::ensure_namespace;
use crate::lua_api::methods::{
    create_string, create_table, table_set, table_set_num, val_to_string,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::closure::RustFn;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub(super) fn register_encoding_util_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_EncodingUtil")?;
    let methods: &[(&'static str, RustFn)] = &[
        ("EncodeHex", encoding_util_passthrough),
        ("DecodeHex", encoding_util_passthrough),
        ("SerializeJSON", encoding_util_serialize_json),
        ("DeserializeJSON", encoding_util_deserialize_json),
    ];
    for &(name, func) in methods {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    Ok(())
}

fn encoding_util_passthrough(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let Some(text) = val_to_string(state, value) else {
        return Ok(0);
    };
    let text = create_string(state, &text);
    state.push(text);
    Ok(1)
}

fn encoding_util_serialize_json(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let Some(json) = lua_value_to_json(state, value) else {
        return Ok(0);
    };
    let text = create_string(state, &json.to_string());
    state.push(text);
    Ok(1)
}

fn encoding_util_deserialize_json(state: &mut LuaState) -> LuaResult<u32> {
    let Some(source) = val_to_string(state, stack_val(state, 1)) else {
        return Ok(0);
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&source) else {
        return Ok(0);
    };
    let value = json_to_lua_value(state, json);
    state.push(value);
    Ok(1)
}

fn lua_value_to_json(state: &LuaState, value: Val) -> Option<serde_json::Value> {
    match value {
        Val::Nil => Some(serde_json::Value::Null),
        Val::Bool(flag) => Some(serde_json::Value::Bool(flag)),
        Val::Num(number) => serde_json::Number::from_f64(number).map(serde_json::Value::Number),
        Val::Str(_) => val_to_string(state, value).map(serde_json::Value::String),
        Val::Table(table_ref) => lua_table_to_json(state, table_ref),
        _ => val_to_string(state, value).map(serde_json::Value::String),
    }
}

fn lua_table_to_json(state: &LuaState, table_ref: GcRef<Table>) -> Option<serde_json::Value> {
    let table = state.gc.tables.get(table_ref)?;
    let mut values = serde_json::Map::new();

    for (index, value) in table.array_slice().iter().enumerate() {
        let json = lua_value_to_json(state, *value)?;
        values.insert((index + 1).to_string(), json);
    }

    for (key, value) in table.hash_entries() {
        let key = json_key_from_lua(state, key)?;
        let json = lua_value_to_json(state, value)?;
        values.insert(key, json);
    }

    Some(serde_json::Value::Object(values))
}

fn json_key_from_lua(state: &LuaState, key: Val) -> Option<String> {
    match key {
        Val::Num(number) if number.fract() == 0.0 => Some((number as i64).to_string()),
        Val::Bool(flag) => Some(flag.to_string()),
        Val::Str(_) => val_to_string(state, key),
        _ => val_to_string(state, key),
    }
}

fn json_to_lua_value(state: &mut LuaState, json: serde_json::Value) -> Val {
    match json {
        serde_json::Value::Null => Val::Nil,
        serde_json::Value::Bool(flag) => Val::Bool(flag),
        serde_json::Value::Number(number) => Val::Num(number.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(text) => create_string(state, &text),
        serde_json::Value::Array(values) => json_array_to_lua_value(state, values),
        serde_json::Value::Object(values) => json_object_to_lua_value(state, values),
    }
}

fn json_array_to_lua_value(state: &mut LuaState, values: Vec<serde_json::Value>) -> Val {
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    for (index, value) in values.into_iter().enumerate() {
        let value = json_to_lua_value(state, value);
        table_set_num(state, table_ref, (index + 1) as f64, value);
    }
    Val::Table(table_ref)
}

fn json_object_to_lua_value(
    state: &mut LuaState,
    values: serde_json::Map<String, serde_json::Value>,
) -> Val {
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    for (key, value) in values {
        let value = json_to_lua_value(state, value);
        if let Ok(index) = key.parse::<i64>() {
            if index > 0 {
                table_set_num(state, table_ref, index as f64, value);
                continue;
            }
        }
        table_set(state, Val::Table(table_ref), &key, value);
    }
    Val::Table(table_ref)
}
