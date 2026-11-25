//! INFERRED Retail/Forever C_EditMode cache and account-layout model.
//!
//! The saved-cache decode and default-setting values preserve the former Lua
//! shim. Generated Edit Mode documentation proves the public table shapes, but
//! it does not prove the native cache decoder or default values.

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get_static, val_to_string,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// VERIFIED: Retail and Forever generated docs declare Account=1 in EditModeLayoutType.
const ACCOUNT_LAYOUT_TYPE: f64 = 1.0;

#[derive(Clone, Debug, PartialEq)]
enum EditModeKey {
    Number(f64),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
enum EditModeValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Table(Vec<(EditModeKey, EditModeValue)>),
}

impl Default for EditModeValue {
    fn default() -> Self {
        Self::Nil
    }
}

#[derive(Clone, Default)]
pub(crate) struct EditModeCacheState {
    layouts: Vec<EditModeValue>,
    active_layout: EditModeValue,
    account_settings: Option<Vec<EditModeValue>>,
}

impl EditModeCacheState {
    fn initial_active_layout(&self) -> EditModeValue {
        if self.active_layout == EditModeValue::Nil {
            EditModeValue::Number(1.0)
        } else {
            self.active_layout.clone()
        }
    }

    fn account_settings(&mut self, defaults: Vec<EditModeValue>) -> Vec<EditModeValue> {
        self.account_settings.get_or_insert(defaults).clone()
    }

    fn merge_account_settings(
        defaults: Vec<EditModeValue>,
        supplied: &[EditModeValue],
    ) -> Vec<EditModeValue> {
        let mut merged = defaults;
        for setting in supplied {
            replace_or_append_setting(&mut merged, setting.clone());
        }
        sort_account_settings(&mut merged);
        merged
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_EditMode")?;
    register_cache_methods(state, namespace)?;
    Ok(())
}

fn register_cache_methods(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetAccountSettings", get_account_settings)?;
    table_set_rust_fn_static(state, namespace, "GetLayouts", get_layouts)?;
    table_set_rust_fn_static(state, namespace, "SaveLayouts", save_layouts)?;
    table_set_rust_fn_static(state, namespace, "SetActiveLayout", set_active_layout)?;
    table_set_rust_fn_static(state, namespace, "SetAccountSetting", set_account_setting)?;
    table_set_rust_fn_static(state, namespace, "__LoadCache", load_cache)
}

fn get_account_settings(state: &mut LuaState) -> LuaResult<u32> {
    let defaults = build_default_account_settings(state);
    let settings = {
        let mut sim = borrow_state_mut(state)?;
        sim.edit_mode_cache.account_settings(defaults)
    };
    let value = value_to_lua(state, &EditModeValue::array(settings))?;
    state.push(value);
    Ok(1)
}

fn get_layouts(state: &mut LuaState) -> LuaResult<u32> {
    let model = borrow_state(state)?.edit_mode_cache.clone();
    let active_layout = model.initial_active_layout();
    let layouts = EditModeValue::array(model.layouts);
    let result = EditModeValue::object([("layouts", layouts), ("activeLayout", active_layout)]);
    let value = value_to_lua(state, &result)?;
    state.push(value);
    Ok(1)
}

fn save_layouts(state: &mut LuaState) -> LuaResult<u32> {
    let input = stack_val(state, 1);
    if !matches!(input, Val::Table(_)) {
        return Ok(0);
    }
    let input = copy_lua_value(state, input)?;
    borrow_state_mut(state)?.edit_mode_cache.save_layouts(input);
    Ok(0)
}

fn set_active_layout(state: &mut LuaState) -> LuaResult<u32> {
    if let Val::Num(index) = stack_val(state, 1) {
        borrow_state_mut(state)?.edit_mode_cache.active_layout = EditModeValue::Number(index);
    }
    Ok(0)
}

fn set_account_setting(state: &mut LuaState) -> LuaResult<u32> {
    let setting = stack_val(state, 1);
    let value = stack_val(state, 2);
    let incoming = setting_record_value(state, setting, value)?;
    let defaults = build_default_account_settings(state);
    let mut sim = borrow_state_mut(state)?;
    let model = &mut sim.edit_mode_cache;
    let mut settings = model.account_settings(defaults);
    replace_or_append_setting(&mut settings, incoming);
    sort_account_settings(&mut settings);
    model.account_settings = Some(settings);
    Ok(0)
}

fn load_cache(state: &mut LuaState) -> LuaResult<u32> {
    let account = string_argument(state, 1);
    let character = string_argument(state, 2);
    let spec_index = numeric_argument(state, 3).unwrap_or(1.0);
    let preferred = string_argument(state, 4);
    let account = decode_account_cache(state, account.as_deref());
    let active = active_layout_from_character_cache(character.as_deref(), spec_index);
    let active = choose_override(state, &account.layouts, preferred.as_deref()).or(active);
    let defaults = (!account.settings.is_empty()).then(|| build_default_account_settings(state));
    let mut sim = borrow_state_mut(state)?;
    let model = &mut sim.edit_mode_cache;
    model.layouts = account.layouts;
    model.active_layout =
        active.map_or_else(|| model.initial_active_layout(), EditModeValue::Number);
    if let Some(defaults) = defaults {
        model.account_settings = Some(EditModeCacheState::merge_account_settings(
            defaults,
            &account.settings,
        ));
    }
    let missing_layouts = account.missing_layouts;
    drop(sim);
    state.push(Val::Bool(missing_layouts));
    Ok(1)
}

impl EditModeValue {
    fn array(values: Vec<Self>) -> Self {
        Self::Table(
            values
                .into_iter()
                .enumerate()
                .map(|(index, value)| (EditModeKey::Number((index + 1) as f64), value))
                .collect(),
        )
    }

