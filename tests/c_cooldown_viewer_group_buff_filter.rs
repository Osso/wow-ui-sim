#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn group_buff_consumer_env() -> WowLuaEnv {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    crate::common::load_profile_framexml(&env);
    let loaded = crate::common::blizzard_addon_harness::load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_CooldownViewer"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_CooldownViewer"));
    env
}

const GROUP_BUFF_FILTER_FIXTURE: &str = r#"
    A_Admin.SetGroupBuffItems({
        { spellID = 1459, displaySpellID = 1459, name = 'Arcane Intellect', iconID = 135932, flags = 0, isKnown = true },
        { spellID = 21562, displaySpellID = 21562, name = 'Power Word: Fortitude', iconID = 135987, flags = Enum.GroupBuffItemFlags.HideByDefault, isKnown = true },
    })
    local filter = CreateFrame('Frame', nil, UIParent, 'GroupBuffFilterTemplate')
    filter:Refresh()
    local function ids(section)
        local result = {}
        for item in section.itemPool:EnumerateActive() do
            table.insert(result, item:GetGroupBuffItem().spellID)
        end
        return result
    end
    local shown, hidden = ids(filter.shownSection), ids(filter.hiddenSection)
    assert(#shown == 1 and shown[1] == 1459, 'shown section must contain Arcane Intellect')
    assert(#hidden == 1 and hidden[1] == 21562, 'hidden section must contain Power Word: Fortitude')
"#;

const EMPTY_GROUP_BUFF_TAB_TEST: &str = r#"
    CooldownViewerSettings:TogglePanel()
    local tab = CooldownViewerSettings.GroupBuffsTab
    assert(tab.isTabDisabled and not tab:IsMouseClickEnabled())
    assert(tab.Icon:IsDesaturated() and math.abs(tab.Icon:GetAlpha() - 0.5) < 0.001)
    CooldownViewerSettings:TogglePanel()
"#;

const TWO_GROUP_BUFF_TAB_TEST: &str = r#"
    A_Admin.SetGroupBuffItems({
        { spellID = 1459, displaySpellID = 1459, name = 'Arcane Intellect', iconID = 135932, flags = 0, isKnown = true },
        { spellID = 21562, displaySpellID = 21562, name = 'Power Word: Fortitude', iconID = 135987, flags = Enum.GroupBuffItemFlags.HideByDefault, isKnown = true },
    })
    CooldownViewerSettings:TogglePanel()
    local tab = CooldownViewerSettings.GroupBuffsTab
    assert(not tab.isTabDisabled and tab:IsMouseClickEnabled())
    assert(not tab.Icon:IsDesaturated() and math.abs(tab.Icon:GetAlpha() - 1) < 0.001)
"#;

const ONE_GROUP_BUFF_TAB_TEST: &str = r#"
    A_Admin.SetGroupBuffItems({
        { spellID = 1459, displaySpellID = 1459, name = 'Arcane Intellect', iconID = 135932, flags = 0, isKnown = true },
    })
    CooldownViewerSettings:TogglePanel()
    CooldownViewerSettings:TogglePanel()
    local tab = CooldownViewerSettings.GroupBuffsTab
    assert(tab.isTabDisabled and not tab:IsMouseClickEnabled())
    assert(tab.Icon:IsDesaturated() and math.abs(tab.Icon:GetAlpha() - 0.5) < 0.001)
    assert(CooldownViewerSettings.displayMode == 'spells')
"#;

fn group_buff_tab_frame_id(env: &WowLuaEnv) -> u64 {
    let state = env.state().borrow();
    let settings_id = state
        .widgets
        .get_id_by_name("CooldownViewerSettings")
        .expect("settings panel should exist");
    *state
        .widgets
        .get(settings_id)
        .and_then(|frame| frame.children_keys.get("GroupBuffsTab"))
        .expect("settings tab should be a keyed child")
}

fn group_buff_settings_env() -> WowLuaEnv {
    let env = crate::common::panel_fixtures::setup_env();
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let loaded = crate::common::blizzard_addon_harness::load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_CooldownViewer"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_CooldownViewer"));
    env
}

#[test]
fn cooldown_viewer_settings_group_buff_tab_tracks_supplied_items() {
    let env = group_buff_settings_env();
    env.exec(EMPTY_GROUP_BUFF_TAB_TEST).expect("empty state should disable the tab");
    env.exec(TWO_GROUP_BUFF_TAB_TEST)
        .expect("two supplied records should enable the tab");
    env.send_click(group_buff_tab_frame_id(&env))
        .expect("simulated click should run original tab scripts");
    let (filter_shown, shown, hidden): (bool, Vec<i32>, Vec<i32>) = env
        .eval(
            r#"
            local function sectionIDs(section)
                local ids = {}
                for item in section.itemPool:EnumerateActive() do
                    table.insert(ids, item:GetGroupBuffItem().spellID)
                end
                return ids
            end
            local filter = CooldownViewerSettings.GroupBuffFilter
            return filter:IsShown(), sectionIDs(filter.shownSection), sectionIDs(filter.hiddenSection)
            "#,
        )
        .expect("tab click should show the group buff filter");
    assert!(filter_shown);
    assert_eq!(shown, [1459]);
    assert_eq!(hidden, [21562]);
    env.exec(ONE_GROUP_BUFF_TAB_TEST)
        .expect("reopening after one item should disable the tab");
}

#[test]
fn original_group_buff_filter_partitions_supplied_items_by_default_hidden_flag() {
    let env = group_buff_consumer_env();
    env.exec(GROUP_BUFF_FILTER_FIXTURE)
        .expect("original Blizzard GroupBuffFilter consumer must partition supplied records");
    #[cfg(feature = "client-retail")]
    assert!(
        env.eval::<bool>("return C_CooldownViewer.GetGroupBuffItems()[1].displaySpellID == nil")
            .unwrap()
    );
    #[cfg(feature = "client-wowforever")]
    assert_eq!(
        env.eval::<i32>("return C_CooldownViewer.GetGroupBuffItems()[1].displaySpellID")
            .unwrap(),
        1459
    );
}
