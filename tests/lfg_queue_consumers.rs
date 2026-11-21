use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-retail")]
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

        local entries = C_LFGInfo.GetAllEntriesForCategory(LE_LFG_CATEGORY_LAIR)
        assert(#entries == 2 and entries[1] == 1203 and entries[2] == 1204,
            'queue category must retain both supplied entry IDs')

        local menuMixin = MenuVariants.GetDefaultContextMenuMixin()
        local function assertQueueStatusTitle(category, entryID)
            local description = MenuUtil.CreateRootMenuDescription(menuMixin)
            QueueStatusDropdown_AddLFGButtons(description, category)
            local count = 0
            for _, row in description:EnumerateElementDescriptions() do
                count = count + 1
                assert(MenuUtil.GetElementText(row) == GetLFGDungeonInfo(entryID),
                    'queue-status title must resolve the supplied entry ID')
            end
            assert(count == 1, 'each category must produce one queue-status title row')
        end

        assertQueueStatusTitle(LE_LFG_CATEGORY_LAIR, 1203)
        assertQueueStatusTitle(LE_LFG_CATEGORY_SCENARIO, 1204)
        "#,
    )
    .expect("the selected QueueStatusFrame consumer must display supplied category entries");
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

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_keeps_empty_legacy_lfg_defaults() {
    let env = WowLuaEnv::new().expect("lua env should initialize");
    let result: String = env
        .eval(
            "return tostring(#C_LFGInfo.GetLFDLockStates()) .. ',' .. tostring(#C_LFGInfo.GetAllEntriesForCategory(8))",
        )
        .expect("Forever legacy LFG defaults should remain callable");
    assert_eq!(result, "0,0");
}