    fn object<const N: usize>(fields: [(&str, Self); N]) -> Self {
        Self::Table(
            fields
                .into_iter()
                .map(|(key, value)| (EditModeKey::String(key.to_owned()), value))
                .collect(),
        )
    }

    fn field(&self, name: &str) -> Option<&Self> {
        let Self::Table(entries) = self else {
            return None;
        };
        entries.iter().find_map(|(key, value)| match key {
            EditModeKey::String(key) if key == name => Some(value),
            _ => None,
        })
    }

    fn number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }
}

fn setting_record(setting: f64, value: f64) -> EditModeValue {
    EditModeValue::object([
        ("setting", EditModeValue::Number(setting)),
        ("value", EditModeValue::Number(value)),
    ])
}

fn setting_record_value(
    state: &mut LuaState,
    setting: Val,
    value: Val,
) -> LuaResult<EditModeValue> {
    Ok(EditModeValue::object([
        ("setting", copy_lua_value(state, setting)?),
        ("value", copy_lua_value(state, value)?),
    ]))
}

fn replace_or_append_setting(settings: &mut Vec<EditModeValue>, incoming: EditModeValue) {
    let Some(setting) = incoming.field("setting") else {
        settings.push(incoming);
        return;
    };
    if let Some(existing) = settings
        .iter_mut()
        .find(|row| row.field("setting") == Some(setting))
    {
        *existing = incoming;
    } else {
        settings.push(incoming);
    }
}

fn sort_account_settings(settings: &mut [EditModeValue]) {
    settings.sort_by(|left, right| {
        left.field("setting")
            .and_then(EditModeValue::number)
            .unwrap_or(f64::NAN)
            .total_cmp(
                &right
                    .field("setting")
                    .and_then(EditModeValue::number)
                    .unwrap_or(f64::NAN),
            )
    });
}

fn build_default_account_settings(state: &mut LuaState) -> Vec<EditModeValue> {
    enum_numeric_values(state, "EditModeAccountSetting")
        .into_iter()
        .map(|setting| setting_record(setting, default_setting_value(state, setting)))
        .collect()
}

