//! Battle.net account GUID type query (WM80).

use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const USAGE: &str = "Usage: local isBNet = C_AccountInfo.IsGUIDBattleNetAccountType(guid)";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    use crate::client_profile::{ACTIVE, ClientProfile};
    if cfg!(feature = "retail-12-1-5")
        && matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr)
    {
        let namespace = super::ensure_namespace(state, "C_AccountInfo")?;
        table_set_rust_fn_static(state, namespace, "IsGUIDBattleNetAccountType", is_bnet_guid)?;
    }
    Ok(())
}

fn is_bnet_guid(state: &mut LuaState) -> LuaResult<u32> {
    let argument = rilua::table_security::unwrap_secret(state, stack_val(state, 1)).map_err(|_| {
        runtime_error(format!(
            "{USAGE}. Secret values are only allowed during untainted execution for this argument."
        ))
    })?;
    let result = match argument {
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .is_some_and(|value| is_bnet_guid_text(value.data())),
        Val::Num(_) => false,
        _ => return Err(runtime_error(USAGE)),
    };
    state.push(Val::Bool(result));
    Ok(1)
}

fn is_bnet_guid_text(text: &[u8]) -> bool {
    const PREFIX: &[u8] = b"BNetAccount-";
    if !text
        .get(..PREFIX.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(PREFIX))
    {
        return false;
    }
    let mut component = 0_u64;
    for byte in text[PREFIX.len()..]
        .iter()
        .take(20)
        .take_while(|byte| byte.is_ascii_digit())
    {
        component = component
            .wrapping_mul(10)
            .wrapping_add(u64::from(byte - b'0'));
    }
    component < (1_u64 << 58)
}
