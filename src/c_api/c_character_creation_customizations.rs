//! Host-seeded R79 publisher. The unseeded collector remains unmodeled.
use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{
    borrow_state, create_string_bytes, create_table, table_set, table_set_static,
};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::{LuaApiMut, LuaResult, Val, vm::state::LuaState};

/// Already-published host values. Their native conversions/shapes are UNKNOWN.
#[derive(Clone, Debug)]
pub enum PublishedValue {
    Nil,
    Number(f64),
    Text(Vec<u8>),
    Boolean(bool),
    Object(Vec<(String, PublishedValue)>),
    Array(Vec<PublishedValue>),
}
#[derive(Clone, Debug, Default)]
pub struct Icon {
    pub id: u32,
    pub resolved: Option<Vec<u8>>,
}
#[derive(Clone, Debug, Default)]
pub struct Category {
    /// Host-supplied stored finalizer key, not the published sequential position.
    pub stored_order_key: i64,
    pub fields: Vec<(String, PublishedValue)>,
    pub icon: Icon,
    pub selected_icon: Icon,
    pub camera_zoom_level: Option<i64>,
    pub camera_distance_offset: Option<f64>,
    pub options: Vec<OptionRecord>,
}
#[derive(Clone, Debug, Default)]
pub struct OptionRecord {
    /// Host-supplied stored finalizer key, not the published sequential position.
    pub stored_order_key: i64,
    pub fields: Vec<(String, PublishedValue)>,
    pub choices: Vec<Choice>,
}
#[derive(Clone, Debug, Default)]
pub struct Choice {
    pub fields: Vec<(String, PublishedValue)>,
}

pub(crate) fn install(lua: &mut rilua::Lua) -> crate::Result<()> {
    let state = lua.state_mut();
    table_set_rust_fn_static(state, state.global, "__wow_host_customizations", publish)?;
    lua.exec(
        r#"
        local publish = __wow_host_customizations
        __wow_host_customizations = nil
        local unseeded = C_CharacterCreation.GetAvailableCustomizations
        C_CharacterCreation.GetAvailableCustomizations = function()
            local result = publish()
            if result ~= nil then return result end
            return unseeded()
        end
    "#,
    )?;
    Ok(())
}

// VERIFIED publication order (#275). Lua hash iteration order is not asserted.
const CATEGORY_FIELDS: &[&str] = &[
    "id",
    "orderIndex",
    "name",
    "icon",
    "selectedIcon",
    "undressModel",
    "subcategory",
    "cameraZoomLevel",
    "cameraDistanceOffset",
    "spellShapeshiftFormID",
    "chrModelID",
    "options",
    "hasNewChoices",
    "needsNativeFormCategory",
];
const OPTION_FIELDS: &[&str] = &[
    "id",
    "name",
    "orderIndex",
    "optionType",
    "choices",
    "currentChoiceIndex",
    "hasNewChoices",
    "isSound",
];
const CHOICE_FIELDS: &[&str] = &[
    "id",
    "name",
    "ineligibleChoice",
    "isNew",
    "swatchColor1",
    "swatchColor2",
    "soundKit",
    "isLocked",
    "lockedText",
];

fn publish(state: &mut LuaState) -> LuaResult<u32> {
    let records = borrow_state(state)?.character_customizations.clone();
    if let Some(records) = records {
        push_sequence(state, &records, push_category);
    } else {
        // None means no collector decision: preserve the captured temporary provider.
        state.push(Val::Nil);
    }
    Ok(1)
}

// All builders leave exactly one rooted value on the stack. Parents stay rooted
// while children and string keys allocate; children are popped only after attach.
fn push_table(state: &mut LuaState) -> Val {
    let table = create_table(state);
    state.push(table);
    table
}

fn push_sequence<T>(state: &mut LuaState, rows: &[T], push: fn(&mut LuaState, &T) -> Val) -> Val {
    let table = push_table(state);
    for (index, row) in rows.iter().enumerate() {
        let child = push(state, row);
        set_table_array(state, table, (index + 1) as i64, child);
        state.pop();
    }
    table
}

fn push_value(state: &mut LuaState, input: &PublishedValue) -> Val {
    let value = match input {
        PublishedValue::Nil => Val::Nil,
        PublishedValue::Number(number) => Val::Num(*number),
        PublishedValue::Boolean(value) => Val::Bool(*value),
        PublishedValue::Text(bytes) => create_string_bytes(state, bytes),
        PublishedValue::Array(values) => return push_sequence(state, values, push_value),
        PublishedValue::Object(fields) => {
            let table = push_table(state);
            for (key, value) in fields {
                let value = push_value(state, value);
                table_set(state, table, key, value);
                state.pop();
            }
            return table;
        }
    };
    state.push(value);
    value
}

fn push_field(state: &mut LuaState, fields: &[(String, PublishedValue)], key: &str) -> Val {
    let value = fields
        .iter()
        .find(|(field, _)| field == key)
        .map(|(_, value)| value);
    push_value(state, value.unwrap_or(&PublishedValue::Nil))
}

fn push_icon(state: &mut LuaState, icon: &Icon) -> Val {
    let value = if icon.id == 0 {
        Val::Nil
    } else {
        create_string_bytes(state, icon.resolved.as_deref().unwrap_or(b""))
    };
    state.push(value);
    value
}

fn push_category(state: &mut LuaState, record: &Category) -> Val {
    let table = push_table(state);
    for &key in CATEGORY_FIELDS {
        let value = match key {
            "icon" => push_icon(state, &record.icon),
            "selectedIcon" => push_icon(state, &record.selected_icon),
            "cameraZoomLevel" => {
                let value = record
                    .camera_zoom_level
                    .map_or(Val::Nil, |n| Val::Num(n as f64));
                state.push(value);
                value
            }
            "cameraDistanceOffset" => {
                let value = record.camera_distance_offset.map_or(Val::Nil, Val::Num);
                state.push(value);
                value
            }
            "options" => push_sequence(state, &record.options, push_option),
            _ => push_field(state, &record.fields, key),
        };
        table_set_static(state, table, key, value);
        state.pop();
    }
    table
}

fn push_option(state: &mut LuaState, record: &OptionRecord) -> Val {
    let table = push_table(state);
    for &key in OPTION_FIELDS {
        let value = if key == "choices" {
            push_sequence(state, &record.choices, push_choice)
        } else {
            push_field(state, &record.fields, key)
        };
        table_set_static(state, table, key, value);
        state.pop();
    }
    table
}

fn push_choice(state: &mut LuaState, record: &Choice) -> Val {
    let table = push_table(state);
    for &key in CHOICE_FIELDS {
        let value = push_field(state, &record.fields, key);
        table_set_static(state, table, key, value);
        state.pop();
    }
    table
}
