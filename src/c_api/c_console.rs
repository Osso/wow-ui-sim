//! Console registry: live CVar storage plus epoch-scoped native command names.

use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const CONSOLE_CATEGORY_NONE: f64 = 10.0;
const CONSOLE_COMMAND_TYPE_CVAR: f64 = 0.0;
const CONSOLE_COMMAND_TYPE_COMMAND: f64 = 1.0;

pub(crate) fn get_all_commands(state: &mut LuaState) -> LuaResult<u32> {
    let mut records: Vec<_> = borrow_state(state)?
        .cvars
        .all_keys()
        .into_iter()
        .map(|name| (name, CONSOLE_COMMAND_TYPE_CVAR))
        .collect();
    records.extend(
        native_command_names(
            crate::client_profile::ACTIVE,
            crate::client_profile::ACTIVE_INTERFACE_VERSION,
        )
        .map(|name| (name.to_owned(), CONSOLE_COMMAND_TYPE_COMMAND)),
    );
    // INFERRED: stable case-insensitive ordering; native ordering is undocumented.
    records.sort_by_key(|(name, _)| name.to_lowercase());
    let commands = create_table(state);
    for (index, (name, command_type)) in records.iter().enumerate() {
        let entry = create_command_record(state, name, *command_type);
        set_table_array(state, commands, index as i64 + 1, entry);
    }
    state.push(commands);
    Ok(1)
}

fn native_command_names(
    profile: crate::client_profile::ClientProfile,
    version: u32,
) -> impl Iterator<Item = &'static str> {
    // 11.1.7/11.2.7 additions precede all supported retail epochs. These are
    // catalog records only: graphics/neighborhood command execution is unmodeled.
    const BASELINE: &[&str] = &[
        "GxMemReport",
        "GxSimulateEvent",
        "NeighborhoodAddManager",
        "NeighborhoodCancelInvitation",
        "NeighborhoodGetInvites",
        "NeighborhoodInviteResident",
        "NeighborhoodRemoveManager",
        "NeighborhoodSetName",
        "NeighborhoodSetPublic",
        "OfferNeighborhoodOwnership",
        "PlayerDeclineHousingInvitation",
        "PlayerGetHousingInvitation",
    ];
    let retail = profile == crate::client_profile::ClientProfile::Retail;
    let baseline = if retail && version >= 120000 {
        BASELINE
    } else {
        &[]
    };
    // INFERRED: later additions remain published, not on unmeasured profiles.
    let later: &[&str] = if retail && version >= 120007 {
        &["fetchBleepProxies", "MemUsageStackTrace"]
    } else {
        &[]
    };
    baseline.iter().chain(later).copied()
}

fn create_command_record(state: &mut LuaState, name: &str, command_type: f64) -> Val {
    let entry = create_table(state);
    let command = create_string(state, name);
    table_set(state, entry, "command", command);
    table_set(state, entry, "category", Val::Num(CONSOLE_CATEGORY_NONE));
    table_set(state, entry, "commandType", Val::Num(command_type));
    // INFERRED: absent native metadata uses None category and empty required
    // strings, not fabricated help or script semantics. No macros/scripts modeled.
    let empty = create_string(state, "");
    for field in ["help", "scriptContents", "scriptParameters"] {
        table_set(state, entry, field, empty);
    }
    entry
}
