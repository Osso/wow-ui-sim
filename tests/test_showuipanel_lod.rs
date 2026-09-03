//! Integration tests for Load-on-Demand panel loading via ShowUIPanel.
//!
//! Tests that LoD addons load correctly and populate their UI state when
//! triggered through the panel system or explicit LoadAddOn calls.
//!
//! Player-Spells-specific keybind/debug tests live in
//! `test_showuipanel_lod_player_spells.rs`; the harness/fixture coverage
//! tests (ActionButtonUtil overrides, UIParentLoadAddOn seam, opt-in
//! CooldownBroadcaster) live in `test_showuipanel_lod_fixtures.rs`. All
//! three binaries share `tests/common/panel_fixtures.rs`.

use crate::common;

use common::panel_fixtures::{blizzard_ui_dir, setup_env};
use wow_ui_sim::loader::load_addon;

#[test]
fn show_macro_frame_loads_and_populates_selector() {
    test_timeout! {
        let env = setup_env();
        env.exec(
            "Constants.MacroConsts = { MAX_ACCOUNT_MACROS = 120, MAX_CHARACTER_MACROS = 30 }",
        )
        .expect("failed to seed retail MacroConsts");
        let result: String = env.eval(r#"
            if not ShowMacroFrame then
                return "missing_show_macro_frame"
            end

            ShowMacroFrame()

            if not MacroFrame or not MacroFrame:IsShown() then
                return "macro_frame_not_shown"
            end
            if PanelTemplates_GetSelectedTab(MacroFrame) ~= 1 then
                return "selected_tab=" .. tostring(PanelTemplates_GetSelectedTab(MacroFrame))
            end
            if MacroFrame.MacroSelector.numMacros ~= 2 then
                return "selector_count=" .. tostring(MacroFrame.MacroSelector.numMacros)
            end
            if MacroFrameSelectedMacroName:GetText() ~= "Raid Beacon" then
                return "selected_name=" .. tostring(MacroFrameSelectedMacroName:GetText())
            end
            if MacroFrameText:GetText() ~= "/rw Stack on star" then
                return "selected_body=" .. tostring(MacroFrameText:GetText())
            end
            if MacroFrame.SelectedMacroButton.Icon:GetTexture() ~= 134400 then
                return "selected_icon=" .. tostring(MacroFrame.SelectedMacroButton.Icon:GetTexture())
            end

            local accountCount, characterCount = GetNumMacros()
            if accountCount ~= 2 or characterCount ~= 1 then
                return "counts=" .. tostring(accountCount) .. "," .. tostring(characterCount)
            end
            return "ok"
        "#).unwrap();
        assert_eq!(result, "ok", "ShowMacroFrame should load and populate the macro selector: {result}");
    }
}

#[test]
fn show_mail_frame_loads_and_populates_inbox_rows() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            A_Admin.ClearInbox()
            A_Admin.AddMail("Thrall", "Unread Orders", "Meet me in Orgrimmar.")
            A_Admin.AddMail("Jaina", "Arcane Invoice", "The Kirin Tor still expects payment.")

            local loaded, reason = C_AddOns.LoadAddOn("Blizzard_MailFrame")
            if not loaded then
                return "load_failed:" .. tostring(reason)
            end
            if not MailFrame or not MailFrame_Show then
                return "mail_frame_missing"
            end

            MailFrame_Show()

            if not MailFrame:IsShown() then
                return "mail_frame_not_shown"
            end
            if not InboxFrame or not InboxFrame:IsShown() then
                return "inbox_frame_not_shown"
            end
            if C_Mail.GetNumItems() ~= 2 then
                return "c_mail_count=" .. tostring(C_Mail.GetNumItems())
            end

            local firstButton = MailItem1Button
            if not firstButton or not firstButton:IsShown() then
                return "mail_item_1_not_shown"
            end
            if firstButton.index ~= 1 then
                return "mail_item_1_index=" .. tostring(firstButton.index)
            end
            if MailItem1Sender:GetText() ~= "Thrall" then
                return "mail_item_1_sender=" .. tostring(MailItem1Sender:GetText())
            end
            if MailItem1Subject:GetText() ~= "Unread Orders" then
                return "mail_item_1_subject=" .. tostring(MailItem1Subject:GetText())
            end
            if not MailItem2Button or not MailItem2Button:IsShown() then
                return "mail_item_2_not_shown"
            end
            if MailItem2Sender:GetText() ~= "Jaina" then
                return "mail_item_2_sender=" .. tostring(MailItem2Sender:GetText())
            end

            return "ok"
        "#).unwrap();
        assert_eq!(result, "ok", "ShowMailFrame should load the inbox panel and populate seeded inbox rows: {result}");
    }
}

