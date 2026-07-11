use std::path::Path;

const RETAIL_REQUIRED_PROFILE_CACHE_ENTRIES: &[&str] = &[
    "Blizzard_FrameXMLUtil/RuneforgeUtil.xml",
    "Blizzard_FrameXMLUtil/RuneforgeUtil.lua",
];

const PTR_REQUIRED_PROFILE_CACHE_ENTRIES: &[&str] = &[
    "Blizzard_FrameXMLUtil/RuneforgeUtil.xml",
    "Blizzard_FrameXMLUtil/RuneforgeUtil.lua",
    "Blizzard_CooldownBroadcaster/Blizzard_CooldownBroadcaster_Bootstrap.lua",
];

pub(super) const MISTS_REQUIRED_PROFILE_CACHE_ENTRIES: &[&str] = &[
    "Blizzard_ActionBar/Classic/ActionButtonTemplate.xml",
    "Blizzard_ActionBar/Classic/ActionButtonUtilOverrides.lua",
    "Blizzard_ActionBar/Classic/ExpBar.xml",
    "Blizzard_ActionBar/Classic/ExpBarOverrides.lua",
    "Blizzard_ActionBar/Classic/MainActionBar.xml",
    "Blizzard_ActionBar/Classic/MainActionBarOverrides.lua",
    "Blizzard_ActionBar/Classic/MainMenuBar.lua",
    "Blizzard_ActionBar/Classic/MainMenuBar.xml",
    "Blizzard_ActionBar/Classic/PetActionBar.xml",
    "Blizzard_ActionBar/Classic/PossessActionBar.xml",
    "Blizzard_ActionBar/Classic/ReputationBarOverrides.lua",
    "Blizzard_ActionBar/Classic/StanceBar.xml",
    "Blizzard_ActionBar/Classic/StanceBarOverrides.lua",
    "Blizzard_ActionBar/Classic/StatusTrackingBar.xml",
    "Blizzard_ActionBar/Classic/StatusTrackingBarTemplate.xml",
    "Blizzard_ActionBar/Classic/StatusTrackingManagerOverrides.lua",
    "Blizzard_ChatFrame/Classic/BattlegroundChatFilters.lua",
    "Blizzard_ChatFrame/Classic/BattlegroundChatFilters.xml",
    "Blizzard_ChatFrame/Classic/ChatConfigFrame.xml",
    "Blizzard_ChatFrame/Classic/ChatConfigFrame_Shared.lua",
    "Blizzard_ChatFrame/Classic/Localization.lua",
    "Blizzard_ChatFrame/Classic/TextToSpeechFrameConstants.lua",
    "Blizzard_ChatFrame/Classic/VoiceChatHeadsetButton.lua",
    "Blizzard_ChatFrame/Classic/VoiceChatHeadsetButton.xml",
    "Blizzard_ChatFrame/Wrath/ChatConfigFrame.lua",
    "Blizzard_ChatFrameBase/Blizzard_ChatFrameBase_Classic.toc",
    "Blizzard_ChatFrameBase/Classic/ChannelFrameButtonMixin.lua",
    "Blizzard_ChatFrameBase/Classic/ChatEmoteConstants.lua",
    "Blizzard_ChatFrameBase/Classic/ChatFrame.xml",
    "Blizzard_ChatFrameBase/Classic/ChatFrameEditBox.xml",
    "Blizzard_ChatFrameBase/Classic/ChatFrameEditBoxOverrides.lua",
    "Blizzard_ChatFrameBase/Classic/ChatFrameMenuButton.lua",
    "Blizzard_ChatFrameBase/Classic/ChatFrameOverrides.lua",
    "Blizzard_ChatFrameBase/Classic/ChatFrameUtilOverrides.lua",
    "Blizzard_ChatFrameBase/Classic/ChatTypeInfoConstantsOverride.lua",
    "Blizzard_ChatFrameBase/Classic/FloatingChatFrame.lua",
    "Blizzard_ChatFrameBase/Classic/FloatingChatFrame.xml",
    "Blizzard_ChatFrameBase/Classic/FriendFrameButtonMixin.lua",
    "Blizzard_ChatFrameBase/Classic/SlashCommandsOverrides.lua",
    "Blizzard_ChatFrameBase/Shared/ChatFrameUtil.lua",
    "Blizzard_FrameXMLUtil/Blizzard_FrameXMLUtil_Classic.toc",
    "Blizzard_FrameXMLUtil/Classic/ArenaUtil.lua",
    "Blizzard_FrameXMLUtil/Classic/AuraUtil.lua",
    "Blizzard_FrameXMLUtil/Classic/Cooldown.xml",
    "Blizzard_FrameXMLUtil/Classic/MapUtil.lua",
    "Blizzard_FrameXMLUtil/Classic/QuestUtils.lua",
    "Blizzard_FrameXMLUtil/Classic/RaidWarning.lua",
    "Blizzard_FrameXMLUtil/Classic/TransmogUtil.lua",
    "Blizzard_Fonts_Shared/Classic/FontStyles.xml",
    "Blizzard_Fonts_Shared/Classic/Fonts.xml",
    "Blizzard_Fonts_Shared/Classic/GameFontStyles.xml",
    "Blizzard_Fonts_Shared/Classic/GameFonts.xml",
    "Blizzard_Fonts_Shared/Classic/GlueFontStyles.xml",
    "Blizzard_Fonts_Shared/Classic/GlueFonts.xml",
    "Blizzard_MoneyFrame/Blizzard_MoneyFrame_Classic.toc",
    "Blizzard_MoneyFrame/Classic/Localization.lua",
    "Blizzard_MoneyFrame/Classic/MoneyFrame.lua",
    "Blizzard_MoneyFrame/Classic/MoneyFrame.xml",
    "Blizzard_MoneyFrame/Classic/MoneyInputFrame.lua",
    "Blizzard_MoneyFrame/Classic/MoneyInputFrame.xml",
    "Blizzard_ObjectAPI/Blizzard_ObjectAPI_Classic.toc",
    "Blizzard_ObjectAPI/Classic/ContinuableContainer.lua",
    "Blizzard_ObjectAPI/Classic/Item.lua",
    "Blizzard_ObjectAPI/Classic/ItemLocation.lua",
    "Blizzard_ObjectAPI/Classic/PlayerLocation.lua",
    "Blizzard_ObjectAPI/Classic/Spell.lua",
    "Blizzard_SharedXML/Classic/GameTooltipTemplate.lua",
    "Blizzard_FrameXML/TBC/WorldStateFrame.lua",
    "Blizzard_MainMenuBarBagButtons/Classic/MainMenuBarBagButtons.lua",
    "Blizzard_MicroMenu/Blizzard_MicroMenu_Classic.toc",
    "Blizzard_MicroMenu/Shared/MicroMenuContainer.lua",
    "Blizzard_SettingsDefinitions_Frame/Blizzard_SettingsDefinitions_Frame_Classic.toc",
    "Blizzard_SettingsDefinitions_Frame/Classic/AccessibilityOverrides.lua",
    "Blizzard_SettingsDefinitions_Frame/Classic/Colorblind.xml",
    "Blizzard_SettingsDefinitions_Frame/Classic/ColorblindOverrides.lua",
    "Blizzard_SettingsDefinitions_Frame/Classic/CombatOverrides.lua",
    "Blizzard_SettingsDefinitions_Frame/Classic/ControlsOverrides.lua",
    "Blizzard_SettingsDefinitions_Frame/Classic/GameplaySettingsGroup.lua",
    "Blizzard_SettingsDefinitions_Frame/Classic/InterfaceOverrides.lua",
    "Blizzard_SettingsDefinitions_Frame/Classic/KeybindingsOverrides.lua",
    "Blizzard_SettingsDefinitions_Frame/Classic/SocialOverrides.lua",
    "Blizzard_SettingsDefinitions_Frame/CombatAudioAlertUtil.lua",
    "Blizzard_SharedMapDataProviders/Blizzard_SharedMapDataProviders_Mists.toc",
    "Blizzard_SharedMapDataProviders/Wrath/BonusObjectiveDataProvider.lua",
    "Blizzard_SharedMapDataProviders/Wrath/DeathMapDataProvider.xml",
    "Blizzard_UIParentPanelManager/Blizzard_UIParentPanelManager_Classic.toc",
    "Blizzard_UIParentPanelManager/Classic/UIPanelWindows.lua",
    "Blizzard_UIParentPanelManager/Classic/UIParentPanelManagerOverrides.lua",
    "Blizzard_UIParent/Classic/UIParent.lua",
    "Blizzard_UIParent/Classic/UIParent.xml",
    "Blizzard_UIParent/Classic/UIParentDebugMenu.lua",
    "Blizzard_UIParent/Classic/WorldFrame.lua",
    "Blizzard_UIParent/Classic/WorldFrame.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgets.toc",
    "Blizzard_UIWidgets/Blizzard_UIWidgetAnimationTemplates.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetAnimationTemplates.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetBelowMinimapFrame.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetBelowMinimapFrame.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetCenterScreenFrame.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetCenterScreenFrame.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetManager.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetManager.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateBase.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateBase.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateBulletTextList.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateBulletTextList.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateButtonHeader.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateButtonHeader.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateCaptureBar.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateCaptureBar.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateCaptureZone.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateCaptureZone.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateDiscreteProgressSteps.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateDiscreteProgressSteps.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateDoubleIconAndText.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateDoubleIconAndText.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateDoubleStateIconRow.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateDoubleStateIconRow.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateDoubleStatusBar.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateDoubleStatusBar.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateFillUpFrames.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateFillUpFrames.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateHorizontalCurrencies.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateHorizontalCurrencies.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateIconAndText.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateIconAndText.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateIconTextAndBackground.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateIconTextAndBackground.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateIconTextAndCurrencies.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateIconTextAndCurrencies.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateItemDisplay.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateItemDisplay.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateMapPinAnimation.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateMapPinAnimation.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplatePreyHuntProgress.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplatePreyHuntProgress.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateScenarioHeaderCurrenciesAndBackground.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateScenarioHeaderCurrenciesAndBackground.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateScenarioHeaderDelves.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateScenarioHeaderDelves.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateScenarioHeaderTimer.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateScenarioHeaderTimer.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateSpacer.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateSpacer.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateSpellDisplay.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateSpellDisplay.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateStackedResourceTracker.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateStackedResourceTracker.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateStatusBar.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateStatusBar.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTextColumnRow.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTextColumnRow.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTextWithState.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTextWithState.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTextWithSubtext.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTextWithSubtext.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTextureAndText.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTextureAndText.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTextureAndTextRow.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTextureAndTextRow.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTextureWithAnimation.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTextureWithAnimation.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTugOfWar.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateTugOfWar.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateUnitPowerBar.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateUnitPowerBar.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateZoneControl.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTemplateZoneControl.xml",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTopCenterFrame.lua",
    "Blizzard_UIWidgets/Blizzard_UIWidgetTopCenterFrame.xml",
    "Blizzard_UIWidgets/Blizzard_WidgetsUtil.lua",
    "Blizzard_SharedXML/Classic/ClassicCvarUtil.lua",
    "Blizzard_SharedXML/Classic/Dialog/DialogTemplates.xml",
    "Blizzard_SharedXML/Classic/Frame/MainMenuFrameTemplates.xml",
    "Blizzard_SharedXML/Classic/GameTooltipTemplate.xml",
    "Blizzard_SharedXML/Classic/GlueCheck.lua",
    "Blizzard_SharedXML/Classic/ModelFrames.lua",
    "Blizzard_SharedXML/Classic/ModelFrames.xml",
    "Blizzard_SharedXML/Classic/NineSliceLayouts.lua",
    "Blizzard_SharedXML/Classic/Scroll/TrimScrollBar.xml",
    "Blizzard_SharedXML/Classic/ScrollDefine.lua",
    "Blizzard_SharedXML/Classic/ScrollDefine.xml",
    "Blizzard_SharedXML/Classic/SecureCurrencyUtil.lua",
    "Blizzard_SharedXML/Classic/Selector/Blizzard_ScrollBoxSelector.xml",
    "Blizzard_SharedXML/Classic/SharedUIPanelTemplates.lua",
    "Blizzard_SharedXML/Classic/SharedUIPanelTemplates.xml",
    "Blizzard_SharedXML/Classic/SharedUtils.lua",
    "Blizzard_SharedXML/Classic/SliderTemplates.xml",
    "Blizzard_SharedXML/Classic/Sound.lua",
    "Blizzard_SharedXML/Classic/Stubs.lua",
    "Blizzard_SharedXML/Classic/Stubs.xml",
    "Blizzard_SharedXML/Classic/UIDropDownMenu.lua",
    "Blizzard_SharedXML/Classic/UIDropDownMenu.xml",
    "Blizzard_SharedXML/Classic/UIDropDownMenuTemplates.lua",
    "Blizzard_SharedXML/Classic/UIDropDownMenuTemplates.xml",
    "Blizzard_SharedXML/TBC/ClassColors.lua",
    "Blizzard_SharedXML/Wrath/SoundKitConstants.lua",
    "Blizzard_FrameXMLBase/Classic/Constants.lua",
    "Blizzard_FrameXMLBase/Classic/FrameLocks.lua",
    "Blizzard_FrameXMLBase/Classic/IconDataProvider.lua",
    "Blizzard_FrameXMLBase/TBC/Localization.lua",
    "Blizzard_FrameXMLBase/Wrath/Constants.lua",
    "Blizzard_FrameXML/Classic/AlertFrameSystems.lua",
    "Blizzard_FrameXML/Classic/AlertFrameSystems.xml",
    "Blizzard_FrameXML/Classic/AlertFrames.lua",
    "Blizzard_FrameXML/Classic/AlertFrames.xml",
    "Blizzard_FrameXML/Classic/ColorPickerFrame.lua",
    "Blizzard_FrameXML/Classic/ColorPickerFrame.xml",
    "Blizzard_FrameXML/Classic/CombatFeedback.lua",
    "Blizzard_BuffFrame/Classic/BuffFrame.lua",
    "Blizzard_GameTooltip/Blizzard_GameTooltip_Classic.toc",
    "Blizzard_GameTooltip/Classic/GameTooltip.lua",
    "Blizzard_GameTooltip/Classic/GameTooltip.xml",
    "Blizzard_ItemButton/Blizzard_ItemButton_Classic.toc",
    "Blizzard_ItemButton/Classic/ItemButtonTemplate.lua",
    "Blizzard_ItemButton/Classic/ItemButtonTemplate.xml",
    "Blizzard_LevelUpDisplay/Cata/LevelUpDisplay.lua",
    "Blizzard_NamePlates/Blizzard_NamePlates.toc",
    "Blizzard_Settings_Shared/Blizzard_Settings_Shared_Classic.toc",
    "Blizzard_Settings_Shared/Classic/AudioOverrides.lua",
    "Blizzard_Settings_Shared/Classic/Blizzard_SettingsPanelTemplates.xml",
    "Blizzard_Settings_Shared/Classic/GraphicsOverrides.lua",
    "Blizzard_UIPanelTemplates/Blizzard_UIPanelTemplates_Classic.toc",
    "Blizzard_UIPanelTemplates/Classic/AutoCastTemplates.lua",
    "Blizzard_UIPanelTemplates/Classic/AutoCastTemplates.xml",
    "Blizzard_UIPanelTemplates/Classic/UIPanelTemplates.lua",
    "Blizzard_UIPanelTemplates/Classic/UIPanelTemplates.xml",
    "Blizzard_UIPanels_Game/Blizzard_UIPanels_Game_Mists.toc",
    "Blizzard_UIPanels_Game/Classic/CastingBarFrame.lua",
    "Blizzard_UIPanels_Game/Classic/CastingBarFrame.xml",
    "Blizzard_UIPanels_Game/Shared/CastingBarFrame.lua",
    "Blizzard_WorldMap/Blizzard_WorldMapTooltip.xml",
    "Blizzard_WorldMap/Blizzard_WorldMap_Mists.toc",
    "Blizzard_WorldMap/Cata/Blizzard_WorldMap.lua",
    "Blizzard_WorldMap/Cata/Blizzard_WorldMap.xml",
    "Blizzard_WorldMap/WM_InvasionDataProvider.lua",
    "Blizzard_WorldMap/WM_InvasionDataProvider.xml",
    "Blizzard_WorldMap/Wrath/QuestLogOwnerMixin.lua",
    "Blizzard_UnitFrame/Cata/EclipseBarFrame.lua",
    "Blizzard_UnitFrame/Cata/EclipseBarFrame.xml",
    "Blizzard_UnitFrame/Cata/RuneFrame.lua",
    "Blizzard_UnitFrame/Cata/RuneFrame.xml",
    "Blizzard_UnitFrame/Classic/ComboFrame.lua",
    "Blizzard_UnitFrame/Classic/ComboFrame.xml",
    "Blizzard_UnitFrame/Classic/CompactUnitFrameOptions.lua",
    "Blizzard_UnitFrame/Classic/Localization.lua",
    "Blizzard_UnitFrame/Classic/PartyFrameTemplates.xml",
    "Blizzard_UnitFrame/Classic/PartyMemberFrame.lua",
    "Blizzard_UnitFrame/Classic/PetFrame.lua",
    "Blizzard_UnitFrame/Classic/PetFrame.xml",
    "Blizzard_UnitFrame/Classic/PlayerFrame.lua",
    "Blizzard_UnitFrame/Classic/PlayerFrame.xml",
    "Blizzard_UnitFrame/Classic/RuneFrame_Shared.lua",
    "Blizzard_UnitFrame/Classic/TargetFrame.lua",
    "Blizzard_UnitFrame/Classic/TargetFrame.xml",
    "Blizzard_UnitFrame/Classic/TotemFrame.lua",
    "Blizzard_UnitFrame/Classic/TotemFrame.xml",
    "Blizzard_UnitFrame/Classic/UnitFrame.lua",
    "Blizzard_UnitFrame/Classic/UnitPowerBarAlt.lua",
    "Blizzard_UnitFrame/Classic/UnitPowerBarAlt.xml",
    "Blizzard_UnitFrame/Mists/AlternatePowerBar.xml",
    "Blizzard_UnitFrame/Mists/MonkHarmonyBar.lua",
    "Blizzard_UnitFrame/Mists/MonkHarmonyBar.xml",
    "Blizzard_UnitFrame/Mists/MonkStaggerBar.lua",
    "Blizzard_UnitFrame/Mists/MonkStaggerBar.xml",
    "Blizzard_UnitFrame/Mists/PaladinPowerBar.lua",
    "Blizzard_UnitFrame/Mists/PaladinPowerBar.xml",
    "Blizzard_UnitFrame/Mists/PriestBar.xml",
    "Blizzard_UnitFrame/Mists/ShardBar.lua",
    "Blizzard_UnitFrame/Mists/ShardBar.xml",
    "Blizzard_UnitFrame/Mists/TotemFrame.lua",
    "Blizzard_UnitFrame/Wrath/AlternatePowerBar.lua",
];

