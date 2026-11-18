//! PTR VoiceChatDocumentation.lua:1476–1484.

use crate::client_profile::{ACTIVE, ClientProfile};
use rilua::Val;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) {
    if ACTIVE != ClientProfile::Ptr {
        return;
    }
    let enums = super::helpers::ensure_enum_table(state);
    let values = super::helpers::ensure_paired_table_field(state, enums, "VoiceProviderID")
        .expect("Enum group must have a secure registration peer");
    super::helpers::set_namespace_member(state, values, "Legacy", Val::Num(0.0))
        .expect("Enum group must have a secure registration peer");
    super::helpers::set_namespace_member(state, values, "Discord", Val::Num(1.0))
        .expect("Enum group must have a secure registration peer");
    let metadata = super::helpers::ensure_paired_table_field(state, enums, "VoiceProviderIDMeta")
        .expect("Enum metadata must have a secure registration peer");
    super::helpers::set_namespace_member(state, metadata, "MinValue", Val::Num(0.0))
        .expect("Enum metadata must have a secure registration peer");
    super::helpers::set_namespace_member(state, metadata, "MaxValue", Val::Num(1.0))
        .expect("Enum metadata must have a secure registration peer");
    super::helpers::set_namespace_member(state, metadata, "NumValues", Val::Num(2.0))
        .expect("Enum metadata must have a secure registration peer");
}
