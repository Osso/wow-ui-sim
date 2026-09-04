use crate::common;

use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

fn run_standard_startup(env: &WowLuaEnv, mut after_step: impl FnMut()) {
    common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        env.fire_event(event).ok();
        after_step();
    }
    env.fire_edit_mode_layouts_updated().ok();
    after_step();
    common::call_global_if_present(env, "RequestTimePlayed");
    common::fire_player_entering_world(env, true, false);
    after_step();

    for event in [
        "UNIT_AURA",
        "BAG_UPDATE_DELAYED",
        "QUEST_LOG_UPDATE",
        "GROUP_ROSTER_UPDATE",
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
        "UPDATE_CHAT_WINDOWS",
    ] {
        env.fire_event(event).ok();
        after_step();
    }

    env.fire_on_update(0.016).ok();
    after_step();
}

fn load_and_startup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons(&ui);

    for (_name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path).expect("Failed to load Blizzard addon");
    }

    env.apply_post_load_workarounds();
    run_standard_startup(&env, || {});
    env
}

prefork_full_ui_case! {
fn startup_wardrobe_tab_has_transmog_locations(env: &WowLuaEnv) {
        let result: String = env
            .eval(
                r#"
                local appearanceSlotInfo, illusionSlotInfo = C_TransmogOutfitInfo.GetAllSlotLocationInfo()
                if type(appearanceSlotInfo) ~= "table" or #appearanceSlotInfo == 0 then
                    return "missing_appearance_slot_info"
                end
                if type(illusionSlotInfo) ~= "table" then
                    return "missing_illusion_slot_info"
                end

                local transmogLocation = TransmogUtil.GetTransmogLocation("HEADSLOT", Enum.TransmogType.Appearance, false)
                if not transmogLocation then
                    return "missing_head_transmog_location"
                end
                if transmogLocation:GetSlotName() ~= "HEADSLOT" then
                    return "wrong_slot_name:" .. tostring(transmogLocation:GetSlotName())
                end

                return "ok"
                "#,
            )
            .expect("wardrobe transmog location probe should run");

        assert_eq!(result, "ok");
}
}

prefork_full_ui_case! {
fn startup_wardrobe_can_switch_from_armor_to_weapon_slot(env: &WowLuaEnv) {
        let result: String = env
            .eval(
                r#"
                ToggleCollectionsJournal(5)
                if CollectionsJournal and CollectionsJournal_SetTab then
                    CollectionsJournal_SetTab(CollectionsJournal, 5)
                end

                local itemsFrame = WardrobeCollectionFrame and WardrobeCollectionFrame.ItemsCollectionFrame
                if not itemsFrame then
                    return "missing_items_frame"
                end

                local headLocation = TransmogUtil.GetTransmogLocation("HEADSLOT", Enum.TransmogType.Appearance, false)
                local mainHandLocation = TransmogUtil.GetTransmogLocation("MAINHANDSLOT", Enum.TransmogType.Appearance, false)
                if not headLocation or not mainHandLocation then
                    return "missing_location"
                end
                if mainHandLocation:GetArmorCategoryID() ~= nil then
                    return "weapon_has_armor_category"
                end

                local headOk, headErr = pcall(function()
                    itemsFrame:SetActiveSlot(headLocation)
                end)
                if not headOk then
                    return "head_error:" .. tostring(headErr)
                end

                local weaponOk, weaponErr = pcall(function()
                    itemsFrame:SetActiveSlot(mainHandLocation)
                end)
                if not weaponOk then
                    return "weapon_error:" .. tostring(weaponErr)
                end

                return "ok"
                "#,
            )
            .expect("wardrobe armor-to-weapon slot switch probe should run");

        assert_eq!(result, "ok");
}
}