pub(super) fn required_profile_cache_entries() -> &'static [&'static str] {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail => RETAIL_REQUIRED_PROFILE_CACHE_ENTRIES,
        crate::client_profile::ClientProfile::Ptr => PTR_REQUIRED_PROFILE_CACHE_ENTRIES,
        crate::client_profile::ClientProfile::Mists => MISTS_REQUIRED_PROFILE_CACHE_ENTRIES,
        _ => &[],
    }
}

pub(super) fn sync_entry_belongs_to_active_profile(entry: &str) -> bool {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail => retail_sync_entry(entry),
        crate::client_profile::ClientProfile::Ptr => ptr_sync_entry(entry),
        _ => true,
    }
}

pub(super) fn cache_entry_is_usable(entry: &str, path: &Path) -> bool {
    if text_manifest_entry(entry) && !is_valid_utf8_file(path) {
        return false;
    }

    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail
        | crate::client_profile::ClientProfile::Ptr => retail_cache_entry_is_usable(entry, path),
        crate::client_profile::ClientProfile::Mists => mists_cache_entry_is_usable(entry, path),
        _ => true,
    }
}

fn retail_sync_entry(entry: &str) -> bool {
    !ptr_only_entry(entry)
}

fn ptr_sync_entry(entry: &str) -> bool {
    !legacy_profile_entry(entry) && !ptr_removed_mainline_entry(entry)
}

