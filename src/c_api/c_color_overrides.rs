//! Retail color-override state exposed through C_ColorOverrides.

use super::c_ui_color::{channels_from_packed_color, copy_mixin_members, find_color_mixin};
use super::helpers::{bad_argument, parse_numeric_string, set_global_val};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_table, registry_table_or_create, table_get,
    table_get_static, table_set, table_set_static,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::{can_access_secrets, is_secret_value, unwrap_secret};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};
use sim_data_profile::global_colors_retail::RETAIL_GLOBAL_COLORS;

const REGISTRY_KEY: &str = "__wow_color_overrides";
const PLAYER_COLOR_OVERRIDES_CVAR: &str = "playerColorOverrides";
const OVERRIDE_TYPE_COUNT: u8 = 8;
const MAX_OVERRIDE_TYPE: u8 = OVERRIDE_TYPE_COUNT - 1;
const MAX_ITEM_QUALITY: u8 = 8;
const ENUM_VALUE_MODULUS: i32 = 256;
const QUALITY_COLOR_TAGS: [&str; 9] = [
    "ITEM_POOR_COLOR",
    "ITEM_STANDARD_COLOR",
    "ITEM_GOOD_COLOR",
    "ITEM_SUPERIOR_COLOR",
    "ITEM_EPIC_COLOR",
    "ITEM_LEGENDARY_COLOR",
    "ITEM_ARTIFACT_COLOR",
    "ITEM_SCALING_STAT_COLOR",
    "ITEM_WOW_TOKEN_COLOR",
];
const MISSING_DEFAULT_COLOR: u32 = 0xFFFF_00FF;
const COLOR_CHANNEL_SCALE: f32 = 255.0;
const COLOR_CHANNEL_ROUNDING: f32 = 0.5;
const COLOR_CHANNEL_MAX: f32 = 1.0;
const CLEAR_FUNCTION: &str = "ClearColorOverrides";
const GET_COLOR_FUNCTION: &str = "GetColorForQuality";
const GET_INFO_FUNCTION: &str = "GetColorOverrideInfo";
const GET_DEFAULT_FUNCTION: &str = "GetDefaultColorForQuality";
const REMOVE_FUNCTION: &str = "RemoveColorOverride";
const SET_FUNCTION: &str = "SetColorOverride";
const GET_COLOR_USAGE: &str = "Usage: local color = C_ColorOverrides.GetColorForQuality(quality)";
const GET_INFO_USAGE: &str =
    "Usage: local overrideInfo = C_ColorOverrides.GetColorOverrideInfo(overrideType)";
const GET_DEFAULT_USAGE: &str =
    "Usage: local color = C_ColorOverrides.GetDefaultColorForQuality(quality)";
const REMOVE_USAGE: &str = "Usage: C_ColorOverrides.RemoveColorOverride(overrideType)";
const SET_USAGE: &str = "Usage: C_ColorOverrides.SetColorOverride(overrideType, color)";
type PersistedOverride = [u8; 5];

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }

    let state = lua.state_mut();
    // Replace the pre-secure snapshot with a fresh ordinary-only namespace.
    let namespace = state.gc.alloc_table(Table::new());
    set_global_val(state, "C_ColorOverrides", Val::Table(namespace));
    table_set_rust_fn_static(state, namespace, CLEAR_FUNCTION, clear_color_overrides)?;
    table_set_rust_fn_static(state, namespace, GET_COLOR_FUNCTION, get_color_for_quality)?;
    table_set_rust_fn_static(state, namespace, GET_INFO_FUNCTION, get_color_override_info)?;
    table_set_rust_fn_static(
        state,
        namespace,
        GET_DEFAULT_FUNCTION,
        get_default_color_for_quality,
    )?;
    table_set_rust_fn_static(state, namespace, REMOVE_FUNCTION, remove_color_override)?;
    table_set_rust_fn_static(state, namespace, SET_FUNCTION, set_color_override)
}

/// Save the current records to playerColorOverrides; lifecycle callers choose when.
pub fn save_color_overrides(state: &mut LuaState) -> LuaResult<()> {
    let records = collect_persisted_overrides(state);
    let serialized = encode_persisted_overrides(&records)?;
    let accepted = borrow_state_mut(state)?
        .cvars
        .set(PLAYER_COLOR_OVERRIDES_CVAR, &serialized);
    if !accepted {
        return Err(runtime_error(
            "playerColorOverrides setting rejected the save",
        ));
    }
    Ok(())
}

