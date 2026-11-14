//! Registry-rooted addon chunk namespaces; load completion stays in AddonInfo.

use crate::lua_api::methods::{
    borrow_state, create_table, registry_get, registry_table_or_create, table_get,
    table_get_static, table_set, table_set_static,
};
use crate::toc::TocFile;
use rilua::{LuaResult, Val, vm::state::LuaState};

const LOCAL_TABLES: &str = "__addon_local_tables";

/// Retain the actual chunk argument before any files compile or execute.
/// Both namespace and record are stack roots during registry construction.
pub(crate) fn retain_namespace(state: &mut LuaState, name: &str, namespace: Val, toc: &TocFile) {
    state.push(namespace);
    let records = registry_table_or_create(state, LOCAL_TABLES);
    let record = create_table(state);
    state.push(record);
    table_set(state, records, name, record);
    table_set_static(state, record, "namespace", namespace);
    table_set_static(
        state,
        record,
        "opted_in",
        Val::Bool(
            toc.metadata
                .get("AllowAddOnTableAccess")
                .is_some_and(|value| value == "1"),
        ),
    );
    table_set_static(
        state,
        record,
        "blizzard",
        Val::Bool(toc.loads_as_blizzard_code()),
    );
    state.pop();
    state.pop();
}

/// No target loading or fabricated namespace. Absence errors are modeled policy.
pub(super) fn get_namespace(state: &mut LuaState, name: &str) -> LuaResult<Val> {
    require_loaded_addon(state, name)?;
    let records = registry_get(state, LOCAL_TABLES);
    let record = table_get(state, records, name);
    if !matches!(table_get_static(state, record, "opted_in"), Val::Bool(true)) {
        return Err(rilua::runtime_error(
            "addon has not enabled AllowAddOnTableAccess: 1",
        ));
    }
    if matches!(table_get_static(state, record, "blizzard"), Val::Bool(true))
        && !rilua::api::state_is_secure(state)
    {
        return Err(rilua::runtime_error(
            "insecure code cannot query Blizzard addon tables",
        ));
    }
    let namespace = table_get_static(state, record, "namespace");
    if !matches!(namespace, Val::Table(_)) {
        return Err(rilua::runtime_error("addon namespace is unavailable"));
    }
    Ok(namespace)
}

fn require_loaded_addon(state: &LuaState, name: &str) -> LuaResult<()> {
    let loaded = borrow_state(state)?
        .addons
        .iter()
        .any(|addon| addon.folder_name == name && addon.loaded);
    if loaded {
        Ok(())
    } else {
        Err(rilua::runtime_error(
            "C_AddOns.GetAddOnLocalTable requires a loaded addon",
        ))
    }
}
