#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn load_selected_lfg_consumers() -> WowLuaEnv {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let selected = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for addon in ["Blizzard_GroupFinder", "Blizzard_QueueStatusFrame"] {
        assert!(
            selected.iter().any(|(name, _)| name == addon),
            "{addon} must be selected by the active Game TOC"
        );
    }
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_GroupFinder", "Blizzard_QueueStatusFrame"],
        &[],
    );
    for addon in ["Blizzard_GroupFinder", "Blizzard_QueueStatusFrame"] {
        assert!(loaded.iter().any(|name| name == addon), "{addon} must load");
    }
    env
}

#[test]
fn supplied_lfg_queue_entries_and_locks_reach_selected_consumers() {
    let env = load_selected_lfg_consumers();
    assert_lfg_consumers_use_supplied_data(&env);
}

fn assert_lfg_consumers_use_supplied_data(env: &WowLuaEnv) {
    env.exec(
        r#"
        A_Admin.SetLfgQueueData(
            {
                { categoryID = LE_LFG_CATEGORY_LAIR, entryID = 1203 },
                { categoryID = LE_LFG_CATEGORY_LAIR, entryID = 1204 },
                { categoryID = LE_LFG_CATEGORY_SCENARIO, entryID = 1204 },
            },
            {
                { lfgID = 1203, reason = 1, hideEntry = false },
                { lfgID = 1204, reason = 2, hideEntry = true },
            }
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

        LFGEventFrame_OnEvent(LFGEventFrame, 'LFG_LOCK_INFO_RECEIVED')
        assert(LFGLockList[1203].reason == 1 and not LFGLockList[1203].hideEntry,
            'GroupFinder frame must consume the first supplied lock row')
        assert(LFGLockList[1204].reason == 2 and LFGLockList[1204].hideEntry,
            'GroupFinder frame must consume the second supplied lock row')
        "#,
    )
    .expect("selected Blizzard LFG consumers must use supplied queue and lock inputs");
}