/// Load playerColorOverrides and replace active records; lifecycle stays external.
pub fn load_color_overrides(state: &mut LuaState) -> LuaResult<()> {
    let serialized = borrow_state(state)?
        .cvars
        .get(PLAYER_COLOR_OVERRIDES_CVAR)
        .unwrap_or_default();
    clear_override_records(state);
    if serialized.is_empty() {
        return Ok(());
    }
    let records = decode_persisted_overrides(&serialized)?;
    for record in records {
        restore_persisted_override(state, record)?;
    }
    Ok(())
}

fn clear_override_records(state: &mut LuaState) {
    let overrides = registry_table_or_create(state, REGISTRY_KEY);
    for override_type in 0..OVERRIDE_TYPE_COUNT {
        table_set(state, overrides, &override_type.to_string(), Val::Nil);
    }
}

fn clear_color_overrides(state: &mut LuaState) -> LuaResult<u32> {
    clear_override_records(state);
    // INFERRED: the documented reset event names the native reset callback.
    dispatch_event_now(state, "COLOR_OVERRIDES_RESET", &[])?;
    // INFERRED: R88v's cleanup path supports zero Lua return values.
    Ok(0)
}

fn get_default_color_for_quality(state: &mut LuaState) -> LuaResult<u32> {
    let quality = read_enum_argument(
        state,
        1,
        GET_DEFAULT_FUNCTION,
        GET_DEFAULT_USAGE,
        MAX_ITEM_QUALITY,
    )?;
    let packed_color = default_packed_color(quality).unwrap_or(MISSING_DEFAULT_COLOR);
    push_color_result(state, packed_color, true)
}

fn get_color_for_quality(state: &mut LuaState) -> LuaResult<u32> {
    let quality = read_enum_argument(
        state,
        1,
        GET_COLOR_FUNCTION,
        GET_COLOR_USAGE,
        MAX_ITEM_QUALITY,
    )?;
    let packed_color = color_for_quality(state, quality);
    push_color_result(state, packed_color, true)
}

fn get_color_override_info(state: &mut LuaState) -> LuaResult<u32> {
    let override_type = read_enum_argument(
        state,
        1,
        GET_INFO_FUNCTION,
        GET_INFO_USAGE,
        MAX_OVERRIDE_TYPE,
    )?;
    let record = read_override_record(state, override_type);
    let Some(packed_color) = record_packed_color(state, record) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let info = build_color_override_info(state, record, override_type, packed_color)?;
    state.push(info);
    Ok(1)
}

fn build_color_override_info(
    state: &mut LuaState,
    record: Val,
    override_type: u8,
    packed_color: u32,
) -> LuaResult<Val> {
    let stack_top = state.top;
    let override_color = build_color_table(state, packed_color, false)?;
    state.push(override_color);
    let info = create_table(state);
    state.push(info);
    table_set_static(
        state,
        info,
        "overrideType",
        Val::Num(f64::from(override_type)),
    );
    table_set_static(state, info, "overrideColor", override_color);
    let color_string = table_get_static(state, record, "colorString");
    table_set_static(state, info, "overrideColorString", color_string);
    state.top = stack_top;
    Ok(info)
}

fn set_color_override(state: &mut LuaState) -> LuaResult<u32> {
    let override_type = read_enum_argument(state, 1, SET_FUNCTION, SET_USAGE, MAX_OVERRIDE_TYPE)?;
    let packed_color = read_color_argument(state)?;
    store_override_record(state, override_type, packed_color);
    publish_override_updated(state, override_type)?;
    // INFERRED: native mutation success returns zero Lua values.
    Ok(0)
}

fn remove_color_override(state: &mut LuaState) -> LuaResult<u32> {
    let override_type =
        read_enum_argument(state, 1, REMOVE_FUNCTION, REMOVE_USAGE, MAX_OVERRIDE_TYPE)?;
    remove_override_record(state, override_type);
    publish_override_updated(state, override_type)?;
    // INFERRED: native mutation success returns zero Lua values.
    Ok(0)
}

fn read_enum_argument(
    state: &LuaState,
    index: i32,
    function: &str,
    usage: &str,
    maximum: u8,
) -> LuaResult<u8> {
    let value = read_required_value(state, index, function, usage)?;
    let number = numeric_value(state, value).unwrap_or(0.0);
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(integer_overflow_error(index, function, number));
    }

    let converted = (number.trunc() as i32).rem_euclid(ENUM_VALUE_MODULUS) as u8;
    if converted > maximum {
        return Err(bad_argument(index, function, usage));
    }
    Ok(converted)
}