prefork_full_ui_case! {
fn startup_wardrobe_head_appearances_are_displayable(env: &WowLuaEnv) {
        let result: String = env
            .eval(
                r#"
                ToggleCollectionsJournal(5)
                if CollectionsJournal and CollectionsJournal_SetTab then
                    CollectionsJournal_SetTab(CollectionsJournal, 5)
                end

                local itemsFrame = WardrobeCollectionFrame and WardrobeCollectionFrame.ItemsCollectionFrame
                if not itemsFrame then
                    return "missing_items_frame"
                end

                local headLocation = TransmogUtil.GetTransmogLocation("HEADSLOT", Enum.TransmogType.Appearance, false)
                if not headLocation then
                    return "missing_head_location"
                end

                itemsFrame:SetActiveSlot(headLocation)
                local model = itemsFrame.Models and itemsFrame.Models[1]
                if not model or not model.visualInfo then
                    return "missing_visual_info"
                end
                if not model.visualInfo.canDisplayOnPlayer then
                    return "not_displayable"
                end
                if model.SlotInvalidTexture:IsShown() then
                    return "invalid_overlay_shown"
                end

                return "ok"
                "#,
            )
            .expect("wardrobe displayability probe should run");

        assert_eq!(result, "ok");
}
}

prefork_full_ui_case! {
fn startup_wardrobe_filter_dropdown_click_toggles_not_collected(env: &WowLuaEnv) {
        let setup_result: String = env
            .eval(
                r#"
                ToggleCollectionsJournal(5)
                if CollectionsJournal and CollectionsJournal_SetTab then
                    CollectionsJournal_SetTab(CollectionsJournal, 5)
                end

                local wardrobeFrame = WardrobeCollectionFrame
                local itemsFrame = wardrobeFrame and wardrobeFrame.ItemsCollectionFrame
                local filterButton = wardrobeFrame and wardrobeFrame.FilterButton
                if not itemsFrame or not filterButton then
                    return "missing_wardrobe_filter"
                end

                local headLocation = TransmogUtil.GetTransmogLocation("HEADSLOT", Enum.TransmogType.Appearance, false)
                if not headLocation then
                    return "missing_head_location"
                end
                itemsFrame:SetActiveSlot(headLocation)

                if not filterButton:IsEnabled() then
                    return "filter_disabled"
                end
                return "ready"
                "#,
            )
            .expect("wardrobe filter dropdown probe should run");

        assert_eq!(setup_result, "ready");

        let state = env.state();
        let filter_button_id = {
            let sim = state.borrow();
            let wardrobe_id = sim
                .widgets
                .get_id_by_name("WardrobeCollectionFrame")
                .expect("WardrobeCollectionFrame should exist");
            sim.widgets
                .get(wardrobe_id)
                .and_then(|frame| frame.children_keys.get("FilterButton"))
                .copied()
                .expect("WardrobeCollectionFrame FilterButton should exist")
        };
        let left_button = env.lua_string("LeftButton");
        env.fire_script_handler(filter_button_id, "OnMouseDown", vec![left_button])
            .expect("wardrobe filter OnMouseDown should dispatch");

        let result: String = env
            .eval(
                r#"
                local filterButton = WardrobeCollectionFrame.FilterButton
                if not filterButton:IsMenuOpen() then
                    return "menu_not_open"
                end

                local notCollectedButton
                local function inspectButton(button)
                    local text = button:GetText()
                    if (text == nil or text == "") and button.fontString then
                        text = button.fontString:GetText()
                    end
                    if text == NOT_COLLECTED then
                        notCollectedButton = button
                    end
                end

                for _, button in ipairs(filterButton.__wow_menu_buttons or {}) do
                    inspectButton(button)
                end
                if not notCollectedButton and filterButton.menu then
                    for _, child in ipairs({ filterButton.menu:GetChildren() }) do
                        if child.GetText then
                            inspectButton(child)
                        end
                    end
                end
                if not notCollectedButton then
                    return "missing_not_collected_button"
                end

                C_TransmogCollection.SetUncollectedShown(true)
                notCollectedButton:Click()
                if C_TransmogCollection.GetUncollectedShown() then
                    return "not_collected_still_enabled"
                end

                return "ok"
                "#,
            )
            .expect("wardrobe filter dropdown probe should run");

        assert_eq!(result, "ok");
}
}

