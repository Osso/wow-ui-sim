//! Namespace members (and whole namespaces) the Patch 12.0.5 / 12.0.7
//! consolidated API tables list as removed that no other module retires.
//! Marking keeps the namespace `__index` autostub from fabricating them on
//! ordinary lookup.
use rilua::LuaResult;
use rilua::vm::state::LuaState;

const RETIRED_12_0_5_MEMBERS: &[(&str, &[&str])] = &[
    (
        "C_GossipInfo",
        &["GetActiveDelveGossip", "GetGossipDelveMapID"],
    ),
    ("C_NamePlateManager", &["SetNamePlateHitTestFrame"]),
];

/// Removed whole; its members moved to `C_PhotoSharing`.
const RETIRED_12_0_5_NAMESPACES: &[&str] = &["C_HousingPhotoSharing"];

#[cfg(feature = "retail-12-0-7")]
const RETIRED_12_0_7_MEMBERS: &[(&str, &[&str])] = &[
    ("C_DurationUtil", &["GetCurrentTime"]),
    ("C_HousingLayout", &["IsDraggingStairwell"]),
    ("C_Minimap", &["GetObjectIconTextureCoords"]),
    ("C_Scenario", &["GetScenarioIconInfo"]),
];

pub(crate) fn mark_retired_members(state: &mut LuaState) -> LuaResult<()> {
    mark_members(state, RETIRED_12_0_5_MEMBERS)?;
    mark_namespaces_absent(state, RETIRED_12_0_5_NAMESPACES)?;
    #[cfg(feature = "retail-12-0-7")]
    mark_members(state, RETIRED_12_0_7_MEMBERS)?;
    Ok(())
}

fn mark_members(state: &mut LuaState, retired: &[(&'static str, &[&str])]) -> LuaResult<()> {
    for (namespace, members) in retired {
        let table = super::ensure_namespace(state, namespace)?;
        super::mark_namespace_keys_removed(state, table, members);
    }
    Ok(())
}

fn mark_namespaces_absent(state: &mut LuaState, names: &[&'static str]) -> LuaResult<()> {
    let absent = super::ensure_namespace(state, "__wow_absent_namespaces")?;
    for name in names {
        crate::lua_api::methods::table_set_static(
            state,
            rilua::Val::Table(absent),
            name,
            rilua::Val::Bool(true),
        );
    }
    Ok(())
}