fn read_required_value(
    state: &LuaState,
    index: i32,
    function: &str,
    usage: &str,
) -> LuaResult<Val> {
    let original = stack_val(state, index);
    if matches!(original, Val::Nil) {
        return Err(bad_argument(index, function, usage));
    }
    read_allowed_value(state, original, index, function, usage)
}

fn read_allowed_value(
    state: &LuaState,
    original: Val,
    index: i32,
    function: &str,
    usage: &str,
) -> LuaResult<Val> {
    if is_secret_value(state, original) && !can_access_secrets(state) {
        let detail = format!(
            "{usage}. Secret values are only allowed during untainted execution for this argument."
        );
        return Err(bad_argument(index, function, &detail));
    }
    unwrap_secret(state, original).map_err(|_| {
        let detail = format!("{usage}. Unknown secret access mode.");
        bad_argument(index, function, &detail)
    })
}

fn numeric_value(state: &LuaState, value: Val) -> Option<f64> {
    match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|text| parse_numeric_string(text.data())),
        _ => None,
    }
}

fn integer_overflow_error(index: i32, function: &str, number: f64) -> rilua::LuaError {
    let value = if number.is_nan() {
        "nan".to_owned()
    } else if number == f64::INFINITY {
        "inf".to_owned()
    } else if number == f64::NEG_INFINITY {
        "-inf".to_owned()
    } else {
        format!("{number:.6}")
    };
    bad_argument(
        index,
        function,
        &format!("integer overflow attempting to store {value}"),
    )
}

fn read_color_argument(state: &mut LuaState) -> LuaResult<u32> {
    let color = read_required_value(state, 2, SET_FUNCTION, SET_USAGE)?;
    let Val::Table(color_table) = color else {
        return Err(bad_argument(2, SET_FUNCTION, SET_USAGE));
    };

    let alpha = read_color_component(state, color_table, "a")?;
    let red = read_color_component(state, color_table, "r")?;
    let green = read_color_component(state, color_table, "g")?;
    let blue = read_color_component(state, color_table, "b")?;
    Ok(pack_color(red, green, blue, alpha))
}

fn read_color_component(
    state: &mut LuaState,
    color_table: GcRef<Table>,
    field: &'static str,
) -> LuaResult<f32> {
    let value = table_get_static(state, Val::Table(color_table), field);
    let value = read_allowed_value(state, value, 2, SET_FUNCTION, SET_USAGE)?;
    let number = numeric_value(state, value);
    let valid = number.filter(|number| {
        number.is_finite() && *number >= f64::from(f32::MIN) && *number <= f64::from(f32::MAX)
    });
    let Some(number) = valid else {
        return Err(bad_argument(2, SET_FUNCTION, SET_USAGE));
    };
    Ok((number as f32).clamp(0.0, COLOR_CHANNEL_MAX))
}

// INFERRED: pack the channels as ARGB to match the GlobalColor representation.
fn pack_color(red: f32, green: f32, blue: f32, alpha: f32) -> u32 {
    let red = rounded_channel(red);
    let green = rounded_channel(green);
    let blue = rounded_channel(blue);
    let alpha = rounded_channel(alpha);
    pack_color_bytes(red, green, blue, alpha)
}

fn pack_color_bytes(red: u8, green: u8, blue: u8, alpha: u8) -> u32 {
    (u32::from(alpha) << 24) | (u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue)
}

fn rounded_channel(component: f32) -> u8 {
    (component * COLOR_CHANNEL_SCALE + COLOR_CHANNEL_ROUNDING).trunc() as u8
}

fn color_for_quality(state: &mut LuaState, quality: u8) -> u32 {
    // Retail ColorConstants maps qualities 0–7 to equal-valued override types.
    let override_type = (quality < OVERRIDE_TYPE_COUNT).then_some(quality);
    let override_color = override_type.and_then(|override_type| {
        let record = read_override_record(state, override_type);
        record_packed_color(state, record)
    });
    override_color
        .or_else(|| default_packed_color(quality))
        .unwrap_or(MISSING_DEFAULT_COLOR)
}

fn default_packed_color(quality: u8) -> Option<u32> {
    // INFERRED: these Retail 12.1.5.70783 generated rows supply the 70688 lookup.
    let base_tag = QUALITY_COLOR_TAGS.get(usize::from(quality))?;
    RETAIL_GLOBAL_COLORS
        .iter()
        .find(|row| row.base_tag == *base_tag)
        .map(|row| row.packed_color)
}

