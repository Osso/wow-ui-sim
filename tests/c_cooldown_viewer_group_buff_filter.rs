#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::loader::load_addon;

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

const SETTINGS_GROUP_BUFF_UI_TEST: &str = r#"
    CooldownViewerSettings:TogglePanel()
    local tab = CooldownViewerSettings.GroupBuffsTab
    assert(tab.isTabDisabled and not tab:IsMouseClickEnabled())
    assert(tab.Icon:IsDesaturated() and math.abs(tab.Icon:GetAlpha() - 0.5) < 0.001)
    CooldownViewerSettings:TogglePanel()
    A_Admin.SetGroupBuffItems({
        { spellID = 1459, displaySpellID = 1459, name = 'Arcane Intellect', iconID = 135932, flags = 0, isKnown = true },
        { spellID = 21562, displaySpellID = 21562, name = 'Power Word: Fortitude', iconID = 135987, flags = Enum.GroupBuffItemFlags.HideByDefault, isKnown = true },
    })
    CooldownViewerSettings:TogglePanel()
    assert(not tab.isTabDisabled and tab:IsMouseClickEnabled())
    assert(not tab.Icon:IsDesaturated() and math.abs(tab.Icon:GetAlpha() - 1) < 0.001)
    CooldownViewerSettings:SetDisplayMode('groupBuffs')
    assert(CooldownViewerSettings.GroupBuffFilter:IsShown())
    local function sectionIDs(section)
        local ids = {}
        for item in section.itemPool:EnumerateActive() do
            table.insert(ids, item:GetGroupBuffItem().spellID)
        end
        return ids
    end
    local shown = sectionIDs(CooldownViewerSettings.GroupBuffFilter.shownSection)
    local hidden = sectionIDs(CooldownViewerSettings.GroupBuffFilter.hiddenSection)
    assert(#shown == 1 and shown[1] == 1459, 'shown record mismatch')
    assert(#hidden == 1 and hidden[1] == 21562, 'hidden record mismatch')
    A_Admin.SetGroupBuffItems({
        { spellID = 1459, displaySpellID = 1459, name = 'Arcane Intellect', iconID = 135932, flags = 0, isKnown = true },
    })
    CooldownViewerSettings:RefreshLayout()
    assert(tab.isTabDisabled and not tab:IsMouseClickEnabled())
    assert(tab.Icon:IsDesaturated() and math.abs(tab.Icon:GetAlpha() - 0.5) < 0.001)
    assert(CooldownViewerSettings.displayMode == 'spells')
"#;

fn full_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    );
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn cooldown_viewer_settings_group_buff_tab_tracks_supplied_items() {
    let env = full_game_env();
    env.exec(SETTINGS_GROUP_BUFF_UI_TEST)
        .expect("original settings panel should track supplied group buff items");
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