#[test]
fn item_socketing_frame_loads_and_populates_socket_buttons() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            C_ItemSocketInfo._state.uiType = Enum.ItemSocketInfoUIType.ItemSocketingUI or 0
            C_ItemSocketInfo._state.isOpen = true
            C_ItemSocketInfo._state.numSockets = 2
            C_ItemSocketInfo._state.itemInfo = {
                name = "Socketed Helm",
                icon = 901,
                itemID = 6948,
                link = "item:6948",
                quality = 4,
                isRefundable = false,
                isBoundTradeable = true,
            }
            C_ItemSocketInfo._state.socketTypes = {
                [1] = "Red",
                [2] = "Blue",
            }
            C_ItemSocketInfo._state.existingSockets = {
                [1] = {
                    name = "Ruby",
                    icon = 111,
                    itemID = 6948,
                    gemMatchesSocket = true,
                    link = "item:6948",
                },
            }
            C_ItemSocketInfo._state.newSockets = {
                [2] = {
                    name = "Bound Sapphire",
                    icon = 222,
                    itemID = 6948,
                    gemMatchesSocket = false,
                    link = "item:6948",
                    isBound = true,
                },
            }
            C_ItemSocketInfo._state.hasBoundGemProposed = true

            local loaded, reason = C_AddOns.LoadAddOn("Blizzard_ItemSocketingUI")
            if not loaded then
                return "load_failed:" .. tostring(reason)
            end

            if ItemSocketingFrame:GetScript("OnEvent") == nil then
                return "missing_on_event"
            end

            ItemSocketingFrame:GetScript("OnEvent")(ItemSocketingFrame, "SOCKET_INFO_UPDATE")

            if not ItemSocketingFrame:IsShown() then
                return "frame_not_shown"
            end
            if not ItemSocketingFrame.SocketingContainer.Socket1:IsShown() then
                return "socket1_hidden"
            end
            if not ItemSocketingFrame.SocketingContainer.Socket2:IsShown() then
                return "socket2_hidden"
            end
            if ItemSocketingFrame.SocketingContainer.Socket1.Icon:GetTexture() ~= 111 then
                return "socket1_icon=" .. tostring(ItemSocketingFrame.SocketingContainer.Socket1.Icon:GetTexture())
            end
            if ItemSocketingFrame.SocketingContainer.Socket2.Icon:GetTexture() ~= 222 then
                return "socket2_icon=" .. tostring(ItemSocketingFrame.SocketingContainer.Socket2.Icon:GetTexture())
            end
            if not ItemSocketingFrame.SocketingContainer.Socket1.Shine:IsShown() then
                return "socket1_shine_hidden"
            end
            if not ItemSocketingFrame.SocketingContainer.itemIsBoundTradeable then
                return "bound_tradeable_flag_missing"
            end
            if ItemSocketingFrame.SocketingContainer.ApplySocketsButton:IsEnabled() ~= true then
                return "apply_disabled"
            end

            local socket1 = ItemSocketingFrame.SocketingContainer.Socket1
            socket1:GetScript("OnEnter")(socket1)
            if not GameTooltip:IsShown() then
                return "tooltip_not_shown"
            end
            if not GameTooltip:IsOwned(socket1) then
                return "tooltip_not_owned"
            end
            if GameTooltip:NumLines() == 0 then
                return "tooltip_empty"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result, "ok",
            "ItemSocketingFrame should load and populate from seeded C_ItemSocketInfo state: {result}"
        );
    }
}

