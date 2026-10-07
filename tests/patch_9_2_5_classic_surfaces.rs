//! Mists namespace lookup is unaffected by retail retirement.
#![cfg(feature = "client-mists")]

#[test]
fn patch_9_2_5_mists_preserves_legacy_lookup() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(r#"
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
    assert(type(_G[entry[1]][entry[2]]) == "function", entry[1] .. "." .. entry[2])
end
"#).unwrap();
}