fn enum_numeric_values(state: &mut LuaState, name: &'static str) -> Vec<f64> {
    let enum_namespace = crate::c_api::global_val(state, "Enum");
    let enum_table = table_get_static(state, enum_namespace, name);
    let Val::Table(table_ref) = enum_table else {
        return Vec::new();
    };
    let Some(table) = state.gc.tables.get(table_ref) else {
        return Vec::new();
    };
    let mut values: Vec<f64> = table
        .hash_entries()
        .into_iter()
        .filter_map(|(_, value)| match value {
            Val::Num(number) => Some(number),
            _ => None,
        })
        .collect();
    values.sort_by(f64::total_cmp);
    values
}

fn enum_field_number(
    state: &mut LuaState,
    enum_name: &'static str,
    field_name: &'static str,
) -> Option<f64> {
    let enum_namespace = crate::c_api::global_val(state, "Enum");
    let enum_table = table_get_static(state, enum_namespace, enum_name);
    table_get_static(state, enum_table, field_name).as_number()
}

fn default_setting_value(state: &mut LuaState, setting: f64) -> f64 {
    if is_zero_default(state, setting) {
        return 0.0;
    }
    if enum_field_number(state, "EditModeAccountSetting", "GridSpacing") == Some(setting) {
        return edit_mode_grid_spacing(state).unwrap_or(100.0);
    }
    1.0
}

fn is_zero_default(state: &mut LuaState, setting: f64) -> bool {
    ["ShowGrid", "SettingsExpanded", "EnableAdvancedOptions"]
        .iter()
        .any(|name| enum_field_number(state, "EditModeAccountSetting", name) == Some(setting))
}

fn edit_mode_grid_spacing(state: &mut LuaState) -> Option<f64> {
    let constants = crate::c_api::global_val(state, "Constants");
    let edit_mode = table_get_static(state, constants, "EditModeConsts");
    table_get_static(state, edit_mode, "EditModeDefaultGridSpacing").as_number()
}

fn string_argument(state: &LuaState, index: i32) -> Option<String> {
    match stack_val(state, index) {
        value @ Val::Str(_) => val_to_string(state, value),
        _ => None,
    }
}

fn numeric_argument(state: &LuaState, index: i32) -> Option<f64> {
    match stack_val(state, index) {
        Val::Num(value) => Some(value),
        _ => None,
    }
}

fn copy_lua_value(state: &mut LuaState, value: Val) -> LuaResult<EditModeValue> {
    match value {
        Val::Nil => Ok(EditModeValue::Nil),
        Val::Bool(value) => Ok(EditModeValue::Boolean(value)),
        Val::Num(value) => Ok(EditModeValue::Number(value)),
        Val::Str(_) => Ok(EditModeValue::String(
            val_to_string(state, value).unwrap_or_default(),
        )),
        Val::Table(table_ref) => copy_lua_table(state, table_ref),
        _ => Err(rilua::runtime_error(
            "C_EditMode layout values must be tables, strings, numbers, booleans, or nil",
        )),
    }
}

