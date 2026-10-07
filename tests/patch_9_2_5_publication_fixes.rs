//! Removed members stay absent after repeated namespace lookup.
#![cfg(feature = "client-retail")]

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
for _, entry in ipairs({
    {'C_Calendar', 'ContextMenuEventComplain'},
    {'C_Cursor', 'DropCursorCommunitiesStream'},
    {'C_Cursor', 'GetCursorCommunitiesStream'},
    {'C_Cursor', 'SetCursorCommunitiesStream'},
    {'C_LFGList', 'ReportSearchResult'},
    {'C_ReportSystem', 'OpenReportPlayerDialog'},
    {'C_ReportSystem', 'SetPendingReportPetTarget'},
    {'C_ReportSystem', 'SetPendingReportTargetByGuid'},
    {'C_ReportSystem', 'SetPendingReportTarget'},
}) do
    local namespace, member = _G[entry[1]], entry[2]
    assert(rawget(namespace, member) == nil, entry[1] .. "." .. member)
    assert(namespace[member] == nil, entry[1] .. "." .. member)
    assert(namespace[member] == nil, entry[1] .. "." .. member)
end
assert(type(C_ReportSystem.InitiateReportPlayer) == "function")
assert(type(C_ReportSystem.SendReportPlayer) == "function")
"#;

#[test]
fn patch_9_2_5_unused_members_stay_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