fn read_override_record(state: &mut LuaState, override_type: u8) -> Val {
    let overrides = registry_table_or_create(state, REGISTRY_KEY);
    table_get(state, overrides, &override_type.to_string())
}

fn record_packed_color(state: &mut LuaState, record: Val) -> Option<u32> {
    let Val::Num(packed_color) = table_get_static(state, record, "packedColor") else {
        return None;
    };
    (packed_color >= 0.0 && packed_color <= f64::from(u32::MAX)).then_some(packed_color as u32)
}

fn collect_persisted_overrides(state: &mut LuaState) -> Vec<PersistedOverride> {
    (0..OVERRIDE_TYPE_COUNT)
        .filter_map(|override_type| {
            let record = read_override_record(state, override_type);
            let [alpha, red, green, blue] = record_packed_color(state, record)?.to_be_bytes();
            Some([override_type, red, green, blue, alpha])
        })
        .collect()
}

fn encode_persisted_overrides(records: &[PersistedOverride]) -> LuaResult<String> {
    if records.is_empty() {
        return Ok(String::new());
    }
    // INFERRED: JSON rows encode [overrideType, red, green, blue, alpha] bytes.
    serde_json::to_string(records)
        .map_err(|error| runtime_error(format!("serialize playerColorOverrides: {error}")))
}

fn decode_persisted_overrides(setting: &str) -> LuaResult<Vec<PersistedOverride>> {
    serde_json::from_str(setting)
        .map_err(|error| runtime_error(format!("parse playerColorOverrides: {error}")))
}

fn restore_persisted_override(
    state: &mut LuaState,
    [override_type, red, green, blue, alpha]: PersistedOverride,
) -> LuaResult<()> {
    if override_type > MAX_OVERRIDE_TYPE {
        return Err(runtime_error(
            "playerColorOverrides contains an invalid overrideType",
        ));
    }
    let packed_color = pack_color_bytes(red, green, blue, alpha);
    store_override_record(state, override_type, packed_color);
    Ok(())
}

fn store_override_record(state: &mut LuaState, override_type: u8, packed_color: u32) {
    let overrides = registry_table_or_create(state, REGISTRY_KEY);
    let stack_top = state.top;
    let record = create_table(state);
    state.push(record);
    table_set_static(
        state,
        record,
        "packedColor",
        Val::Num(f64::from(packed_color)),
    );
    // INFERRED: format the packed AARRGGBB value as lowercase hexadecimal.
    let color_string = format!("{packed_color:08x}");
    let color_string = Val::Str(state.gc.intern_string(color_string.as_bytes()));
    table_set_static(state, record, "colorString", color_string);
    table_set(state, overrides, &override_type.to_string(), record);
    state.top = stack_top;
}

fn remove_override_record(state: &mut LuaState, override_type: u8) {
    let overrides = registry_table_or_create(state, REGISTRY_KEY);
    table_set(state, overrides, &override_type.to_string(), Val::Nil);
}

fn push_color_result(
    state: &mut LuaState,
    packed_color: u32,
    include_alpha: bool,
) -> LuaResult<u32> {
    let color = build_color_table(state, packed_color, include_alpha)?;
    state.push(color);
    Ok(1)
}

fn build_color_table(
    state: &mut LuaState,
    packed_color: u32,
    include_alpha: bool,
) -> LuaResult<Val> {
    let mixin = find_color_mixin(state)?;
    let color_ref = state.gc.alloc_table(Table::new());
    let color = Val::Table(color_ref);
    let stack_top = state.top;
    state.push(color);
    let copied = copy_mixin_members(state, mixin, color_ref);
    if let Err(error) = copied {
        state.top = stack_top;
        return Err(error);
    }

    let [red, green, blue, alpha] = channels_from_packed_color(packed_color);
    table_set_static(state, color, "r", Val::Num(red));
    table_set_static(state, color, "g", Val::Num(green));
    table_set_static(state, color, "b", Val::Num(blue));
    if include_alpha {
        table_set_static(state, color, "a", Val::Num(alpha));
    }
    state.top = stack_top;
    Ok(color)
}