fn copy_lua_table(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<EditModeValue> {
    use rilua::table_security::{TableAccess, check_table_access};
    check_table_access(state, table_ref, None, TableAccess::Index)?;
    let table = state
        .gc
        .tables
        .get(table_ref)
        .ok_or_else(|| rilua::runtime_error("table has been collected"))?;
    let array = table.array_slice().to_vec();
    let hash = table.hash_entries();
    let mut entries = Vec::new();
    for (index, value) in array.into_iter().enumerate() {
        if value != Val::Nil {
            entries.push((
                EditModeKey::Number((index + 1) as f64),
                copy_lua_value(state, value)?,
            ));
        }
    }
    for (key, value) in hash {
        let key = copy_lua_key(state, key)?;
        entries.push((key, copy_lua_value(state, value)?));
    }
    Ok(EditModeValue::Table(entries))
}

fn copy_lua_key(state: &LuaState, key: Val) -> LuaResult<EditModeKey> {
    match key {
        Val::Num(value) => Ok(EditModeKey::Number(value)),
        Val::Str(_) => val_to_string(state, key)
            .map(EditModeKey::String)
            .ok_or_else(|| rilua::runtime_error("C_EditMode layout key is not a string")),
        _ => Err(rilua::runtime_error(
            "C_EditMode layout keys must be strings or numbers",
        )),
    }
}

fn value_to_lua(state: &mut LuaState, value: &EditModeValue) -> LuaResult<Val> {
    match value {
        EditModeValue::Nil => Ok(Val::Nil),
        EditModeValue::Boolean(value) => Ok(Val::Bool(*value)),
        EditModeValue::Number(value) => Ok(Val::Num(*value)),
        EditModeValue::String(value) => Ok(create_string(state, value)),
        EditModeValue::Table(entries) => table_to_lua(state, entries),
    }
}

fn table_to_lua(state: &mut LuaState, entries: &[(EditModeKey, EditModeValue)]) -> LuaResult<Val> {
    let table = create_table(state);
    state.push(table);
    let Val::Table(table_ref) = table else {
        unreachable!()
    };
    let handle = rilua::Table::from_gc_ref(table_ref);
    for (key, value) in entries {
        let key = key_to_lua(state, key)?;
        let value = value_to_lua(state, value)?;
        handle.raw_set(state, key, value)?;
    }
    Ok(state.pop())
}

fn key_to_lua(state: &mut LuaState, key: &EditModeKey) -> LuaResult<Val> {
    match key {
        EditModeKey::Number(value) => Ok(Val::Num(*value)),
        EditModeKey::String(value) => Ok(create_string(state, value)),
    }
}

impl EditModeValue {
    fn from_table_field(value: &EditModeValue, field: &str) -> Option<EditModeValue> {
        value.field(field).cloned()
    }
}

impl EditModeCacheState {
    fn save_layouts(&mut self, save_info: EditModeValue) {
        let layouts = EditModeValue::from_table_field(&save_info, "layouts")
            .and_then(EditModeValue::table_values)
            .unwrap_or_default();
        let active = EditModeValue::from_table_field(&save_info, "activeLayout")
            .filter(|value| !matches!(value, EditModeValue::Nil | EditModeValue::Boolean(false)))
            .unwrap_or_else(|| self.initial_active_layout());
        self.layouts = layouts;
        self.active_layout = active;
    }
}

impl EditModeValue {
    fn table_values(self) -> Option<Vec<EditModeValue>> {
        let Self::Table(entries) = self else {
            return None;
        };
        let mut values: Vec<_> = entries
            .into_iter()
            .filter_map(|(key, value)| match key {
                EditModeKey::Number(index) if index >= 1.0 && index.fract() == 0.0 => {
                    Some((index, value))
                }
                _ => None,
            })
            .collect();
        values.sort_by(|left, right| left.0.total_cmp(&right.0));
        Some(values.into_iter().map(|(_, value)| value).collect())
    }
}

struct ParsedCache {
    layouts: Vec<EditModeValue>,
    settings: Vec<EditModeValue>,
    missing_layouts: bool,
}

fn decode_account_cache(state: &mut LuaState, text: Option<&str>) -> ParsedCache {
    let Some(text) = text else {
        return empty_cache();
    };
    let text = text.replace('\0', "");
    let mut reader = CacheReader::new(&text);
    let Some(layout_count) = reader.read_count() else {
        return empty_cache();
    };
    let Some(setting_count) = reader.read_count() else {
        return missing_cache(layout_count > 0);
    };
    let (settings, complete) = read_cached_account_settings(&mut reader, setting_count);
    if !complete {
        return ParsedCache {
            layouts: Vec::new(),
            settings,
            missing_layouts: layout_count > 0,
        };
    }
    let layouts = read_cached_layouts(state, &mut reader, layout_count);
    ParsedCache {
        missing_layouts: layouts.len() < layout_count,
        layouts,
        settings,
    }
}

fn read_cached_layouts(
    state: &mut LuaState,
    reader: &mut CacheReader<'_>,
    count: usize,
) -> Vec<EditModeValue> {
    let start = reader.cursor;
    let (mut layouts, metadata_end) = read_layouts(state, reader, count, true);
    if reader.has_token_at(metadata_end) {
        reader.cursor = start;
        let (legacy, legacy_end) = read_layouts(state, reader, count, false);
        if legacy_end >= metadata_end {
            layouts = legacy;
        }
    }
    layouts
}

fn empty_cache() -> ParsedCache {
    ParsedCache {
        layouts: Vec::new(),
        settings: Vec::new(),
        missing_layouts: false,
    }
}

fn missing_cache(missing_layouts: bool) -> ParsedCache {
    ParsedCache {
        layouts: Vec::new(),
        settings: Vec::new(),
        missing_layouts,
    }
}

fn read_cached_account_settings(
    reader: &mut CacheReader<'_>,
    count: usize,
) -> (Vec<EditModeValue>, bool) {
    let mut settings = Vec::new();
    for setting in 0..count {
        let Some(value) = reader.read_number() else {
            return (settings, false);
        };
        settings.push(setting_record(setting as f64, value));
    }
    (settings, true)
}

struct CacheReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> CacheReader<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            bytes: text.as_bytes(),
            cursor: 0,
        }
    }

    fn skip_space(&mut self) {
        while self
            .bytes
            .get(self.cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.cursor += 1;
        }
    }

    fn read_token(&mut self) -> Option<&'a str> {
        self.skip_space();
        let start = self.cursor;
        while self
            .bytes
            .get(self.cursor)
            .is_some_and(|byte| !byte.is_ascii_whitespace())
        {
            self.cursor += 1;
        }
        if start == self.cursor {
            return None;
        }
        std::str::from_utf8(&self.bytes[start..self.cursor]).ok()
    }

    fn read_number(&mut self) -> Option<f64> {
        self.read_token()?.parse().ok()
    }

    fn read_count(&mut self) -> Option<usize> {
        let value = self.read_number()?;
        (value >= 0.0 && value.fract() == 0.0 && value <= usize::MAX as f64)
            .then_some(value as usize)
    }

    fn read_counted_name(&mut self, byte_count: usize) -> Option<String> {
        let separator = *self.bytes.get(self.cursor)?;
        if !separator.is_ascii_whitespace() || byte_count == 0 {
            return None;
        }
        let start = self.cursor + 1;
        let end = start.checked_add(byte_count)?;
        if !self.bytes.get(end).is_some_and(u8::is_ascii_whitespace) {
            return None;
        }
        let name = std::str::from_utf8(self.bytes.get(start..end)?)
            .ok()?
            .to_owned();
        self.cursor = end;
        Some(name)
    }

    fn has_token_at(&self, position: usize) -> bool {
        let mut cursor = Self {
            bytes: self.bytes,
            cursor: position,
        };
        cursor.read_token().is_some()
    }
}

