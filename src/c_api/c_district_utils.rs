//! Host-supplied superdistrict information for Forever and PTR.

use super::helpers::bad_argument;
use crate::c_api::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set, table_set_static, val_to_string,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const GET_SUPER_DISTRICT_INFO_USAGE: &str =
    "Usage: local info = C_DistrictUtils.GetSuperDistrictInfo(superDistrictID)";
const INT32_RANGE_ERROR: &str = "outside of expected range -2147483648 to 2147483647";

/// One host-supplied record returned by `C_DistrictUtils.GetSuperDistrictInfo`.
#[derive(Debug, Clone, Default)]
pub struct SuperDistrictInfo {
    pub super_district_id: i32,
    pub display_name: String,
    pub description: String,
    pub icon_file_data_id: u32,
    pub disallow_login: bool,
    pub no_realms_available: bool,
    pub override_bg_atlas: String,
    pub override_bg_glow_atlas: String,
}

/// Register PTR's Game-surface district lookup during native API setup.
pub(crate) fn register_ptr_game_surface(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE == ClientProfile::Ptr {
        update_registration(state, true)
    } else {
        Ok(())
    }
}

/// Update the profile's `GetSuperDistrictInfo` registration for the selected screen.
pub(crate) fn update_screen_registration(env: &WowLuaEnv, is_glue: bool) {
    if !matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
        return;
    }

    let should_register = match ACTIVE {
        ClientProfile::Ptr => true,
        ClientProfile::WowForever => is_glue,
        _ => false,
    };
    let mut lua = env.lua.borrow_mut();
    update_registration(lua.state_mut(), should_register)
        .expect("register C_DistrictUtils.GetSuperDistrictInfo on the selected screen");
}

fn update_registration(state: &mut LuaState, should_register: bool) -> LuaResult<()> {
    let namespace = match LuaApiMut::get_global_val(state, "C_DistrictUtils") {
        Val::Table(namespace) => namespace,
        _ if should_register => ensure_namespace(state, "C_DistrictUtils")?,
        _ => return Ok(()),
    };

    if should_register {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetSuperDistrictInfo",
            get_super_district_info,
        )
    } else {
        table_set_static(
            state,
            Val::Table(namespace),
            "GetSuperDistrictInfo",
            Val::Nil,
        );
        Ok(())
    }
}

fn get_super_district_info(state: &mut LuaState) -> LuaResult<u32> {
    let super_district_id =
        read_super_district_id(state, "GetSuperDistrictInfo", GET_SUPER_DISTRICT_INFO_USAGE)?;
    let info = borrow_state(state)?
        .super_district_info_by_id
        .get(&super_district_id)
        .cloned();
    let Some(info) = info else {
        return Ok(0);
    };

    push_super_district_info(state, &info);
    Ok(1)
}

fn push_super_district_info(state: &mut LuaState, info: &SuperDistrictInfo) {
    let result = create_table(state);
    state.push(result);
    set_super_district_identity_fields(state, result, info);
    set_super_district_access_fields(state, result, info);
    set_super_district_background_fields(state, result, info);
}

fn set_super_district_identity_fields(state: &mut LuaState, table: Val, info: &SuperDistrictInfo) {
    table_set(
        state,
        table,
        "superDistrictID",
        Val::Num(f64::from(info.super_district_id)),
    );
    set_string_field(state, table, "displayName", &info.display_name);
    set_string_field(state, table, "description", &info.description);
    table_set(
        state,
        table,
        "iconFileDataID",
        Val::Num(f64::from(info.icon_file_data_id)),
    );
}

fn set_super_district_access_fields(state: &mut LuaState, table: Val, info: &SuperDistrictInfo) {
    table_set(
        state,
        table,
        "disallowLogin",
        Val::Bool(info.disallow_login),
    );
    table_set(
        state,
        table,
        "noRealmsAvailable",
        Val::Bool(info.no_realms_available),
    );
}

fn set_super_district_background_fields(
    state: &mut LuaState,
    table: Val,
    info: &SuperDistrictInfo,
) {
    set_string_field(state, table, "overrideBgAtlas", &info.override_bg_atlas);
    set_string_field(
        state,
        table,
        "overrideBgGlowAtlas",
        &info.override_bg_glow_atlas,
    );
}

fn set_string_field(state: &mut LuaState, table: Val, field: &str, value: &str) {
    let value = create_string(state, value);
    table_set(state, table, field, value);
}

pub(crate) fn read_super_district_id(
    state: &LuaState,
    function_name: &str,
    usage: &str,
) -> LuaResult<i32> {
    let argument = stack_val(state, 1);
    if matches!(argument, Val::Nil) {
        return Err(bad_argument(1, function_name, usage));
    }

    let Some(value) = numeric_argument(state, argument) else {
        return super_district_id_range_error(function_name, usage);
    };
    if !value.is_finite() || value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return super_district_id_range_error(function_name, usage);
    }

    Ok(value.trunc() as i32)
}

fn numeric_argument(state: &LuaState, value: Val) -> Option<f64> {
    match value {
        Val::Num(number) => Some(number),
        Val::Str(_) => val_to_string(state, value).and_then(|text| parse_numeric_string(&text)),
        _ => None,
    }
}

fn parse_numeric_string(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    let (sign, unsigned) = if let Some(unsigned) = trimmed.strip_prefix('-') {
        (-1.0, unsigned)
    } else if let Some(unsigned) = trimmed.strip_prefix('+') {
        (1.0, unsigned)
    } else {
        (1.0, trimmed)
    };

    if let Some(hexadecimal) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        return u64::from_str_radix(hexadecimal, 16)
            .ok()
            .map(|number| sign * number as f64);
    }

    trimmed.parse().ok()
}

fn super_district_id_range_error(function_name: &str, usage: &str) -> LuaResult<i32> {
    let detail = format!("{INT32_RANGE_ERROR} - {usage}");
    Err(bad_argument(1, function_name, &detail))
}
