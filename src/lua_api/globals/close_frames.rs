//! Frame-close verbs that flip per-kind `SimState.*_open` flags and fire
//! the matching `*_CLOSED` event.
//!
//! Migrates 9 entries off `GLOBAL_NIL_STUBS`:
//!
//! | Function            | SimState flag cleared        | Event fired         |
//! |---------------------|------------------------------|---------------------|
//! | CloseBankFrame      | bank_frame_open              | BANKFRAME_CLOSED    |
//! | CloseGuildBankFrame | guild_bank_frame_open        | GUILDBANKFRAME_CLOSED |
//! | CloseMerchant       | merchant_frame_open          | MERCHANT_CLOSED     |
//! | CloseTabardCreation | tabard_frame_open            | TABARD_CANCELED     |
//! | CloseTrainerFrame   | trainer_frame_open           | TRAINER_CLOSED      |
//! | CloseSocketInfo     | socket_frame_open            | SOCKET_INFO_CLOSE   |
//! | CloseLoot           | loot_frame_open              | LOOT_CLOSED         |
//! | CloseGuildRegistrar | guild_registrar_open         | PETITION_CLOSED     |
//! `ClosePetStables` is a classic-only wrapper for the `C_StableInfo` model.
//!
//! `CloseInbox` is handled by `mail_verbs.rs` since it fires `MAIL_CLOSED`
//! and belongs to the mail pipeline.
//!
//! Flags are always cleared (idempotent); events always fire so addons
//! observing a "close at startup" sequence still see the drained edges.
//!
//! Registered from `register_tail_globals` after `missing_surface`.

use crate::event::Event;
use crate::lua_api::methods::borrow_state_mut;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult};

fn push_event(state: &mut LuaState, name: &str) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: name.to_string(),
        args: Vec::new(),
    });
    Ok(())
}

macro_rules! define_close_verb {
    ($fn_name:ident, $field:ident, $event:literal) => {
        fn $fn_name(state: &mut LuaState) -> LuaResult<u32> {
            borrow_state_mut(state)?.$field = false;
            push_event(state, $event)?;
            Ok(0)
        }
    };
}

define_close_verb!(close_bank_frame, bank_frame_open, "BANKFRAME_CLOSED");
define_close_verb!(
    close_guild_bank_frame,
    guild_bank_frame_open,
    "GUILDBANKFRAME_CLOSED"
);
define_close_verb!(close_merchant, merchant_frame_open, "MERCHANT_CLOSED");
define_close_verb!(close_tabard_creation, tabard_frame_open, "TABARD_CANCELED");
define_close_verb!(close_trainer_frame, trainer_frame_open, "TRAINER_CLOSED");
define_close_verb!(close_socket_info, socket_frame_open, "SOCKET_INFO_CLOSE");
define_close_verb!(close_loot, loot_frame_open, "LOOT_CLOSED");
define_close_verb!(
    close_guild_registrar,
    guild_registrar_open,
    "PETITION_CLOSED"
);

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "CloseBankFrame", close_bank_frame)?;
    LuaApiMut::register_function(lua, "CloseGuildBankFrame", close_guild_bank_frame)?;
    LuaApiMut::register_function(lua, "CloseMerchant", close_merchant)?;
    LuaApiMut::register_function(lua, "CloseTabardCreation", close_tabard_creation)?;
    LuaApiMut::register_function(lua, "CloseTrainerFrame", close_trainer_frame)?;
    LuaApiMut::register_function(lua, "CloseSocketInfo", close_socket_info)?;
    LuaApiMut::register_function(lua, "CloseLoot", close_loot)?;
    LuaApiMut::register_function(lua, "CloseGuildRegistrar", close_guild_registrar)?;
    #[cfg(not(any(feature = "client-retail", feature = "client-ptr")))]
    LuaApiMut::register_function(
        lua,
        "ClosePetStables",
        crate::c_api::c_stable_info::close_pet_stables,
    )?;
    Ok(())
}
