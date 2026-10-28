//! PTR VoiceChatDocumentation.lua:1476–1484.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{create_table, table_set};
use rilua::Val;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) {
    if ACTIVE != ClientProfile::Ptr {
        return;
    }
    let enums = super::helpers::ensure_global_table(state, "Enum");
    let values = create_table(state);
    table_set(state, values, "Legacy", Val::Num(0.0));
    table_set(state, values, "Discord", Val::Num(1.0));
    table_set(state, enums, "VoiceProviderID", values);
    let metadata = create_table(state);
    table_set(state, metadata, "MinValue", Val::Num(0.0));
    table_set(state, metadata, "MaxValue", Val::Num(1.0));
    table_set(state, metadata, "NumValues", Val::Num(2.0));
    table_set(state, enums, "VoiceProviderIDMeta", metadata);
}