fn read_layouts(
    state: &mut LuaState,
    reader: &mut CacheReader<'_>,
    layout_count: usize,
    has_metadata: bool,
) -> (Vec<EditModeValue>, usize) {
    let mut layouts = Vec::new();
    for _ in 0..layout_count {
        let start = reader.cursor;
        let Some(layout) = read_layout(state, reader, has_metadata) else {
            return (layouts, start);
        };
        layouts.push(layout);
    }
    (layouts, reader.cursor)
}

fn read_layout(
    state: &mut LuaState,
    reader: &mut CacheReader<'_>,
    has_metadata: bool,
) -> Option<EditModeValue> {
    let (name, index) = read_layout_identity(reader, has_metadata)?;
    let count = reader.read_count()?;
    let systems = read_layout_systems(state, reader, count, has_metadata)?;
    Some(build_layout(name, index, systems))
}

fn read_layout_identity(
    reader: &mut CacheReader<'_>,
    has_metadata: bool,
) -> Option<(String, Option<f64>)> {
    let prefix = reader.read_number()?;
    if prefix < 0.0 || prefix.fract() != 0.0 {
        return None;
    }
    if has_metadata {
        let name = reader.read_counted_name(prefix as usize)?;
        reader.read_count()?;
        Some((name, None))
    } else {
        Some((reader.read_token()?.to_owned(), Some(prefix)))
    }
}

