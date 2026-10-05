//! Namespace members the Patch 12.0.7 consolidated API table lists as removed
//! that no other module retires. Marking keeps the namespace `__index`
//! autostub from fabricating them on ordinary lookup.
use rilua::LuaResult;
use rilua::vm::state::LuaState;

const RETIRED_MEMBERS: &[(&str, &[&str])] = &[
    ("C_DurationUtil", &["GetCurrentTime"]),
    ("C_HousingLayout", &["IsDraggingStairwell"]),
    ("C_Minimap", &["GetObjectIconTextureCoords"]),
    ("C_Scenario", &["GetScenarioIconInfo"]),
];

pub(crate) fn mark_retired_members(state: &mut LuaState) -> LuaResult<()> {
    for (namespace, members) in RETIRED_MEMBERS {
        let table = super::ensure_namespace(state, namespace)?;
        super::mark_namespace_keys_removed(state, table, members);
    }
    Ok(())
}