fn ptr_only_entry(entry: &str) -> bool {
    matches!(
        entry,
        "Blizzard_CooldownBroadcaster/Blizzard_CooldownBroadcaster_Bootstrap.lua"
    )
}

fn legacy_profile_entry(entry: &str) -> bool {
    entry.contains("/Classic/")
        || entry.contains("/Mists/")
        || entry.contains("/Wrath/")
        || entry.contains("/Cata/")
        || entry.contains("/TBC/")
        || entry.contains("_Classic.toc")
        || entry.contains("_Mists.toc")
        || entry.contains("_Wrath.toc")
        || entry.contains("_Vanilla.toc")
}

fn ptr_removed_mainline_entry(entry: &str) -> bool {
    matches!(
        entry,
        "Blizzard_WorldMap/Blizzard_WorldMapTooltip.xml"
            | "Blizzard_WorldMap/WM_InvasionDataProvider.lua"
            | "Blizzard_WorldMap/WM_InvasionDataProvider.xml"
    )
}

fn text_manifest_entry(entry: &str) -> bool {
    let lower = entry.to_ascii_lowercase();
    lower.ends_with(".lua")
        || lower.ends_with(".xml")
        || lower.ends_with(".toc")
        || lower.ends_with(".xsd")
}

fn is_valid_utf8_file(path: &Path) -> bool {
    std::fs::read_to_string(path).is_ok()
}