#[test]
#[ignore = "EncounterTimeline behavior is intentionally unsupported by wow-ui-sim"]
fn encounter_timeline_loads_and_populates_track_view_event_frames() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            local loaded, reason = C_AddOns.LoadAddOn("Blizzard_EncounterTimeline")
            if not loaded then
                return "load_failed:" .. tostring(reason)
            end
            if not EncounterTimeline or not EncounterTimeline.TrackView then
                return "encounter_timeline_missing"
            end

            EncounterTimeline:UpdateSystemSettingViewType()
            EncounterTimeline:SetExplicitlyShown(true)
            EncounterTimeline.TrackView:ActivateView()
            EncounterTimeline:UpdateVisibility()

            if not EncounterTimeline:IsShown() then
                return "timeline_not_shown"
            end
            if not EncounterTimeline.TrackView:IsShown() then
                return "track_view_not_shown"
            end
            if EncounterTimeline.TrackView:GetTrackCount() < 3 then
                return "track_count=" .. tostring(EncounterTimeline.TrackView:GetTrackCount())
            end
            if EncounterTimeline.TrackView:GetActiveEventFrameCount() ~= 1 then
                return "active_frame_count=" .. tostring(EncounterTimeline.TrackView:GetActiveEventFrameCount())
            end

            local eventFrame = EncounterTimeline.TrackView:GetEventFrame(1)
            if not eventFrame then
                return "missing_event_frame"
            end
            if not eventFrame:IsShown() then
                return "event_frame_not_shown"
            end
            if not eventFrame.IconContainer or not eventFrame.IconContainer.IconTexture then
                return "missing_icon_container"
            end
            if not eventFrame.IconContainer.IconTexture:GetTexture() then
                return "missing_icon_texture"
            end

            local eventInfo = eventFrame:GetEventInfo()
            if not eventInfo or eventInfo.spellID ~= 19750 then
                return "event_info_spell_id=" .. tostring(eventInfo and eventInfo.spellID)
            end

            return "ok"
        "#).unwrap();
        assert_eq!(result, "ok", "EncounterTimeline should load and populate a visible track event frame: {result}");
    }
}

#[test]
fn damage_meter_loads_and_populates_primary_session_window() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            SetCVar("damageMeterEnabled", "1")

            local loaded, reason = C_AddOns.LoadAddOn("Blizzard_DamageMeter")
            if not loaded then
                return "load_failed:" .. tostring(reason)
            end
            if not DamageMeter then
                return "damage_meter_missing"
            end

            DamageMeter:OnVariablesLoaded()
            DamageMeter:SetShown(true)

            local sessionWindow = DamageMeter:GetPrimarySessionWindow()
            if not sessionWindow then
                return "primary_window_missing"
            end

            sessionWindow:Refresh(ScrollBoxConstants.DiscardScrollPosition)
            sessionWindow:GetScrollBox():FullUpdate(ScrollBoxConstants.UpdateImmediately)

            if not DamageMeter:IsShown() then
                return "damage_meter_hidden"
            end
            if not sessionWindow:IsShown() then
                return "session_window_hidden"
            end
            if sessionWindow:GetEntryFrameCount() == 0 then
                return "entry_frame_count=0"
            end

            local entryData = sessionWindow:GetScrollBox():GetDataProvider():Find(1)
            if not entryData then
                return "missing_entry_data"
            end
            if entryData.name ~= "Player" then
                return "entry_name=" .. tostring(entryData.name)
            end
            if entryData.totalAmount ~= 52000 then
                return "entry_total=" .. tostring(entryData.totalAmount)
            end
            if entryData.amountPerSecond ~= 1300 then
                return "entry_per_second=" .. tostring(entryData.amountPerSecond)
            end
            if not entryData.isLocalPlayer then
                return "entry_not_local_player"
            end

            sessionWindow:ShowSourceWindow(entryData)
            local sourceWindow = sessionWindow:GetSourceWindow()
            if not sourceWindow:IsShown() then
                return "source_window_hidden"
            end
            if sourceWindow:GetEntryFrameCount() == 0 then
                return "source_entry_frame_count=0"
            end

            local spellData = sourceWindow:GetScrollBox():GetDataProvider():Find(1)
            if not spellData then
                return "missing_spell_data"
            end
            if spellData.spellID ~= 19750 then
                return "spell_id=" .. tostring(spellData.spellID)
            end
            if spellData.totalAmount ~= 52000 then
                return "spell_total=" .. tostring(spellData.totalAmount)
            end

            return "ok"
        "#).unwrap();
        assert_eq!(result, "ok", "DamageMeter should load and populate the primary session and source windows: {result}");
    }
}

