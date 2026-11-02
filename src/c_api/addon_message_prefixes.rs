//! Session-local addon-message subscriptions, independent of outbound delivery.
//!
//! Blizzard ChatInfo/ChatConstants documentation defines the three queries and
//! result codes. The published 4.1 API announcement supplies the 16-byte prefix
//! and 512-prefix client limits; current native boundary behavior is unmeasured.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table, table_set_num};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const PREFIX_BYTE_LIMIT: usize = 16;
const PREFIX_CAPACITY: usize = 512;
const SUCCESS: u32 = 0;
const DUPLICATE_PREFIX: u32 = 1;
const INVALID_PREFIX: u32 = 2;
const MAX_PREFIXES: u32 = 3;

#[derive(Debug, Default)]
pub(crate) struct AddonMessagePrefixes {
    prefixes: Vec<Vec<u8>>,
}

impl AddonMessagePrefixes {
    fn register(&mut self, prefix: Vec<u8>) -> u32 {
        if prefix.is_empty() || prefix.len() > PREFIX_BYTE_LIMIT {
            return INVALID_PREFIX;
        }
        if self.prefixes.contains(&prefix) {
            return DUPLICATE_PREFIX;
        }
        if self.prefixes.len() == PREFIX_CAPACITY {
            return MAX_PREFIXES;
        }
        self.prefixes.push(prefix);
        SUCCESS
    }
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_ChatInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "RegisterAddonMessagePrefix",
        register_prefix,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsAddonMessagePrefixRegistered",
        is_prefix_registered,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetRegisteredAddonMessagePrefixes",
        get_registered_prefixes,
    )
}

fn read_prefix(state: &LuaState) -> LuaResult<Vec<u8>> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    let Val::Str(reference) = value else {
        return Err(runtime_error("addon message prefix must be a string"));
    };
    let bytes = state
        .gc
        .string_arena
        .get(reference)
        .expect("rooted addon-message prefix")
        .data();
    // INFERRED cstring conversion: embedded NUL terminates the native prefix.
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    Ok(bytes[..end].to_vec())
}

fn register_prefix(state: &mut LuaState) -> LuaResult<u32> {
    let prefix = read_prefix(state)?;
    let result = borrow_state_mut(state)?
        .addon_message_prefixes
        .register(prefix);
    state.push(Val::Num(f64::from(result)));
    Ok(1)
}

fn is_prefix_registered(state: &mut LuaState) -> LuaResult<u32> {
    let prefix = read_prefix(state)?;
    let registered = borrow_state(state)?
        .addon_message_prefixes
        .prefixes
        .contains(&prefix);
    state.push(Val::Bool(registered));
    Ok(1)
}

fn get_registered_prefixes(state: &mut LuaState) -> LuaResult<u32> {
    let prefixes = borrow_state(state)?.addon_message_prefixes.prefixes.clone();
    let array = create_table(state);
    state.push(array);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, prefix) in prefixes.iter().enumerate() {
        let prefix = Val::Str(state.gc.intern_string(prefix));
        table_set_num(state, table, (index + 1) as f64, prefix);
    }
    Ok(1)
}