fn retail_cache_entry_is_usable(entry: &str, path: &Path) -> bool {
    match entry {
        "Blizzard_FrameXMLUtil/RuneforgeUtil.xml" => {
            file_contains(path, r#"<Script file="RuneforgeUtil.lua"/>"#)
                || file_contains(path, r#"name="RuneforgeCovenantSigilTemplate""#)
        }
        _ => true,
    }
}

fn mists_cache_entry_is_usable(entry: &str, path: &Path) -> bool {
    mists_action_bar_cache_entry_is_usable(entry, path)
        .or_else(|| mists_core_cache_entry_is_usable(entry, path))
        .or_else(|| mists_map_cache_entry_is_usable(entry, path))
        .unwrap_or(true)
}

fn mists_action_bar_cache_entry_is_usable(entry: &str, path: &Path) -> Option<bool> {
    let is_usable = match entry {
        "Blizzard_ActionBar/Classic/ActionButtonTemplate.xml" => {
            file_contains(path, "ActionBarButtonTemplate")
                && file_contains(path, r#"parentKey="chargeCooldown""#)
        }
        "Blizzard_ActionBar/Classic/MainMenuBar.lua" => {
            file_contains(path, "function MainMenuBar_SetMaxLevelBarShown(shown)")
        }
        "Blizzard_ActionBar/Classic/MainMenuBar.xml" => {
            file_contains(path, r#"name="MainMenuBarMaxLevelBar""#)
                && !file_contains(path, "ExpBar_Update()")
                && !file_contains(path, "ExhaustionTick_OnUpdate")
        }
        "Blizzard_ActionBar/Classic/PossessActionBar.xml" => {
            file_contains(path, r#"name="PossessActionBar""#)
                && file_contains(path, "PossessActionBarMixin")
        }
        _ => return None,
    };

    Some(is_usable)
}

fn mists_core_cache_entry_is_usable(entry: &str, path: &Path) -> Option<bool> {
    let is_usable = match entry {
        "Blizzard_ChatFrame/Classic/ChatConfigFrame.xml" => mists_chat_config_is_usable(path),
        "Blizzard_MicroMenu/Blizzard_MicroMenu_Classic.toc" => file_contains(
            path,
            r#"Cata\MainMenuBarMicroButtons.xml [AllowLoadGameType mists]"#,
        ),
        "Blizzard_MicroMenu/Shared/MicroMenuContainer.lua" => {
            !file_contains(path, "PostAddButtonCallback")
        }
        "Blizzard_Fonts_Shared/Classic/GameFonts.xml" => {
            file_contains(path, r#"FontFamily name="PriceFont""#)
        }
        "Blizzard_MoneyFrame/Blizzard_MoneyFrame_Classic.toc" => mists_money_toc_is_usable(path),
        "Blizzard_MoneyFrame/Classic/MoneyInputFrame.xml" => mists_money_input_is_usable(path),
        "Blizzard_NamePlates/Blizzard_NamePlates.toc" => file_contains(
            path,
            "Blizzard_ClassNameplateBar.lua [AllowLoadGameType mainline]",
        ),
        "Blizzard_UIPanels_Game/Shared/CastingBarFrame.lua" => {
            file_contains(path, "function PlayerCastingBarMixin:OnShow()")
        }
        _ => return None,
    };

    Some(is_usable)
}

fn mists_map_cache_entry_is_usable(entry: &str, path: &Path) -> Option<bool> {
    let is_usable = match entry {
        "Blizzard_SharedMapDataProviders/Blizzard_SharedMapDataProviders_Mists.toc" => {
            mists_shared_map_data_providers_toc_is_usable(path)
        }
        "Blizzard_WorldMap/Blizzard_WorldMap_Mists.toc" => mists_world_map_toc_is_usable(path),
        "Blizzard_WorldMap/Cata/Blizzard_WorldMap.xml" => mists_world_map_xml_is_usable(path),
        _ => return None,
    };

    Some(is_usable)
}

fn mists_chat_config_is_usable(path: &Path) -> bool {
    file_contains(path, r#"name="ChatConfigCheckButtonTemplate""#)
        && file_contains(path, r#"name="$parentText""#)
}

fn mists_money_toc_is_usable(path: &Path) -> bool {
    file_contains(path, "Classic\\MoneyInputFrame.lua")
        && file_contains(path, "## AllowLoadGameType: classic")
}

fn mists_money_input_is_usable(path: &Path) -> bool {
    file_contains(path, r#"name="MoneyInputFrameTemplate""#)
        && file_contains(path, r#"parentKey="copper""#)
}

fn mists_shared_map_data_providers_toc_is_usable(path: &Path) -> bool {
    file_contains(path, "## AllowLoadGameType: mists")
        && file_contains(path, "Wrath\\BonusObjectiveDataProvider.lua")
}

fn mists_world_map_toc_is_usable(path: &Path) -> bool {
    file_contains(path, "## AllowLoadGameType: mists")
        && file_contains(path, "Cata\\Blizzard_WorldMap.xml")
}

fn mists_world_map_xml_is_usable(path: &Path) -> bool {
    file_contains(path, r#"name="WorldMapTrackQuest""#)
        && file_contains(path, r#"parentKey="WorldMapOptionsDropDown""#)
}

fn file_contains(path: &Path, needle: &str) -> bool {
    std::fs::read_to_string(path).is_ok_and(|contents| contents.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::cache_entry_is_usable;
    #[cfg(any(feature = "client-retail", feature = "client-ptr"))]
    use super::required_profile_cache_entries;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "wow-ui-sim-profile-cache-{label}-{}-{unique}",
            std::process::id()
        ))
    }

    #[test]
    #[cfg(feature = "client-retail")]
    fn retail_requires_runeforge_util_cache_entries() {
        let required = required_profile_cache_entries();

        assert!(required.contains(&"Blizzard_FrameXMLUtil/RuneforgeUtil.xml"));
        assert!(required.contains(&"Blizzard_FrameXMLUtil/RuneforgeUtil.lua"));
    }

    #[test]
    #[cfg(feature = "client-ptr")]
    fn ptr_requires_runeforge_util_cache_entries() {
        let required = required_profile_cache_entries();

        assert!(required.contains(&"Blizzard_FrameXMLUtil/RuneforgeUtil.xml"));
        assert!(required.contains(&"Blizzard_FrameXMLUtil/RuneforgeUtil.lua"));
    }

    #[test]
    #[cfg(feature = "client-ptr")]
    fn ptr_requires_cooldown_broadcaster_bootstrap_cache_entry() {
        let required = required_profile_cache_entries();

        assert!(
            required.contains(
                &"Blizzard_CooldownBroadcaster/Blizzard_CooldownBroadcaster_Bootstrap.lua"
            )
        );
        assert!(super::sync_entry_belongs_to_active_profile(
            "Blizzard_CooldownBroadcaster/Blizzard_CooldownBroadcaster_Bootstrap.lua"
        ));
    }

    #[test]
    #[cfg(feature = "client-retail")]
    fn retail_excludes_ptr_only_cooldown_broadcaster_bootstrap() {
        assert!(!super::sync_entry_belongs_to_active_profile(
            "Blizzard_CooldownBroadcaster/Blizzard_CooldownBroadcaster_Bootstrap.lua"
        ));
    }

    #[test]
    #[cfg(feature = "client-retail")]
    fn retail_rejects_runeforge_xml_without_script_include() {
        let root = unique_temp_dir("retail-runeforge-util");
        std::fs::create_dir_all(&root).expect("create cache root");
        let runeforge_xml = root.join("RuneforgeUtil.xml");

        std::fs::write(&runeforge_xml, r#"<Ui></Ui>"#).expect("write stale RuneforgeUtil.xml");
        assert!(
            !cache_entry_is_usable("Blizzard_FrameXMLUtil/RuneforgeUtil.xml", &runeforge_xml),
            "Retail cache must reject RuneforgeUtil.xml without the Lua include"
        );

        std::fs::write(
            &runeforge_xml,
            r#"<Ui><Script file="RuneforgeUtil.lua"/></Ui>"#,
        )
        .expect("write legacy RuneforgeUtil.xml with Lua include");
        assert!(
            cache_entry_is_usable("Blizzard_FrameXMLUtil/RuneforgeUtil.xml", &runeforge_xml),
            "Retail cache should accept a legacy RuneforgeUtil.xml with the Lua include"
        );

        std::fs::write(
            &runeforge_xml,
            r#"<Ui><Frame name="RuneforgeCovenantSigilTemplate" mixin="RuneforgeCovenantSigilMixin" virtual="true"/></Ui>"#,
        )
        .expect("write current RuneforgeUtil.xml");
        assert!(
            cache_entry_is_usable("Blizzard_FrameXMLUtil/RuneforgeUtil.xml", &runeforge_xml),
            "Retail cache should accept the current RuneforgeUtil.xml template surface"
        );

        std::fs::remove_dir_all(root).expect("remove cache root");
    }

    #[test]
    fn text_manifest_entries_reject_binary_cache_data() {
        let root = unique_temp_dir("binary-text-entry");
        std::fs::create_dir_all(&root).expect("create temp root");
        let lua_path = root.join("Bad.lua");
        std::fs::write(&lua_path, [0xff, 0xfe, 0xfd]).expect("write binary lua");

        assert!(
            !cache_entry_is_usable("Blizzard_Test/Bad.lua", &lua_path),
            "text manifest entries must reject binary cache contents"
        );

        std::fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    #[cfg(feature = "client-mists")]
    fn mists_rejects_main_menu_bar_without_max_level_bar_helper() {
        let root = unique_temp_dir("mists-main-menu-bar");
        std::fs::create_dir_all(&root).expect("create cache root");
        let main_menu_bar = root.join("MainMenuBar.lua");

        std::fs::write(&main_menu_bar, "function MainMenuBar_OnLoad() end")
            .expect("write stale MainMenuBar.lua");
        assert!(
            !cache_entry_is_usable("Blizzard_ActionBar/Classic/MainMenuBar.lua", &main_menu_bar),
            "Mists 5.5.4 cache must reject MainMenuBar.lua without MainMenuBar_SetMaxLevelBarShown"
        );

        std::fs::write(
            &main_menu_bar,
            "function MainMenuBar_SetMaxLevelBarShown(shown) end",
        )
        .expect("write current MainMenuBar.lua");
        assert!(
            cache_entry_is_usable("Blizzard_ActionBar/Classic/MainMenuBar.lua", &main_menu_bar),
            "Mists 5.5.4 cache should accept MainMenuBar.lua with MainMenuBar_SetMaxLevelBarShown"
        );

        std::fs::remove_dir_all(root).expect("remove cache root");
    }

    #[test]
    #[cfg(feature = "client-mists")]
    fn mists_rejects_main_menu_bar_xml_with_legacy_exp_bar_scripts() {
        let root = unique_temp_dir("mists-main-menu-bar-xml");
        std::fs::create_dir_all(&root).expect("create cache root");
        let main_menu_bar_xml = root.join("MainMenuBar.xml");

        std::fs::write(
            &main_menu_bar_xml,
            r#"<Ui><Frame name="MainMenuBarMaxLevelBar"/><OnLoad>ExpBar_Update()</OnLoad></Ui>"#,
        )
        .expect("write stale MainMenuBar.xml");
        assert!(
            !cache_entry_is_usable(
                "Blizzard_ActionBar/Classic/MainMenuBar.xml",
                &main_menu_bar_xml
            ),
            "Mists 5.5.4 cache must reject MainMenuBar.xml with old ExpBar_Update hooks"
        );

        std::fs::write(
            &main_menu_bar_xml,
            r#"<Ui><Frame name="MainMenuBarMaxLevelBar"/></Ui>"#,
        )
        .expect("write current MainMenuBar.xml");
        assert!(
            cache_entry_is_usable(
                "Blizzard_ActionBar/Classic/MainMenuBar.xml",
                &main_menu_bar_xml
            ),
            "Mists 5.5.4 cache should accept MainMenuBar.xml without legacy ExpBar script hooks"
        );

        std::fs::remove_dir_all(root).expect("remove cache root");
    }
}