#[test]
fn audio_settings_register_seeded_output_device_dropdown_options() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            if not Settings or not Settings.AUDIO_CATEGORY_ID then
                return "audio_category_missing"
            end

            local category = Settings.GetCategory(Settings.AUDIO_CATEGORY_ID)
            if not category then
                return "audio_category_lookup_failed"
            end

            local layout = SettingsPanel:GetLayout(category)
            if not layout then
                return "audio_layout_missing"
            end

            local outputInitializer = nil
            for _, initializer in ipairs(layout:GetInitializers()) do
                local setting = initializer.GetSetting and initializer:GetSetting()
                if setting and setting:GetVariable() == "Sound_OutputDriverIndex" then
                    outputInitializer = initializer
                    break
                end
            end

            if not outputInitializer then
                return "output_initializer_missing"
            end

            local optionsFunc = outputInitializer:GetOptions()
            if type(optionsFunc) ~= "function" then
                return "options_func_type=" .. tostring(type(optionsFunc))
            end

            local options = optionsFunc()
            if #options ~= 1 then
                return "option_count=" .. tostring(#options)
            end
            if options[1].value ~= 0 then
                return "option_value=" .. tostring(options[1].value)
            end
            if options[1].label ~= "Silent Output Device" then
                return "option_label=" .. tostring(options[1].label)
            end

            return "ok"
        "#).unwrap();
        assert_eq!(result, "ok", "Audio settings should register a seeded output-device dropdown option: {result}");
    }
}

#[test]
fn settings_open_to_interface_category_opens_settings_panel() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            if not Settings or not Settings.OpenToCategory then
                return "missing_settings_open_to_category"
            end
            local categoryID = Settings.INTERFACE_CATEGORY_ID
            if not categoryID then
                return "missing_interface_category_id"
            end
            Settings.OpenToCategory(categoryID)
            if not SettingsPanel or not SettingsPanel:IsShown() then
                return "settings_panel_not_shown"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "Settings.OpenToCategory(Settings.INTERFACE_CATEGORY_ID) should open SettingsPanel on the interface category: {result}"
        );
    }
}