fn publish_override_updated(state: &mut LuaState, override_type: u8) -> LuaResult<()> {
    // INFERRED: the documented update event names the native update callback.
    dispatch_event_now(
        state,
        "COLOR_OVERRIDE_UPDATED",
        &[Val::Num(f64::from(override_type))],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;
    use crate::lua_api::methods::{registry_get, table_get_static};
    use rilua::LuaApiMut;

    const INITIAL_OVERRIDES_LUA: &str = r#"
        ColorMixin = {}
        C_ColorOverrides.SetColorOverride(0, { r = 0.2, g = 0.4, b = 0.6, a = 0.8 })
        C_ColorOverrides.SetColorOverride(5, { r = 0.1, g = 0.3, b = 0.5, a = 0.7 })
    "#;
    const MUTATE_BEFORE_LOAD_LUA: &str = r#"
        ColorMixin = {}
        C_ColorOverrides.SetColorOverride(0, { r = 1, g = 1, b = 1, a = 1 })
        C_ColorOverrides.SetColorOverride(2, { r = 1, g = 0, b = 0, a = 1 })
    "#;
    const ASSERT_RESTORED_OVERRIDES_LUA: &str = r#"
        local poor = C_ColorOverrides.GetColorForQuality(0)
        local legendary = C_ColorOverrides.GetColorForQuality(5)
        local stale = C_ColorOverrides.GetColorOverrideInfo(2)
        if math.abs(poor.r - 51 / 255) > 0.000001 then return "poor_r" end
        if math.abs(poor.a - 204 / 255) > 0.000001 then return "poor_a" end
        if math.abs(legendary.g - 77 / 255) > 0.000001 then return "legendary_g" end
        if math.abs(legendary.a - 179 / 255) > 0.000001 then return "legendary_a" end
        if stale ~= nil then return "stale_type" end
        return "ok"
    "#;

    #[test]
    fn secure_color_override_namespace_matches_master_snapshot() {
        if ACTIVE != ClientProfile::Retail {
            return;
        }
        let env = WowLuaEnv::new().expect("Lua environment should initialize");
        let members = secure_color_override_member_names(&env);
        // origin/master ffc9675 installs these two functions before the secure snapshot.
        assert_eq!(
            members,
            vec![
                "GetColorForQuality".to_owned(),
                "GetColorOverrideInfo".to_owned(),
            ]
        );
    }

    #[test]
    fn player_color_overrides_setting_roundtrips_and_replaces_active_records() {
        if ACTIVE != ClientProfile::Retail {
            return;
        }
        assert_eq!(roundtrip_player_color_overrides_setting(), "ok");
    }

    fn roundtrip_player_color_overrides_setting() -> String {
        let session = tempfile::tempdir().expect("CVar session directory should create");
        let session_path = session.path().to_path_buf();
        let env = WowLuaEnv::with_session_wtf_path(session_path.clone())
            .expect("Lua environment should initialize");
        env.exec(INITIAL_OVERRIDES_LUA)
            .expect("initial records should set");
        save_player_color_overrides(&env);
        assert_setting_contains_overrides(&env);

        let restored = WowLuaEnv::with_session_wtf_path(session_path)
            .expect("restored Lua environment should initialize");
        restored
            .exec(MUTATE_BEFORE_LOAD_LUA)
            .expect("extra records should set");
        load_player_color_overrides(&restored);
        restored
            .eval(ASSERT_RESTORED_OVERRIDES_LUA)
            .expect("loaded records should be queryable")
    }

    fn save_player_color_overrides(env: &WowLuaEnv) {
        let mut lua = env.rilua_mut();
        save_color_overrides(lua.state_mut()).expect("setting save should succeed");
    }

    fn load_player_color_overrides(env: &WowLuaEnv) {
        let mut lua = env.rilua_mut();
        load_color_overrides(lua.state_mut()).expect("setting load should succeed");
    }

    fn assert_setting_contains_overrides(env: &WowLuaEnv) {
        let saved = env
            .state()
            .borrow()
            .cvars
            .get(PLAYER_COLOR_OVERRIDES_CVAR)
            .expect("save should write playerColorOverrides");
        assert!(!saved.is_empty());
    }

    fn secure_color_override_member_names(env: &WowLuaEnv) -> Vec<String> {
        let mut lua = env.rilua_mut();
        let state = lua.state_mut();
        let secureenv = registry_get(state, "__secureenv");
        let namespace = table_get_static(state, secureenv, "C_ColorOverrides");
        let Val::Table(namespace) = namespace else {
            return Vec::new();
        };
        let table = state.gc.tables.get(namespace).expect("namespace table");
        let mut key = Val::Nil;
        let mut members = Vec::new();
        while let Some((next_key, _)) = table.next(key, &state.gc.string_arena).expect("next") {
            if let Val::Str(string) = next_key {
                if let Some(text) = state.gc.string_arena.get(string) {
                    members.push(String::from_utf8_lossy(text.data()).into_owned());
                }
            }
            key = next_key;
        }
        members.sort();
        members
    }
}
