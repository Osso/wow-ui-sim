//! Battle.net account GUID queries (WM80 and WM80b).

use crate::lua_api::globals::unit_misc::existing_guid_for_unit;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const TYPE_USAGE: &str = "Usage: local isBNet = C_AccountInfo.IsGUIDBattleNetAccountType(guid)";
const ID_USAGE: &str = "Usage: local battleNetAccountID = C_AccountInfo.GetIDFromBattleNetAccountGUID(battleNetAccountGUID)";
const RELATED_USAGE: &str =
    "Usage: local isLocalUser = C_AccountInfo.IsGUIDRelatedToLocalAccount(guid)";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    use crate::client_profile::{ACTIVE, ClientProfile};
    if cfg!(feature = "retail-12-1-5")
        && matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr)
    {
        let namespace = super::ensure_namespace(state, "C_AccountInfo")?;
        table_set_rust_fn_static(state, namespace, "IsGUIDBattleNetAccountType", is_bnet_guid)?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetIDFromBattleNetAccountGUID",
            get_bnet_id,
        )?;
        table_set_rust_fn_static(
            state,
            namespace,
            "IsGUIDRelatedToLocalAccount",
            is_local_guid,
        )?;
    }
    Ok(())
}

fn guid_argument(state: &LuaState, usage: &str) -> LuaResult<Option<Vec<u8>>> {
    let argument = rilua::table_security::unwrap_secret(state, stack_val(state, 1)).map_err(|_| {
        runtime_error(format!(
            "{usage}. Secret values are only allowed during untainted execution for this argument."
        ))
    })?;
    match argument {
        Val::Str(reference) => Ok(state
            .gc
            .string_arena
            .get(reference)
            .map(|value| value.data().to_vec())),
        // Converting a number to text cannot produce a GUID prefix.
        Val::Num(_) => Ok(None),
        _ => Err(runtime_error(usage)),
    }
}

fn is_bnet_guid(state: &mut LuaState) -> LuaResult<u32> {
    let text = guid_argument(state, TYPE_USAGE)?;
    let result = text.as_deref().and_then(parse_bnet_guid).is_some();
    state.push(Val::Bool(result));
    Ok(1)
}

fn get_bnet_id(state: &mut LuaState) -> LuaResult<u32> {
    let text = guid_argument(state, ID_USAGE)?;
    let id = text
        .as_deref()
        .and_then(parse_bnet_guid)
        .map(|(_, id)| id as u32);
    state.push(id.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(1)
}

fn is_local_guid(state: &mut LuaState) -> LuaResult<u32> {
    let text = guid_argument(state, RELATED_USAGE)?;
    let related = if let Some(text) = text.as_deref() {
        if parse_bnet_guid(text).is_some() {
            // No local Battle.net identity is currently modeled.
            false
        } else if let Ok(guid) = std::str::from_utf8(text) {
            let sim = borrow_state(state)?;
            existing_guid_for_unit(&sim, "player").as_deref() == Some(guid)
        } else {
            false
        }
    } else {
        false
    };
    state.push(Val::Bool(related));
    Ok(1)
}

fn parse_bnet_guid(text: &[u8]) -> Option<(u64, u64)> {
    const PREFIX: &[u8] = b"BNetAccount-";
    if !text
        .get(..PREFIX.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(PREFIX))
    {
        return None;
    }
    let text = &text[PREFIX.len()..];
    let digits = text
        .iter()
        .take(20)
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    let decimal = text[..digits].iter().fold(0_u64, |value, byte| {
        value.wrapping_mul(10).wrapping_add(u64::from(byte - b'0'))
    });
    if decimal >= (1_u64 << 58) {
        return None;
    }
    let hex_start = (digits + 1).min(text.len());
    let mut hex = 0_u64;
    for byte in text[hex_start..].iter().take(16) {
        let Some(value) = (*byte as char).to_digit(16) else {
            break;
        };
        hex = (hex << 4) | u64::from(value);
    }
    Some((decimal, hex))
}
