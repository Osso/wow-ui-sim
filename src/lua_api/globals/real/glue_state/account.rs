//! R88ad.7 saved-account and token-login settings (Retail 12.1.5.70688).
//!
//! The getters publish the named setting text or its empty fallback. The
//! setters submit text to a present setting only; an absent setting skips the
//! write. The native setter's acceptance result is not returned.

use crate::c_api::helpers::{read_untainted_string_argument, read_untainted_truthy_argument};
use crate::lua_api::methods::{borrow_state, create_string};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, RustFn};

const NAME_SETTING: &str = "accountName";
const LIST_SETTING: &str = "accountList";
const TOKEN_SETTING: &str = "g_accountUsesToken";

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let globals: [(&str, RustFn); 5] = [
        ("GetSavedAccountName", get_saved_account_name),
        ("SetSavedAccountName", set_saved_account_name),
        ("GetSavedAccountList", get_saved_account_list),
        ("ClearSavedAccountList", clear_saved_account_list),
        ("SetUsesToken", set_uses_token),
    ];
    for (name, function) in globals {
        LuaApiMut::register_function(state, name, function)?;
    }
    Ok(())
}

fn get_saved_account_name(state: &mut LuaState) -> LuaResult<u32> {
    push_setting_text(state, NAME_SETTING)
}

fn get_saved_account_list(state: &mut LuaState) -> LuaResult<u32> {
    push_setting_text(state, LIST_SETTING)
}

fn push_setting_text(state: &mut LuaState, name: &str) -> LuaResult<u32> {
    let text = borrow_state(state)?.cvars.get(name).unwrap_or_default();
    let value = create_string(state, &text);
    state.push(value);
    Ok(1)
}

fn set_saved_account_name(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: SetSavedAccountName(accountName)";
    let bytes = read_untainted_string_argument(state, 1, "SetSavedAccountName", USAGE)?;
    // The native copy stops at the first NUL.
    let text = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
    write_present_setting(state, NAME_SETTING, &String::from_utf8_lossy(text))?;
    Ok(0)
}

fn clear_saved_account_list(state: &mut LuaState) -> LuaResult<u32> {
    write_present_setting(state, LIST_SETTING, "")?;
    Ok(0)
}

fn set_uses_token(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: SetUsesToken(usesToken)";
    let uses_token = read_untainted_truthy_argument(state, 1, "SetUsesToken", USAGE)?;
    write_present_setting(state, TOKEN_SETTING, if uses_token { "1" } else { "0" })?;
    Ok(0)
}

fn write_present_setting(state: &LuaState, name: &str, value: &str) -> LuaResult<()> {
    let sim = borrow_state(state)?;
    if sim.cvars.get(name).is_some() {
        sim.cvars.set(name, value);
    }
    Ok(())
}
