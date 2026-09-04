//! Integration tests for ShowUIPanel toggle and interaction behaviors.
//!
//! Tests panel displacement, coexistence, and toggling via Blizzard global functions.

use crate::common;

use std::path::PathBuf;
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

/// Load PlayerSpells through the runtime API so declared dependencies and
/// `ADDON_LOADED` behavior match the client.
fn load_player_spells(env: &WowLuaEnv) {
    let (loaded, reason): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_PlayerSpells")"#)
        .expect("Blizzard_PlayerSpells load should return");
    assert!(loaded, "Blizzard_PlayerSpells should load: {reason:?}");
}

/// Eager panel publishers in dependency order. `load_required_blizzard_addon`
/// resolves the active profile TOC and fails immediately for missing/invalid loads.
const PANEL_ADDONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_Colors",
    "Blizzard_SharedXML",
    "Blizzard_Menu",
    "Blizzard_SharedXMLGame",
    "Blizzard_UIPanelTemplates",
    "Blizzard_FrameXMLBase",
    "Blizzard_FrameEffects",
    "Blizzard_LoadLocale",
    "Blizzard_Fonts_Shared",
    "Blizzard_HelpPlate",
    "Blizzard_AccessibilityTemplates",
    "Blizzard_ObjectAPI",
    "Blizzard_UIParent",
    "Blizzard_TextStatusBar",
    "Blizzard_MoneyFrame",
    "Blizzard_POIButton",
    "Blizzard_Flyout",
    "Blizzard_GameMenuEsc",
    "Blizzard_UIParentUtil",
    "Blizzard_Game",
    "Blizzard_ChatFrameBase",
    "Blizzard_ChatFrame",
    "Blizzard_GuildControlUI",
    "Blizzard_TimerunningUtil",
    "Blizzard_FrameXMLUtil",
    "Blizzard_Communities",
    "Blizzard_StoreUI",
    "Blizzard_MicroMenu",
    "Blizzard_ManagedFrameSystem",
    "Blizzard_EditMode",
    "Blizzard_GarrisonBase",
    "Blizzard_GameTooltip",
    "Blizzard_StaticPopup_Game",
    "Blizzard_TransmogShared",
    "Blizzard_UIParentPanelManager",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    "Blizzard_FrameXML",
    "Blizzard_UIPanels_Game",
    "Blizzard_ActionBar",
    "Blizzard_UnitFrame",
    "Blizzard_TokenUI",
    "Blizzard_HousingTemplates",
    "Blizzard_HousingModelPreview",
    "Blizzard_HousingBlueprint",
];

fn setup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    let ui = blizzard_ui_dir();
    for addon_name in PANEL_ADDONS {
        common::load_required_blizzard_addon(&env, &ui, addon_name);
    }
    load_lod_bootstrap(&env, &ui, "Blizzard_Collections");
    load_lod_bootstrap(&env, &ui, "Blizzard_Professions");

    env.apply_post_load_workarounds();
    env.exec(r#"CHARACTERFRAME_SUBFRAMES = { "PaperDollFrame", "ReputationFrame", "TokenFrame" }"#)
        .expect("character subframe fixture should restore cleanup-pruned constant");
    fire_startup_events(&env);
    env
}

fn load_lod_bootstrap(env: &WowLuaEnv, ui: &std::path::Path, addon_name: &str) {
    let path = ui
        .join(addon_name)
        .join(format!("{addon_name}_Bootstrap.lua"));
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let source = source.replacen(
        "local AddonName = ...;",
        &format!("local AddonName = {addon_name:?};"),
        1,
    );
    env.exec(&source)
        .unwrap_or_else(|error| panic!("failed to load {}: {error}", path.display()));
}

fn fire_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    common::fire_player_entering_world(env, true, false);
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }
}