fn read_layout_systems(
    state: &mut LuaState,
    reader: &mut CacheReader<'_>,
    count: usize,
    has_metadata: bool,
) -> Option<Vec<EditModeValue>> {
    (0..count)
        .map(|_| read_system(state, reader, has_metadata))
        .collect()
}

struct CachedSystem {
    system: f64,
    index: Option<f64>,
    default_position: f64,
    point: String,
    relative_point: String,
    relative_to: String,
    offset_x: f64,
    offset_y: f64,
    encoded_settings: String,
}

fn read_system(
    state: &mut LuaState,
    reader: &mut CacheReader<'_>,
    has_metadata: bool,
) -> Option<EditModeValue> {
    let cached = read_cached_system(reader, has_metadata)?;
    let status_bar = enum_field_number(state, "EditModeSystem", "StatusTrackingBar");
    let hidden = status_bar == Some(cached.system) && cached.default_position == 0.0;
    Some(build_system_info(cached, hidden))
}

fn read_cached_system(reader: &mut CacheReader<'_>, has_metadata: bool) -> Option<CachedSystem> {
    let system = reader.read_number()?;
    let index = adjust_system_index(reader.read_number()?, has_metadata);
    let default_position = reader.read_number()?;
    let point = frame_point(reader.read_number()?);
    let relative_point = frame_point(reader.read_number()?);
    let relative_to = reader.read_token()?.to_owned();
    if relative_to.parse::<f64>().is_ok() {
        return None;
    }
    let offset_x = reader.read_number()?;
    let offset_y = reader.read_number()?;
    reader.read_token()?;
    let encoded_settings = reader.read_token()?.to_owned();
    Some(CachedSystem {
        system,
        index,
        default_position,
        point,
        relative_point,
        relative_to,
        offset_x,
        offset_y,
        encoded_settings,
    })
}

fn adjust_system_index(index: f64, has_metadata: bool) -> Option<f64> {
    if index == -1.0 && has_metadata {
        return None;
    }
    Some(if index >= 0.0 { index + 1.0 } else { index })
}

fn build_system_info(cached: CachedSystem, hidden: bool) -> EditModeValue {
    EditModeValue::object([
        ("system", EditModeValue::Number(cached.system)),
        (
            "systemIndex",
            cached
                .index
                .map_or(EditModeValue::Nil, EditModeValue::Number),
        ),
        ("hidden", EditModeValue::Boolean(hidden)),
        (
            "isInDefaultPosition",
            EditModeValue::Boolean(cached.default_position != 0.0),
        ),
        ("anchorInfo", build_anchor_info(&cached)),
        (
            "settings",
            EditModeValue::array(decode_setting_chunks(&cached.encoded_settings)),
        ),
    ])
}

fn build_anchor_info(cached: &CachedSystem) -> EditModeValue {
    EditModeValue::object([
        ("point", EditModeValue::String(cached.point.clone())),
        (
            "relativeTo",
            EditModeValue::String(cached.relative_to.clone()),
        ),
        (
            "relativePoint",
            EditModeValue::String(cached.relative_point.clone()),
        ),
        ("offsetX", EditModeValue::Number(cached.offset_x)),
        ("offsetY", EditModeValue::Number(cached.offset_y)),
    ])
}

fn build_layout(
    name: String,
    layout_index: Option<f64>,
    systems: Vec<EditModeValue>,
) -> EditModeValue {
    let mut fields = vec![
        (
            EditModeKey::String("layoutName".into()),
            EditModeValue::String(name),
        ),
        (
            EditModeKey::String("layoutType".into()),
            EditModeValue::Number(ACCOUNT_LAYOUT_TYPE),
        ),
        (
            EditModeKey::String("systems".into()),
            EditModeValue::array(systems),
        ),
    ];
    if let Some(index) = layout_index {
        fields.push((
            EditModeKey::String("layoutIndex".into()),
            EditModeValue::Number(index),
        ));
    }
    EditModeValue::Table(fields)
}

