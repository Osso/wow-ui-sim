type BootstrapFallbackOwner = (&'static str, &'static str);

const RECENTLY_MOVED_BOOTSTRAP_FALLBACKS: &[BootstrapFallbackOwner] = &[
        ("function ReloadUI", "Rust event API"),
        ("AlertFrame =", "temporary AlertFrame workaround"),
        (
            "__wow_register_alert_frame",
            "temporary AlertFrame workaround",
        ),
        (
            "AddQueuedAlertFrameSubSystem",
            "temporary AlertFrame workaround",
        ),
        (
            "__wow_register_catalog_shop_inbound_globals",
            "temporary catalog-shop inbound workaround",
        ),
        (
            "CatalogShopInboundInterface",
            "temporary catalog-shop inbound workaround",
        ),
        (
            "SimpleCheckoutInboundInterface",
            "temporary catalog-shop inbound workaround",
        ),
        ("HasArtifactEquipped", "temporary inert-global workaround"),
        ("IsPVPTimerRunning", "temporary inert-global workaround"),
        (
            "GetAlternativeDefaultLanguage",
            "temporary inert-global workaround",
        ),
        ("UI_SPECIAL_FRAMES", "Rust global table surface"),
        ("UISpecialFrames =", "Rust global table surface"),
        ("StaticPopup_Show", "temporary StaticPopup workaround"),
        ("StaticPopup_Hide", "temporary StaticPopup workaround"),
        (
            "StaticPopup_AddShowCondition",
            "temporary StaticPopup workaround",
        ),
        (
            "StaticPopupDialogs = StaticPopupDialogs or {}",
            "Rust global table surface",
        ),
        (
            "__wow_ensure_glue_character_select_surface",
            "temporary client-info workaround",
        ),
        (
            "__wow_ensure_spellbook_surface",
            "temporary legacy spell workaround",
        ),
        (
            "function RegisterEventCallback",
            "temporary dispatcher callback workaround",
        ),
        (
            "function UnregisterEventCallback",
            "temporary dispatcher callback workaround",
        ),
        (
            "function RegisterUnitEventCallback",
            "temporary dispatcher callback workaround",
        ),
        (
            "function UnregisterUnitEventCallback",
            "temporary dispatcher callback workaround",
        ),
        (
            "function DevTools_AddMessageHandler",
            "temporary dispatcher callback workaround",
        ),
        (
            "__wow_ensure_dispatcher_surface",
            "temporary Dispatcher surface workaround",
        ),
        ("DISPATCHER_VERSION = 2.0", "temporary Dispatcher surface workaround"),
        ("Dispatcher = dispatcher", "temporary Dispatcher surface workaround"),
        (
            "function GetSpecializationInfoForSpecID",
            "temporary glue character-select workaround",
        ),
        (
            "function GetCharacterUndeleteStatus",
            "temporary glue character-select workaround",
        ),
        (
            "function IsCharacterTimerunning",
            "temporary glue character-select workaround",
        ),
        (
            "function ShouldShowExpansionUpgradeBanner",
            "temporary glue character-select workaround",
        ),
        (
            "function GetCharacterListGroupsInfo",
            "temporary glue character-select workaround",
        ),
        (
            "function CanAutoSetGamePadCursorControl",
            "temporary gamepad cursor-control workaround",
        ),
        (
            "function SetGamePadCursorControl",
            "temporary gamepad cursor-control workaround",
        ),
        ("ChatTypeInfo =", "temporary chat-window workaround"),
        ("function GetChatTypeIndex", "temporary chat-window workaround"),
        ("function GetChatWindowInfo", "temporary chat-window workaround"),
        ("function SetChatWindowShown", "temporary chat-window workaround"),
        (
            "function GetChatWindowSavedDimensions",
            "temporary chat-window workaround",
        ),
        (
            "function SetChatWindowSavedDimensions",
            "temporary chat-window workaround",
        ),
        (
            "function GetChatWindowSavedPosition",
            "temporary chat-window workaround",
        ),
        (
            "function SetChatWindowSavedPosition",
            "temporary chat-window workaround",
        ),
        (
            "function GetCameraFOVDefaults",
            "temporary camera/tutorial workaround",
        ),
        (
            "function GetTutorialsEnabled",
            "temporary camera/tutorial workaround",
        ),
        ("MoveViewOutStart", "temporary camera/tutorial workaround"),
        ("MoveViewDownStop", "temporary camera/tutorial workaround"),
        (
            "function GetPVPLifetimeStats",
            "temporary difficulty/PVP utility workaround",
        ),
        (
            "function GetModifiedClick",
            "temporary modified-click settings workaround",
        ),
        (
            "function SetModifiedClick",
            "temporary modified-click settings workaround",
        ),
        (
            "function GetLFDRoleRestrictions",
            "temporary legacy LFG workaround",
        ),
        (
            "function GetLFGRoleShortageRewards",
            "temporary legacy LFG workaround",
        ),
        ("function UnitClass", "Rust unit query owner"),
        ("function UnitRace", "Rust unit state owner"),
        ("function UnitNameUnmodified", "Rust unit query owner"),
        ("function UnitSex", "Rust unit state owner"),
        ("function UnitIsDead", "Rust unit liveness owner"),
        (
            "function RegisterUIPanel",
            "temporary UIParent panel workaround",
        ),
        (
            "function CloseAllWindows",
            "temporary UIParent panel workaround",
        ),
        ("function GetScreenWidth", "Rust screen-size runtime owner"),
        ("function GetScreenHeight", "Rust screen-size runtime owner"),
        (
            "function GetPhysicalScreenSize",
            "Rust screen-size runtime owner",
        ),
        (
            "function GetNumLanguages",
            "temporary inert language workaround",
        ),
        (
            "function GetProfessionSkillLineID",
            "temporary trade-skill UI workaround",
        ),
        ("function UnitName", "Rust unit query owner"),
        ("function UnitGUID", "Rust unit metadata owner"),
        ("function UnitLevel", "Rust unit query owner"),
        ("function UnitEffectiveLevel", "Rust unit query owner"),
        ("function UnitIsGhost", "Rust unit liveness owner"),
        ("function UnitIsConnected", "Rust unit liveness owner"),
        (
            "function UnitAffectingCombat",
            "Rust unit combat owner",
        ),
        (
            "function GetNumShapeshiftForms",
            "Rust shapeshift owner",
        ),
        ("function GetShapeshiftForm", "Rust shapeshift owner"),
        (
            "function GetMaxPlayerLevel",
            "temporary client-info workaround",
        ),
        ("function PlayerHasToy", "Rust ToyBox owner"),
        (
            "function GetNumSpecializations",
            "Rust C_SpecializationInfo owner",
        ),
        (
            "function GetSpecializationInfoForClassID",
            "Rust C_SpecializationInfo owner",
        ),
        ("function GetTotemInfo", "temporary totem defaults workaround"),
        (
            "function GetQuestResetTime",
            "temporary GameTime/calendar workaround",
        ),
        (
            "function GetScenariosChoiceOrder",
            "Rust battlefield/LFG scenario owner",
        ),
        (
            "function GetNumRandomScenarios",
            "Rust battlefield/LFG scenario owner",
        ),
        (
            "function GetRandomScenarioInfo",
            "Rust battlefield/LFG scenario owner",
        ),
        (
            "function GetDifficultyInfo",
            "temporary difficulty/PVP utility workaround",
        ),
        ("function LocalizedClassList", "Rust class info owner"),
        (
            "function GetUnitTotalModifiedMaxHealthPercent",
            "Rust unit stats owner",
        ),
        (
            "function CombatLogAddFilter",
            "temporary combat-log state workaround",
        ),
        (
            "function CombatLogResetFilter",
            "temporary combat-log state workaround",
        ),
        (
            "function CombatLogAdvanceEntry",
            "temporary combat-log state workaround",
        ),
        (
            "function CombatLogGetCurrentEntry",
            "temporary combat-log state workaround",
        ),
        (
            "function CombatLogGetCurrentEventInfo",
            "temporary combat-log state workaround",
        ),
        (
            "function CombatLogGetNumEntries",
            "temporary combat-log state workaround",
        ),
        (
            "function CombatLogSetCurrentEntry",
            "temporary combat-log state workaround",
        ),
        (
            "function CombatLogShowCurrentEntry",
            "temporary combat-log state workaround",
        ),
        (
            "function CombatLogClearEntries",
            "temporary combat-log state workaround",
        ),
        (
            "function CombatLogSetRetentionTime",
            "temporary combat-log state workaround",
        ),
        (
            "function CombatLogGetRetentionTime",
            "temporary combat-log state workaround",
        ),
        (
            "function CombatLog_Object_IsA",
            "temporary combat-log state workaround",
        ),
        ("function UnitIsHumanPlayer", "Rust unit probe owner"),
        ("function IsTargetLoose", "Rust targeting owner"),
        (
            "LE_TOKEN_REDEEM_TYPE_GAME_TIME",
            "enum missing-constants surface",
        ),
        (
            "LE_TOKEN_REDEEM_TYPE_BALANCE",
            "enum missing-constants surface",
        ),
        (
            "LE_TOKEN_RESULT_ERROR_BALANCE_NEAR_CAP",
            "enum missing-constants surface",
        ),
        (
            "function SetPortraitTexture",
            "temporary portrait texture workaround",
        ),
        (
            "function SetPortraitTextureFromCreatureDisplayID",
            "temporary portrait texture workaround",
        ),
        ("function issecure", "rilua taint runtime"),
        ("function GetItemID", "Rust C_Item owner"),
        ("__wow_extract_item_id", "Rust C_Item owner"),
        ("AuraUtil.AuraFilters", "Rust aura owner"),
        ("AuraUtil.CreateFilterString", "Rust aura owner"),
        ("AuraUtil.UnpackAuraData", "Rust aura owner"),
        ("AuraUtil.ForEachAura", "Rust aura owner"),
        ("AuraUtil.FindAura", "Rust aura owner"),
        ("AuraUtil.FindAuraByName", "Rust aura owner"),
        (
            "AuraUtil.GetAuraDataByAuraInstanceID",
            "Rust aura owner",
        ),
        ("function GetPlayerAuraBySpellID", "Rust aura owner"),
        ("function UnitBuff", "Rust aura owner"),
        ("function UnitDebuff", "Rust aura owner"),
        ("function UnitAura", "Rust aura owner"),
        ("function UnitPosition", "Rust movement/map-position owner"),
        ("function UnitIsPossessed", "Rust unit relationship owner"),
        ("LE_REALM_RELATION_SAME", "Rust unit relationship owner"),
        (
            "function UnitRealmRelationship",
            "Rust unit relationship owner",
        ),
        ("function UnitInPartyIsAI", "Rust unit relationship owner"),
        (
            "function UnitIsPVPFreeForAll",
            "Rust unit relationship owner",
        ),
        ("function UnitPhaseReason", "Rust unit relationship owner"),
        (
            "__wow_ensure_chat_voice_button_surface",
            "temporary chat voice button workaround",
        ),
        (
            "function __wow_apply_chat_voice_button_surface",
            "temporary chat voice button workaround",
        ),
        (
            "function C_Map.GetBestMapForUnit",
            "temporary map runtime workaround",
        ),
        (
            "function C_Map.GetFallbackWorldMapID",
            "temporary map runtime workaround",
        ),
        (
            "function C_Map.MapHasArt",
            "temporary map runtime workaround",
        ),
        (
            "function MapUtil.GetDisplayableMapForPlayer",
            "temporary map runtime workaround",
        ),
        (
            "function MapUtil.IsShadowlandsZoneMap",
            "temporary map runtime workaround",
        ),
        (
            "function GetIconForRole",
            "temporary legacy LFG workaround",
        ),
        (
            "function GetIconForRoleEnum",
            "temporary legacy LFG workaround",
        ),
        (
            "function UnitGroupRolesAssigned",
            "temporary legacy LFG workaround",
        ),
        (
            "function UnitGroupRolesAssignedEnum",
            "temporary legacy LFG workaround",
        ),
        (
            "function GetInventoryItemID",
            "Rust inventory state probe",
        ),
        (
            "function GetInventoryItemsForSlot",
            "temporary inventory query workaround",
        ),
        (
            "function GetChatWindowChannels",
            "Rust chat-window state probe",
        ),
        (
            "function IsInventoryItemLocked",
            "Rust inventory state probe",
        ),
        (
            "ContentTrackingUtil = {}",
            "temporary content tracking workaround",
        ),
        (
            "function ContentTrackingUtil.IsContentTrackingEnabled",
            "temporary content tracking workaround",
        ),
        (
            "function ContentTrackingUtil.MakeCombinedID",
            "temporary content tracking workaround",
        ),
        (
            "function UnitHasVehiclePlayerFrameUI",
            "Rust global false stub",
        ),
        (
            "function UnitInVehicle",
            "Rust vehicle possession state",
        ),
        (
            "function UnitGetAvailableRoles",
            "temporary legacy LFG workaround",
        ),
        (
            "function IsTutorialFlagged",
            "temporary camera/tutorial workaround",
        ),
        (
            "function GetDungeonDifficultyID",
            "temporary difficulty/PVP utility workaround",
        ),
        ("function UnitThreatSituation", "Rust unit threat surface"),
        (
            "function UnitDetailedThreatSituation",
            "temporary unit threat workaround",
        ),
        (
            "function UnitThreatPercentageOfLead",
            "temporary unit threat workaround",
        ),
        ("function GetSendMailPrice", "Rust mail verb surface"),
        ("function GetMerchantFilter", "temporary merchant filter state"),
        ("function SetMerchantFilter", "temporary merchant filter state"),
        (
            "function AbbreviateNumbers",
            "temporary formatting utility workaround",
        ),
        ("function BNGetInfo", "temporary Battle.net account workaround"),
        (
            "function GetLFGDeserterExpiration",
            "temporary legacy LFG workaround",
        ),
        ("function UnitStagger", "temporary unit stagger workaround"),
        ("function GetPossessInfo", "temporary possess info workaround"),
        (
            "function StoreSecureReference",
            "temporary secure reference workaround",
        ),
        ("function IsInJailersTower", "Rust Torghast state surface"),
        (
            "function IsInventoryItemProfessionBag",
            "temporary inventory query workaround",
        ),
        (
            "function SetItemButtonTexture",
            "temporary item-button helper workaround",
        ),
        (
            "function SetItemButtonCount",
            "temporary item-button helper workaround",
        ),
        (
            "function SetItemButtonTextureVertexColor",
            "temporary item-button helper workaround",
        ),
        (
            "function SetItemButtonNormalTextureVertexColor",
            "temporary item-button helper workaround",
        ),
        (
            "TooltipDataProcessor",
            "temporary tooltip data processor workaround",
        ),
        (
            "function AddTooltipDataAccessor",
            "temporary tooltip data processor workaround",
        ),
        (
            "UIWidgetManager = UIWidgetManager",
            "temporary UI widget manager workaround",
        ),
        (
            "RegisterWidgetVisTypeTemplate = __wow_noop",
            "temporary UI widget manager workaround",
        ),
        ("Settings = Settings", "temporary Settings surface workaround"),
        (
            "function InterfaceOptions_AddCategory",
            "temporary Settings surface workaround",
        ),
        (
            "function Settings.RegisterCanvasLayoutCategory",
            "temporary Settings surface workaround",
        ),
        (
            "function GetInventoryItemTexture",
            "Rust inventory probe surface",
        ),
        (
            "function GetNumArenaOpponentSpecs",
            "temporary inert global workaround",
        ),
        ("SecureTypes = {}", "temporary secure types workaround"),
        (
            "SecureTypes.CreateSecureMap",
            "temporary secure types workaround",
        ),
        (
            "SecureTypes.CreateSecureArray",
            "temporary secure types workaround",
        ),
        (
            "__wow_securecall_accepts_names",
            "temporary secure types workaround",
        ),
        ("function GetActionInfo", "Rust action slot state surface"),
        ("function IsTrialAccount", "temporary client-info workaround"),
        (
            "function IsRestrictedAccount",
            "temporary client-info workaround",
        ),
        (
            "function IsVeteranTrialAccount",
            "temporary client-info workaround",
        ),
        (
            "function IsAccountSecured",
            "temporary client-info workaround",
        ),
        (
            "function GetFileStreamingStatus",
            "temporary client-info workaround",
        ),
        (
            "function GetBackgroundLoadingStatus",
            "temporary client-info workaround",
        ),
        ("function GetWebTicket", "temporary client-info workaround"),
        ("PlayerLocation = {}", "temporary PlayerLocation workaround"),
        (
            "function PlayerLocation:CreateFromGUID",
            "temporary PlayerLocation workaround",
        ),
        (
            "function PlayerLocation:CreateFromUnit",
            "temporary PlayerLocation workaround",
        ),
        (
            "function PlayerLocation:CreateFromCommunityChatData",
            "temporary PlayerLocation workaround",
        ),
        (
            "function PlayerLocation:CreateFromBattleNetID",
            "temporary PlayerLocation workaround",
        ),
        (
            "function PlayerLocation:CreateFromVoiceID",
            "temporary PlayerLocation workaround",
        ),
        (
            "function ChatFrameUtil.ProcessMessageEventFilters",
            "temporary chat-window workaround",
        ),
        (
            "function ChatFrameUtil.GetChatWindowName",
            "temporary chat-window workaround",
        ),
        (
            "function ChatFrameUtil.GetCommunitiesChannelColor",
            "temporary chat-window workaround",
        ),
        (
            "function ChatFrameUtil.GetCommunitiesChannelLocalID",
            "temporary chat-window workaround",
        ),
        ("function GetChannelName", "Rust channel state surface"),
        (
            "function GetCurrentEnvironment",
            "temporary debug/environment workaround",
        ),
        (
            "function SwapToGlobalEnvironment",
            "temporary debug/environment workaround",
        ),
        (
            "function CreateSecureDelegate",
            "Rust secure helpers owner",
        ),
        (
            "function debug.getfenv",
            "temporary debug/environment workaround",
        ),
        (
            "function GetContainerNumSlots",
            "Rust C_Container owner",
        ),
        ("function GetContainerItemID", "Rust C_Container owner"),
        ("function GetContainerItemLink", "Rust C_Container owner"),
        (
            "function GetTradeSkillTexture",
            "temporary trade-skill UI workaround",
        ),
        ("function IsArtifactRelicItem", "Rust item socket owner"),
        ("if abs == nil", "globals compat-overrides owner"),
        ("if ceil == nil", "globals compat-overrides owner"),
        ("if floor == nil", "globals compat-overrides owner"),
        ("if max == nil", "globals compat-overrides owner"),
        ("if min == nil", "globals compat-overrides owner"),
        ("if strlen == nil", "globals compat-overrides owner"),
        ("if sort == nil", "globals compat-overrides owner"),
        (
            "ActionButtonSpellAlertManager =",
            "temporary legacy action-bar workaround",
        ),
        (
            "__wow_action_button_alert_fields",
            "temporary legacy action-bar workaround",
        ),
        ("if bit == nil", "globals compat-overrides owner"),
        ("bit = {", "globals compat-overrides owner"),
        (
            "TextureKitConstants =",
            "temporary shared XML utility workaround",
        ),
        (
            "EditModeAccountSettingsMixin =",
            "temporary Settings/EditMode workaround",
        ),
        (
            "BaseActionButtonMixin =",
            "temporary legacy action-bar workaround",
        ),
        (
            "GameRulesUtil =",
            "temporary game-rules namespace workaround",
        ),
        (
            "function GameRulesUtil.ShouldShowPlayerCastBar",
            "temporary game-rules namespace workaround",
        ),
        (
            "EVERY_X_PERCENT =",
            "temporary formatting utility workaround",
        ),
        (
            "TRANSMOGRIFY_TOOLTIP_APPEARANCE_KNOWN =",
            "temporary formatting utility workaround",
        ),
        (
            "ERR_QUEST_SESSION_RESULT_RESYNC =",
            "temporary formatting utility workaround",
        ),
        (
            "CLASS_SORT_ORDER =",
            "temporary formatting utility workaround",
        ),
        (
            "EVENT_TOAST_MANAGER_OFFSET_Y_OVERRIDE =",
            "temporary inert-global workaround",
        ),
        (
            "CLOCK_TICKER_Y_OVERRIDE =",
            "temporary inert-global workaround",
        ),
        ("ChatTypeGroup =", "Rust chat-frame utility owner"),
        (
            "PartyMemberFramePool =",
            "temporary pool constructor workaround",
        ),
        (
            "ContainerFrameContainer =",
            "temporary legacy container workaround",
        ),
        ("ChatFrame1", "temporary chat-window workaround"),
        ("SettingsPanel", "temporary Settings surface workaround"),
        ("AccessibilityFontPreview", "temporary Settings surface workaround"),
        ("QuestTextPreview", "temporary Settings surface workaround"),
        ("LFGListFrame", "temporary legacy LFG workaround"),
        ("SearchPanel", "temporary legacy LFG workaround"),
        (
            "EventToastManagerFrame =",
            "temporary global-frame workaround",
        ),
        (
            "EditModeManagerFrame =",
            "temporary global-frame workaround",
        ),
        ("RolePollPopup =", "temporary global-frame workaround"),
        ("TimerTracker =", "temporary global-frame workaround"),
        ("UIErrorsFrame =", "temporary global-frame workaround"),
        ("SideDressUpFrame =", "temporary global-frame workaround"),
        ("LootFrame =", "temporary global-frame workaround"),
        ("RaidWarningFrame =", "temporary global-frame workaround"),
        ("GossipFrame =", "temporary global-frame workaround"),
        ("FriendsFrame =", "temporary global-frame workaround"),
        ("HelpFrame =", "temporary global-frame workaround"),
        ("BuffFrame =", "temporary global-frame workaround"),
        ("AuraContainer.iconScale", "temporary global-frame workaround"),
        (
            "AddonCompartmentFrame =",
            "temporary addon-compartment workaround",
        ),
        (
            "__wow_register_addon_compartment",
            "temporary addon-compartment workaround",
        ),
];

#[test]
fn recently_moved_startup_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");
    let shared_bootstrap = include_str!("../src/lua_api/env_init/shared_bootstrap.lua");

    for &(needle, owner) in RECENTLY_MOVED_BOOTSTRAP_FALLBACKS {
        assert!(
            !bootstrap.contains(needle),
            "{needle} fallback must live in the explicit {owner}, not runtime bootstrap"
        );
        assert!(
            !shared_bootstrap.contains(needle),
            "{needle} fallback must live in the explicit {owner}, not shared bootstrap"
        );
    }
}