#[test]
fn show_ui_panel_displaces_previous_occupant() {
    test_timeout! {
        let env = setup_env();
        // UIParentPanelManager manages left/center/right slots. When two panels both
        // have pushable=0 and a left slot is occupied, the new panel replaces the old.
        // CharacterFrame (pushable=3) gets pushed to center instead of replaced.
        //
        // Test 1: Pushable panel gets pushed to center (not closed)
        // CharacterFrame (pushable=3) in left, then FriendsFrame (pushable=0) opens
        // → CharacterFrame pushed to center, both visible
        let result: String = env.eval(r#"
            if not CharacterFrame or not FriendsFrame then
                return "missing_frames"
            end
            ShowUIPanel(CharacterFrame)
            if not CharacterFrame:IsShown() then return "char_not_shown" end
            ShowUIPanel(FriendsFrame)
            if not FriendsFrame:IsShown() then return "friends_not_shown" end
            -- CharacterFrame should be pushed to center, still visible
            if not CharacterFrame:IsShown() then return "char_should_be_pushed_not_closed" end
            return "ok"
        "#).unwrap();
        assert_eq!(result, "ok", "Pushable panel should be pushed to center, not closed: {result}");

        // Test 2: Non-pushable panel replaces another non-pushable panel
        // When both panels have pushable=0, the old one is replaced (hidden).
        let result: String = env.eval(r#"
            -- Close everything first
            CloseAllWindows()
            -- Register two test panels with pushable=0
            local a = CreateFrame("Frame", "TestPanelA", UIParent)
            a:SetSize(300, 400)
            a:Hide()
            UIPanelWindows["TestPanelA"] = { area = "left", pushable = 0, whileDead = 1 }
            local b = CreateFrame("Frame", "TestPanelB", UIParent)
            b:SetSize(300, 400)
            b:Hide()
            UIPanelWindows["TestPanelB"] = { area = "left", pushable = 0, whileDead = 1 }

            ShowUIPanel(a)
            if not a:IsShown() then return "a_not_shown" end
            ShowUIPanel(b)
            if not b:IsShown() then return "b_not_shown" end
            if a:IsShown() then return "a_not_replaced" end
            return "ok"
        "#).unwrap();
        assert_eq!(result, "ok", "Non-pushable panel should replace previous non-pushable occupant: {result}");
    }
}

#[test]
fn player_spells_panel_replaces_character_frame() {
    test_timeout! {
        let env = setup_env();
        load_player_spells(&env);

        // Retail 12.1.0.69497 closes CharacterFrame when PlayerSpells opens.
        let result: String = env.eval(r#"
            if not CharacterFrame then return "no_char_frame" end
            if not PlayerSpellsFrame then return "no_spellbook_frame" end

            ShowUIPanel(CharacterFrame)
            if not CharacterFrame:IsShown() then return "char_not_shown" end

            ShowUIPanel(PlayerSpellsFrame)
            if not PlayerSpellsFrame:IsShown() then return "spellbook_not_shown" end
            if CharacterFrame:IsShown() then return "char_not_closed" end
            return "ok"
        "#).unwrap();
        assert_eq!(result, "ok", "PlayerSpells should replace CharacterFrame: {result}");
    }
}

#[test]
fn toggle_spellbook_legacy_global_opens_and_closes_spellbook_panel() {
    test_timeout! {
        let env = setup_env();
        load_player_spells(&env);

        let result: String = env.eval(r#"
            if not ToggleSpellBook then
                return "missing_toggle_spellbook"
            end

            ToggleSpellBook(BOOKTYPE_SPELL)
            if not PlayerSpellsFrame or not PlayerSpellsFrame:IsShown() then
                return "spellbook_not_shown"
            end
            if not PlayerSpellsFrame.SpellBookFrame or not PlayerSpellsFrame.SpellBookFrame:IsShown() then
                return "spellbook_tab_not_shown"
            end

            ToggleSpellBook(BOOKTYPE_SPELL)
            if PlayerSpellsFrame:IsShown() then
                return "spellbook_not_hidden"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "ToggleSpellBook(BOOKTYPE_SPELL) should toggle the spellbook panel: {result}"
        );
    }
}

#[test]
fn toggle_player_spells_frame_opens_and_closes_talent_panel() {
    test_timeout! {
        let env = setup_env();
        load_player_spells(&env);

        let result: String = env.eval(r#"
            if not PlayerSpellsUtil or not PlayerSpellsUtil.TogglePlayerSpellsFrame then
                return "missing_toggle_player_spells_frame"
            end

            PlayerSpellsUtil.TogglePlayerSpellsFrame()
            if not PlayerSpellsFrame or not PlayerSpellsFrame:IsShown() then
                return "player_spells_not_shown"
            end

            PlayerSpellsUtil.TogglePlayerSpellsFrame()
            if PlayerSpellsFrame:IsShown() then
                return "player_spells_not_hidden"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "PlayerSpellsUtil.TogglePlayerSpellsFrame() should toggle the talent panel: {result}"
        );
    }
}

#[test]
fn toggle_collections_journal_opens_mounts_pets_and_toys_tabs_and_accepts_search_text() {
    test_timeout! {
        let env = setup_env();

        let result: String = env.eval(r#"
            if not ToggleCollectionsJournal then
                return "missing_toggle_collections_journal"
            end

            local cases = {
                { COLLECTIONS_JOURNAL_TAB_INDEX_MOUNTS, "MountJournal", "searchBox", "gryphon" },
                { COLLECTIONS_JOURNAL_TAB_INDEX_PETS, "PetJournal", "searchBox", "cat" },
                { COLLECTIONS_JOURNAL_TAB_INDEX_TOYS, "ToyBox", "searchBox", "ball" },
            }

            for _, case in ipairs(cases) do
                local tabIndex, childName, searchKey, searchText = case[1], case[2], case[3], case[4]
                ToggleCollectionsJournal(tabIndex)

                if not CollectionsJournal or not CollectionsJournal:IsShown() then
                    return "journal_not_shown_" .. tostring(tabIndex)
                end
                if CollectionsJournal_GetTab(CollectionsJournal) ~= tabIndex then
                    return "wrong_tab_" .. tostring(tabIndex) .. "_" .. tostring(CollectionsJournal_GetTab(CollectionsJournal))
                end

                local child = _G[childName]
                if not child or not child:IsShown() then
                    return "child_not_shown_" .. childName
                end
                local searchBox = child[searchKey]
                if not searchBox then
                    return "search_box_missing_" .. childName
                end
                searchBox:SetText(searchText)
                if searchBox:GetText() ~= searchText then
                    return "search_text_not_set_" .. childName
                end

                ToggleCollectionsJournal(tabIndex)
                if CollectionsJournal:IsShown() then
                    return "journal_not_hidden_" .. tostring(tabIndex)
                end
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "ToggleCollectionsJournal(tab) should open, switch tabs, accept search text, and close for mounts, pets, and toys: {result}"
        );
    }
}

#[test]
fn collections_mount_list_count_matches_displayed_mount_count() {
    test_timeout! {
        let env = setup_env();

        let result: String = env.eval(r#"
            if not ToggleCollectionsJournal then
                return "missing_toggle_collections_journal"
            end

            ToggleCollectionsJournal(COLLECTIONS_JOURNAL_TAB_INDEX_MOUNTS)

            if not (CollectionsJournal and CollectionsJournal:IsShown()) then
                return "journal_not_shown"
            end
            if not (MountJournal and MountJournal:IsShown()) then
                return "mount_journal_not_shown"
            end
            if not MountJournal.ScrollBox then
                return "missing_mount_scroll_box"
            end

            local dataProvider = MountJournal.ScrollBox:GetDataProvider()
            if not dataProvider then
                return "missing_mount_data_provider"
            end

            local expected = C_MountJournal.GetNumDisplayedMounts()
            local actual = dataProvider:GetSize()
            if actual ~= expected then
                return string.format(
                    "mount_list_count_mismatch_expected_%s_actual_%s",
                    tostring(expected),
                    tostring(actual)
                )
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "MountJournal scrollbox data-provider size should match C_MountJournal.GetNumDisplayedMounts(): {result}"
        );
    }
}

#[test]
fn toggle_achievement_frame_opens_and_closes_achievement_panel() {
    test_timeout! {
        let env = setup_env();

        let result: String = env.eval(r#"
            if not ToggleAchievementFrame then
                return "missing_toggle_achievement_frame"
            end

            ToggleAchievementFrame()
            if not AchievementFrame or not AchievementFrame:IsShown() then
                return "achievement_not_shown"
            end

            ToggleAchievementFrame()
            if AchievementFrame:IsShown() then
                return "achievement_not_hidden"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "ToggleAchievementFrame() should open and close the achievement panel: {result}"
        );
    }
}

#[test]
fn toggle_encounter_journal_opens_and_closes_panel() {
    test_timeout! {
        let env = setup_env();

        let result: String = env.eval(r#"
            if not ToggleEncounterJournal then
                return "missing_toggle_encounter_journal"
            end

            ToggleEncounterJournal()
            if not EncounterJournal or not EncounterJournal:IsShown() then
                return "encounter_journal_not_shown"
            end

            ToggleEncounterJournal()
            if EncounterJournal:IsShown() then
                return "encounter_journal_not_hidden"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "ToggleEncounterJournal() should open and close the encounter journal panel: {result}"
        );
    }
}

#[test]
fn open_trade_skill_opens_blacksmithing_panel() {
    test_timeout! {
        let env = setup_env();

        let result: String = env.eval(r#"
            if not C_TradeSkillUI or not C_TradeSkillUI.OpenTradeSkill then
                return "missing_open_trade_skill"
            end

            local opened = C_TradeSkillUI.OpenTradeSkill(164)
            if opened ~= true then
                return "opened=" .. tostring(opened)
            end

            if not ProfessionsFrame or not ProfessionsFrame:IsShown() then
                return "professions_frame_not_shown"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "C_TradeSkillUI.OpenTradeSkill(164) should open the professions frame: {result}"
        );
    }
}

#[test]
fn toggle_guild_frame_opens_and_closes_communities_panel() {
    test_timeout! {
        let env = setup_env();
        let (loaded, reason): (bool, Option<String>) = env
            .eval(r#"return C_AddOns.LoadAddOn("Blizzard_Communities")"#)
            .expect("Blizzard_Communities load should return");
        assert!(loaded, "Blizzard_Communities should load: {reason:?}");

        let result: String = env.eval(r#"
            if not ToggleGuildFrame then
                return "missing_toggle_guild_frame"
            end

            local opened = ToggleGuildFrame()
            if CommunitiesFrame and not CommunitiesFrame:IsShown() then
                return "communities_frame_not_shown"
            end

            ToggleGuildFrame()
            if CommunitiesFrame and CommunitiesFrame:IsShown() then
                return "communities_frame_not_hidden"
            end

            return tostring(opened == nil or opened == true) == "true" and "ok" or ("opened=" .. tostring(opened))
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "ToggleGuildFrame() should open and close the guild/communities panel: {result}"
        );
    }
}

#[test]
fn toggle_lfd_parent_frame_opens_and_closes_group_finder_panel() {
    test_timeout! {
        let env = setup_env();

        let result: String = env.eval(r#"
            local loaded, reason = C_AddOns.LoadAddOn("Blizzard_GroupFinder")
            if not loaded then
                return "group_finder_load_failed:" .. tostring(reason)
            end

            if not ToggleLFDParentFrame then
                return "missing_toggle_lfd_parent_frame"
            end

            ToggleLFDParentFrame()
            if not PVEFrame or not PVEFrame:IsShown() then
                return "pve_frame_not_shown"
            end

            ToggleLFDParentFrame()
            if PVEFrame and PVEFrame:IsShown() then
                return "pve_frame_not_hidden"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "ToggleLFDParentFrame() should open and close the group finder panel: {result}"
        );
    }
}

#[test]
fn toggle_character_reputation_frame_selects_and_toggles_reputation_panel() {
    test_timeout! {
        let env = setup_env();

        let result: String = env.eval(r#"
            CHARACTERFRAME_SUBFRAMES = { "PaperDollFrame", "ReputationFrame", "TokenFrame" }
            if not ToggleCharacter then
                return "missing_toggle_character"
            end
            if not ReputationFrame then
                return "missing_reputation_frame"
            end

            ToggleCharacter("ReputationFrame")
            if not CharacterFrame or not CharacterFrame:IsShown() then
                return "character_frame_not_shown"
            end
            if not ReputationFrame:IsShown() then
                return "reputation_frame_not_shown"
            end
            if PaperDollFrame and PaperDollFrame:IsShown() then
                return "paperdoll_should_be_hidden"
            end

            ToggleCharacter("ReputationFrame")
            if CharacterFrame and CharacterFrame:IsShown() then
                return "character_frame_not_hidden"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "ToggleCharacter(\"ReputationFrame\") should select and toggle the reputation panel: {result}"
        );
    }
}

#[test]
fn character_reputation_tab_click_selects_reputation_panel() {
    test_timeout! {
        let env = setup_env();

        let result: String = env.eval(r#"
            CHARACTERFRAME_SUBFRAMES = { "PaperDollFrame", "ReputationFrame", "TokenFrame" }
            ToggleCharacter("PaperDollFrame", true)
            if not CharacterFrameTab2 then
                return "missing_reputation_tab"
            end

            local onClick = CharacterFrameTab2:GetScript("OnClick")
            if type(onClick) ~= "function" then
                return "missing_onclick"
            end

            onClick(CharacterFrameTab2, "LeftButton")
            if not CharacterFrame or not CharacterFrame:IsShown() then
                return "character_frame_not_shown"
            end
            if not ReputationFrame or not ReputationFrame:IsShown() then
                return "reputation_frame_not_shown"
            end
            if PaperDollFrame and PaperDollFrame:IsShown() then
                return "paperdoll_should_be_hidden"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "CharacterFrameTab2 OnClick should select the reputation panel: {result}"
        );
    }
}

#[test]
fn admin_open_mailbox_opens_and_closes_mail_panel() {
    test_timeout! {
        let env = setup_env();

        let result: String = env.eval(r#"
            A_Admin.ClearInbox()
            A_Admin.AddMail("Thrall", "Unread Orders", "Meet me in Orgrimmar.")

            local loaded, reason = C_AddOns.LoadAddOn("Blizzard_MailFrame")
            if not loaded then
                return "mail_load_failed:" .. tostring(reason)
            end

            if not A_Admin.OpenMailbox or not A_Admin.CloseMailbox then
                return "missing_mailbox_admin_api"
            end

            A_Admin.OpenMailbox()
            if not MailFrame or not MailFrame:IsShown() then
                return "mail_frame_not_shown"
            end
            if not InboxFrame or not InboxFrame:IsShown() then
                return "inbox_frame_not_shown"
            end

            A_Admin.CloseMailbox()
            if MailFrame and MailFrame:IsShown() then
                return "mail_frame_not_hidden"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "A_Admin mailbox interaction should open and close the mail panel: {result}"
        );
    }
}

#[test]
fn open_all_bags_opens_a_bag_frame() {
    test_timeout! {
        let env = setup_env();

        let result: String = env.eval(r#"
            if not OpenAllBags then
                return "missing_open_all_bags"
            end

            OpenAllBags()

            if ContainerFrameCombinedBags and ContainerFrameCombinedBags:IsShown() then
                return "ok"
            end

            for i = 1, 6 do
                local frame = _G["ContainerFrame" .. i]
                if frame and frame:IsShown() then
                    return "ok"
                end
            end

            return "no_bag_frame_shown"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "OpenAllBags() should show a combined or individual bag frame: {result}"
        );
    }
}

#[test]
fn housing_dashboard_loads_and_opens_panel() {
    test_timeout! {
        let env = setup_env();
        let result: String = env
            .eval(
                r#"
                    local loaded, reason = C_AddOns.LoadAddOn("Blizzard_HousingDashboard")
                    if not loaded then
                        return "load_failed:" .. tostring(reason)
                    end
                    if not HousingDashboardFrame then
                        return "missing_frame"
                    end

                    local panelEntry = UIPanelWindows["HousingDashboardFrame"]
                    if not panelEntry then
                        return "missing_panel_registration"
                    end

                    local ok, err = pcall(function()
                        ShowUIPanel(HousingDashboardFrame)
                    end)
                    if not ok then
                        return "show_failed:" .. tostring(err)
                    end
                    return HousingDashboardFrame:IsShown() and "shown" or "panel_not_shown"
                "#,
            )
            .unwrap();
        assert_eq!(
            result, "shown",
            "Housing dashboard should load and open via ShowUIPanel: {result}"
        );

        env.process_timers()
            .expect("owned house response timer should process");
        let result: String = env
            .eval(
                r#"
                    if HousingDashboardFrame.HouseInfoContent.LoadingSpinner:IsShown() then
                        return "spinner_still_shown"
                    end
                    return HousingDashboardFrame.HouseInfoContent.DashboardNoHousesFrame:IsShown()
                        and "ok" or "no_houses_dashboard_not_shown"
                "#,
            )
            .unwrap();
        assert_eq!(
            result, "ok",
            "Housing dashboard should show its empty state after the owned-house response: {result}"
        );
    }
}