fn decode_setting_chunks(encoded: &str) -> Vec<EditModeValue> {
    let bytes = encoded.as_bytes();
    let mut settings = Vec::new();
    let mut last_setting = None;
    let mut place_value = 1.0;
    for pair in bytes.chunks_exact(2) {
        let setting = f64::from(pair[0]) - 35.0;
        let value_chunk = f64::from(pair[1]) - 35.0;
        if last_setting == Some(setting) {
            place_value *= 90.0;
            if let Some(EditModeValue::Table(fields)) = settings.last_mut() {
                add_setting_value(fields, value_chunk * place_value);
            }
        } else {
            settings.push(setting_record(setting, value_chunk));
            last_setting = Some(setting);
            place_value = 1.0;
        }
    }
    settings
}

fn add_setting_value(fields: &mut [(EditModeKey, EditModeValue)], increment: f64) {
    if let Some((_, EditModeValue::Number(value))) = fields
        .iter_mut()
        .find(|(key, _)| *key == EditModeKey::String("value".into()))
    {
        *value += increment;
    }
}

fn frame_point(value: f64) -> String {
    const POINTS: [&str; 9] = [
        "TOPLEFT",
        "TOP",
        "TOPRIGHT",
        "LEFT",
        "CENTER",
        "RIGHT",
        "BOTTOMLEFT",
        "BOTTOM",
        "BOTTOMRIGHT",
    ];
    if value.fract() == 0.0 && (0.0..POINTS.len() as f64).contains(&value) {
        POINTS[value as usize].to_owned()
    } else {
        "CENTER".to_owned()
    }
}

fn active_layout_from_character_cache(text: Option<&str>, spec_index: f64) -> Option<f64> {
    let text = text?.replace('\0', " ");
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let selected = if spec_index >= 1.0 && spec_index.fract() == 0.0 {
        tokens
            .get(spec_index as usize - 1)
            .and_then(|token| token.parse::<f64>().ok())
    } else {
        None
    };
    selected
        .filter(|active| *active > 0.0)
        .or_else(|| tokens.iter().find_map(|token| positive_number(token)))
}

fn positive_number(token: &str) -> Option<f64> {
    token.parse::<f64>().ok().filter(|value| *value > 0.0)
}

fn choose_override(
    state: &mut LuaState,
    layouts: &[EditModeValue],
    preferred: Option<&str>,
) -> Option<f64> {
    let preferred = preferred?;
    if preferred.is_empty() {
        return None;
    }
    let presets = enum_field_number(state, "EditModePresetLayoutsMeta", "NumValues")?;
    choose_numeric_override(layouts, preferred, presets)
        .or_else(|| choose_named_override(layouts, preferred, presets))
}

fn choose_numeric_override(
    layouts: &[EditModeValue],
    preferred: &str,
    presets: f64,
) -> Option<f64> {
    let number = preferred.parse::<f64>().ok()?;
    if number <= 0.0 {
        return None;
    }
    if number.fract() == 0.0 && number as usize <= layouts.len() {
        return Some(presets + number);
    }
    let saved_index = layouts
        .iter()
        .position(|row| row.field("layoutIndex").and_then(EditModeValue::number) == Some(number));
    Some(saved_index.map_or(number, |index| presets + index as f64 + 1.0))
}

fn choose_named_override(layouts: &[EditModeValue], preferred: &str, presets: f64) -> Option<f64> {
    layouts
        .iter()
        .position(|row| {
            row.field("layoutName")
                .and_then(value_string)
                .is_some_and(|name| name.eq_ignore_ascii_case(preferred))
        })
        .map(|index| presets + index as f64 + 1.0)
}

fn value_string(value: &EditModeValue) -> Option<&str> {
    match value {
        EditModeValue::String(value) => Some(value),
        _ => None,
    }
}
