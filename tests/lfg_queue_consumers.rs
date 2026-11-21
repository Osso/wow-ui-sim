#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn supplied_entries_reach_selected_queue_status_rows(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_QueueStatusFrame'))
        A_Admin.SetLfgQueueData(
            {
                { categoryID = LE_LFG_CATEGORY_LAIR, entryID = 1203 },
                { categoryID = LE_LFG_CATEGORY_LAIR, entryID = 1204 },
                { categoryID = LE_LFG_CATEGORY_SCENARIO, entryID = 1204 },
            },
            {}
        )
        SetLFGDungeon(LE_LFG_CATEGORY_LAIR, 1203)
        JoinLFG(LE_LFG_CATEGORY_LAIR)
        SetLFGDungeon(LE_LFG_CATEGORY_SCENARIO, 1204)
        JoinLFG(LE_LFG_CATEGORY_SCENARIO)
        QueueStatusFrame:Update()
        local firstRow = QueueStatusFrame:GetEntry(1)
        local secondRow = QueueStatusFrame:GetEntry(2)
        assert(firstRow.Title:GetText() == GetLFGDungeonInfo(1203),
            'Lair queue-status row must use its first supplied entry ID')
        assert(secondRow.Title:GetText() == GetLFGDungeonInfo(1204),
            'Scenario queue-status row must use its supplied entry ID')
        "#,
    )
    .expect("the selected QueueStatusFrame must display supplied category entry IDs");
}
}

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn supplied_locks_reach_selected_group_finder_frame(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_GroupFinder'))
        A_Admin.SetLfgQueueData(
            {},
            {
                { lfgID = 1203, reason = 1, hideEntry = false },
                { lfgID = 1204, reason = 2, hideEntry = true },
            }
        )
        LFGEventFrame_OnEvent(LFGEventFrame, 'LFG_LOCK_INFO_RECEIVED')
        assert(LFGLockList[1203].reason == 1 and not LFGLockList[1203].hideEntry,
            'GroupFinder frame must consume the first supplied lock row')
        assert(LFGLockList[1204].reason == 2 and LFGLockList[1204].hideEntry,
            'GroupFinder frame must consume the second supplied lock row')
        "#,
    )
    .expect("the selected GroupFinder frame must consume both supplied lock rows");
}
}