#[test]
#[ignore = "C_ProfSpecs path/tree semantics are not modeled"]
fn professions_frame_loads_and_populates_specialization_tab() {
    test_timeout! {
        let env = setup_env();
        let ui = blizzard_ui_dir();
        for (name, toc) in [
            ("Blizzard_FrameXMLUtil", "Blizzard_FrameXMLUtil.toc"),
            ("Blizzard_ProfessionsTemplates", "Blizzard_ProfessionsTemplates.toc"),
            ("Blizzard_SpellSearch", "Blizzard_SpellSearch.toc"),
            ("Blizzard_SharedTalentUI", "Blizzard_SharedTalentUI.toc"),
            ("Blizzard_Professions", "Blizzard_Professions.toc"),
        ] {
            let toc_path = ui.join(name).join(toc);
            if let Err(error) = load_addon(&env.loader_env(), &toc_path) {
                panic!("failed to load {name}: {error}");
            }
        }
        let result: String = env.eval(r#"
            if not ProfessionsFrame or not ProfessionsFrame.SpecPage or not Professions then
                return "professions_spec_page_missing"
            end

            local professionInfo = Professions.GetProfessionInfo()
            if not professionInfo or professionInfo.professionID ~= 164 then
                return "profession_info=" .. tostring(professionInfo and professionInfo.professionID)
            end

            ProfessionsFrame.SpecPage:Refresh(professionInfo)

            local selectedTab = ProfessionsFrame.SpecPage.selectedTab
            if not selectedTab then
                return "selected_spec_tab_missing"
            end
            if not selectedTab.tabInfo or not selectedTab.tabInfo.rootNodeID then
                return "selected_tab_root_missing"
            end

            local rootNodeID = selectedTab.tabInfo.rootNodeID
            if ProfessionsFrame.SpecPage:GetTalentTreeID() ~= selectedTab.traitTreeID then
                return "talent_tree_id=" .. tostring(ProfessionsFrame.SpecPage:GetTalentTreeID())
            end
            local children = C_ProfSpecs.GetChildrenForPath(rootNodeID)
            if #children == 0 then
                return "root_children=0"
            end

            if not ProfessionsFrame.SpecPage:IsTreeDirty() then
                return "tree_not_marked_dirty"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(result, "ok", "ProfessionsFrame should show a populated specialization tab: {result}");
    }
}

#[test]
fn professions_frame_xml_pages_are_parent_keyed() {
    test_timeout! {
        let env = setup_env();
        let ui = blizzard_ui_dir();
        for (name, toc) in [
            ("Blizzard_FrameXMLUtil", "Blizzard_FrameXMLUtil.toc"),
            ("Blizzard_ProfessionsTemplates", "Blizzard_ProfessionsTemplates.toc"),
            ("Blizzard_SharedTalentUI", "Blizzard_SharedTalentUI.toc"),
            ("Blizzard_Professions", "Blizzard_Professions.toc"),
        ] {
            let toc_path = ui.join(name).join(toc);
            if let Err(error) = load_addon(&env.loader_env(), &toc_path) {
                panic!("failed to load {name}: {error}");
            }
        }

        let result: String = env.eval(r#"
            if not ProfessionsFrame then return "professions_frame_missing" end
            if not ProfessionsFrame.CraftingPage then return "crafting_page_missing" end
            if not ProfessionsFrame.SpecPage then return "spec_page_missing" end
            if not ProfessionsFrame.OrdersPage then return "orders_page_missing" end
            if not ProfessionsFrame.CraftingPage.LinkButton then return "link_button_missing" end
            if not ProfessionsFrame.Pages or #ProfessionsFrame.Pages ~= 3 then
                return "pages=" .. tostring(ProfessionsFrame.Pages and #ProfessionsFrame.Pages or nil)
            end
            return "ok"
        "#).unwrap();

        assert_eq!(
            result,
            "ok",
            "ProfessionsFrame XML page children should be available through parentKey: {result}"
        );
    }
}

#[test]
fn loss_of_control_frame_shows_seeded_overlay_on_added_event() {
    test_timeout! {
        let env = setup_env();
        let loss_of_control_enabled: bool = env
            .eval(
                "Constants.LossOfControlConsts = { LOSS_OF_CONTROL_ACTIVE_INDEX = 1 }; \
                 RegisterCVar('lossOfControl', '0'); return SetCVar('lossOfControl', '1')",
            )
            .expect("failed to register and enable loss-of-control events");
        assert!(
            loss_of_control_enabled,
            "lossOfControl CVar should accept the enabled value"
        );
        let _ = env.fire_event_with_args(
            "LOSS_OF_CONTROL_ADDED",
            &[
                env.lua_string("player"),
                rilua::Val::Num(1.0),
            ],
        );

        let result: String = env.eval(r#"
            if not LossOfControlFrame then
                return "missing_frame"
            end
            if not LossOfControlFrame:IsShown() then
                return "frame_hidden"
            end
            if LossOfControlFrame.AbilityName:GetText() ~= "Kidney Shot" then
                return "text=" .. tostring(LossOfControlFrame.AbilityName:GetText())
            end
            if LossOfControlFrame.Icon:GetTexture() ~= 132298 then
                return "icon=" .. tostring(LossOfControlFrame.Icon:GetTexture())
            end
            if not LossOfControlFrame.TimeLeft:IsShown() then
                return "time_hidden"
            end
            return "ok"
        "#).unwrap();

        assert_eq!(
            result, "ok",
            "LossOfControlFrame should show the seeded loss-of-control overlay: {result}"
        );
    }
}