prefork_full_ui_case! {
fn startup_wardrobe_class_dropdown_uses_localized_radio_rows(env: &WowLuaEnv) {
        let result: String = env
            .eval(
                r#"
                ToggleCollectionsJournal(5)
                if CollectionsJournal and CollectionsJournal_SetTab then
                    CollectionsJournal_SetTab(CollectionsJournal, 5)
                end

                local dropdown = WardrobeCollectionFrame and WardrobeCollectionFrame.ClassDropdown
                if not dropdown then
                    return "missing_class_dropdown"
                end

                dropdown:Show()
                dropdown:Refresh()
                dropdown:OpenMenu()

                local selectedText = dropdown.Text and dropdown.Text:GetText() or dropdown:GetText()
                if type(selectedText) ~= "string" or not selectedText:find("|c") then
                    return "selected_not_colored:" .. tostring(selectedText)
                end
                if selectedText:find("PALADIN", 1, true) then
                    return "selected_uppercase:" .. selectedText
                end
                local paladinColor = GetClassColorObj("PALADIN")
                local r, g, b = dropdown.Text:GetTextColor()
                if math.abs(r - paladinColor.r) > 0.01
                    or math.abs(g - paladinColor.g) > 0.01
                    or math.abs(b - paladinColor.b) > 0.01 then
                    return "selected_color=" .. tostring(r) .. "," .. tostring(g) .. "," .. tostring(b)
                end

                local firstButton
                local function inspectButton(button)
                    local text = button:GetText()
                    if (text == nil or text == "") and button.fontString then
                        text = button.fontString:GetText()
                    end
                    if text == "Warrior" then
                        firstButton = button
                    end
                end

                for _, button in ipairs(dropdown.__wow_menu_buttons or {}) do
                    inspectButton(button)
                end
                if not firstButton and dropdown.menu then
                    for _, child in ipairs({ dropdown.menu:GetChildren() }) do
                        if child.GetText then
                            inspectButton(child)
                        end
                    end
                end
                if not firstButton then
                    return "missing_warrior_button"
                end
                if not firstButton.leftTexture1 then
                    return "missing_radio_texture"
                end

                return "ok"
                "#,
            )
            .expect("wardrobe class dropdown probe should run");

        assert_eq!(result, "ok");
}
}

prefork_full_ui_case! {
fn startup_collections_journal_closes_on_escape(env: &WowLuaEnv) {

        env.exec("ToggleCollectionsJournal(COLLECTIONS_JOURNAL_TAB_INDEX_APPEARANCES)")
            .expect("collections journal should open");
        let opened: bool = env
            .eval("return CollectionsJournal and CollectionsJournal:IsShown()")
            .expect("collections journal open state should be readable");
        assert!(opened, "CollectionsJournal should be shown before Escape");

        env.send_key_press("ESCAPE", None)
            .expect("Escape dispatch should succeed");
        let closed_without_menu: bool = env
            .eval(
                "return (not CollectionsJournal:IsShown()) \
                    and (GameMenuFrame == nil or not GameMenuFrame:IsShown())",
            )
            .expect("collections journal closed state should be readable");
        assert!(
            closed_without_menu,
            "Escape should close CollectionsJournal through the UIPanel close path"
        );
}
}

prefork_full_ui_case! {
fn startup_adventure_guide_closes_on_escape(env: &WowLuaEnv) {

        env.exec("ToggleEncounterJournal()")
            .expect("adventure guide should open");
        let opened: bool = env
            .eval("return EncounterJournal and EncounterJournal:IsShown()")
            .expect("adventure guide open state should be readable");
        assert!(opened, "EncounterJournal should be shown before Escape");

        env.send_key_press("ESCAPE", None)
            .expect("Escape dispatch should succeed");
        let closed_without_menu: bool = env
            .eval(
                "return (not EncounterJournal:IsShown()) \
                    and (GameMenuFrame == nil or not GameMenuFrame:IsShown())",
            )
            .expect("adventure guide closed state should be readable");
        assert!(
            closed_without_menu,
            "Escape should close EncounterJournal through the UIPanel close path"
        );
}
}
