# WoW UI Simulator - Plan

## GTK4 Migration

### Why Migrate from iced

The current iced canvas renderer has fundamental limitations:
- No text measurement APIs - centering text requires guessing character widths
- Manual layout calculations for everything
- No proper widget system - reimplementing basic UI primitives
- Fighting the framework instead of leveraging it

GTK4/relm4 provides:
- Proper text rendering with Pango (measurement, wrapping, ellipsis)
- Layout containers (Box, Grid, Overlay) that handle positioning
- Real widget system with CSS styling
- Same Elm-like architecture as iced (via relm4)

### Migration Strategy

**Phase 1: Scaffold GTK App** ✅
- [x] Add gtk4, relm4, libadwaita dependencies
- [x] Create basic window with relm4 Application
- [x] Port console/command input (bottom panel)
- [x] Port event buttons (ADDON_LOADED, etc.)
- [x] Port frames list sidebar
- [x] Integrate gtk-layout-inspector for debugging
- [x] Add WoW-style CSS theming
- [x] Cairo rendering for WoW frames

**Phase 2: WoW UI Canvas** ✅
- [x] Create custom GtkDrawingArea for WoW frame rendering
- [x] Port nine-slice texture rendering to Cairo
- [x] Port texture/image rendering
- [x] Implement proper text rendering with Pango
- [x] Mouse event handling (hover, click)

**Phase 3: Widget Mapping** ✅
- [x] Map WoW Button → GTK rendering with proper text centering
- [x] Map WoW FontString → Pango text layout (with justify_h/justify_v support)
- [x] Map WoW Frame → Cairo rectangle with backdrop
- [x] Map WoW Texture → Cairo image surface
- [x] Map WoW EditBox → Cairo text input field rendering

**Phase 4: Cleanup** ✅
- [x] Remove iced dependencies
- [x] Remove render/ui.rs (1400 lines of manual layout)
- [x] Remove render/nine_slice.rs
- [x] gtk-layout-inspector integrated

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│ relm4 Application                                       │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────┐ ┌─────────────────┐ │
│ │ WoW Canvas (GtkDrawingArea)     │ │ Frames Sidebar  │ │
│ │ - Cairo rendering               │ │ - GtkListView   │ │
│ │ - Pango text                    │ │                 │ │
│ │ - Mouse events                  │ │                 │ │
│ └─────────────────────────────────┘ └─────────────────┘ │
├─────────────────────────────────────────────────────────┤
│ Event Buttons (GtkBox with GtkButtons)                  │
├─────────────────────────────────────────────────────────┤
│ Command Input (GtkEntry) │ Console Output (GtkTextView) │
└─────────────────────────────────────────────────────────┘
```

### Files to Modify

| Current (iced) | New (GTK4) |
|----------------|------------|
| src/render/ui.rs (1400 lines) | src/gtk_app.rs - relm4 component |
| src/render/nine_slice.rs | src/render/cairo_nine_slice.rs |
| src/main.rs | src/main.rs - gtk4 app init |
| Cargo.toml (iced deps) | Cargo.toml (gtk4, relm4, adw) |

### Keep Unchanged

These modules are renderer-agnostic and stay the same:
- `src/lua_api/` - Lua bindings (mlua)
- `src/loader.rs` - Addon loading
- `src/widget/` - Frame tree data structures
- `src/xml/` - XML parsing
- `src/toc.rs` - TOC file parsing
- `src/saved_variables.rs` - SavedVariables persistence
- `src/event/` - Event system

### Dependencies

```toml
[dependencies]
# Remove
# iced = ...
# iced_runtime = ...
# iced-layout-inspector = ...

# Add
relm4 = { version = "0.10", features = ["libadwaita"] }
gtk = { version = "0.10", package = "gtk4" }
adw = { version = "0.8", package = "libadwaita" }
gtk-layout-inspector = { path = "...", features = ["server"] }
```

### Reference Implementation

See `/home/osso/Projects/apps/reddit-desktop-gtk` for relm4 patterns:
- TypedListView for frame list
- CSS styling
- gtk-layout-inspector integration
- Async operations with tokio

---

## Current State

- Basic rendering with WoW-style textures and 9-slice
- Mouse interaction (hover, click, OnEnter/OnLeave/OnClick)
- Event system (ADDON_LOADED, PLAYER_LOGIN, PLAYER_ENTERING_WORLD)
- Saved variables persistence (JSON in ~/.local/share/wow-ui-sim/)
- Loading real addons: Ace3, SharedXML, GameMenu, WeakAuras, DBM, Details, Plater, AllTheThings

### Load Statistics (2026-01-18 - Updated v21)
**Overall: 127/127 addons, 120+ with 0 warnings, ~7 with warnings, 19 total warnings**

Current session (v21):
- Fixed $parent/$Parent substitution in XML anchors (relativeTo and relativeKey)
- Previously only `$parent_Foo` was handled, now `$parentFoo` and `$parent` as relativeKey also work
- Added support for chained $parent (e.g., `$parent.$parent.ScrollFrame` → `parent:GetParent()["ScrollFrame"]`)
- Added case-insensitive file resolution for TOC files and XML includes (Windows/macOS compatibility)
- OmniCD: 5 → 4 warnings, 51 → 62 Lua files (+11), now loads locales correctly
- Many addons with case-sensitivity issues now load properly
- Total: 39 → 19 warnings (-20 warnings, -51%)
- Lua files: 5158 → 5399 (+241 files)
- XML files: 296 → 309 (+13 files)

Previous session (v20):
- Added POWER_TYPE_ESSENCE, UNIT_NAME_FRIENDLY_TOTEMS, HUD_EDIT_MODE_EXPERIENCE_BAR_LABEL
- Added HUD_EDIT_MODE_HUD_TOOLTIP_LABEL, HUD_EDIT_MODE_TIMER_BARS_LABEL
- Added BAG_NAME_BACKPACK, BAGSLOTTEXT, LOSS_OF_CONTROL, COOLDOWN_VIEWER_LABEL
- Added BINDING_HEADER_HOUSING_SYSTEM
- EditModeExpanded: 1 warning → 0 warnings (89 Lua files)
- Total: 34 → 32 warnings (-2 warnings, -6%)
- Addons with 0 warnings: 108 → 110 (+2)
- Lua files: 5256 → 5271 (+15 files)
- Remaining 32 warnings breakdown:
  - ~12 missing files (addon packaging issues - IO error: No such file)
  - ~20 addon-internal issues (dependencies, internal methods, cascading failures)

Previous session (v19):
- Added TaggableObjectMixin and MapCanvasPinMixin (map canvas pin support)
- Added LFGEventFrame global frame (LFG event handling)
- TomCats: 1 warning → 0 warnings, 0 Lua → 103 Lua files (+103 files)
- Added NamePlateDriverFrame, UIErrorsFrame, InterfaceOptionsFrame, AuctionHouseFrame, SideDressUpFrame
- Added C_AddOnProfiler namespace with GetAddOnMetric, GetOverallMetric
- Added C_CurveUtil namespace with CreateCurve, CreateColorCurve
- Added Enum.AddOnProfilerMetric (SessionAverageTime, RecentAverageTime, etc.)
- **Switched from LuaJIT to Lua 5.1** - fixes 65k constant limit and Korean escape sequences
- BetterWardrobe: 2 warnings → 0 warnings, 77 → 79 Lua files (ColorFilter.lua now loads)
- AllTheThings: 2 warnings → 1 warning, 93 → 119 Lua files (Categories/Instances.lua now loads)
- Cell: 1 warning → 0 warnings, 102 → 113 Lua files (Korean locale now loads)
- Total: 36 → 34 warnings (-2 warnings, -6%)
- Addons with 0 warnings: 108 → 108 (some gained warnings from newly exposed issues)
- Lua files: 5130 → 5256 (+126 files)
- XML files: 300 → 302 (+2 files)

Previous session (v18):
- Added TRACKER_HEADER_PROVINGGROUNDS, TRACKER_HEADER_DUNGEON, TRACKER_HEADER_DELVES, etc.
- Added OBJECTIVES_WATCH_TOO_MANY and OBJECTIVES_TRACKER_LABEL constants
- Added DifficultyUtil.ID constants (DungeonNormal, DungeonHeroic, RaidLFR, etc.)
- Added C_CampaignInfo namespace with GetCampaignInfo, IsCampaignQuest
- Fixed C_GossipInfo.GetFriendshipReputation to return proper table (was returning nil)
- !KalielsTracker: 1 warning → 0 warnings, 84 Lua → 95 Lua files (+11 files, +1 XML)
- Fixed loader.rs to use lossy UTF-8 conversion (from_utf8_lossy) for invalid encoding
- Added POIButtonMixin and Menu/MenuUtil globals
- Added FriendsFrame global frame
- Added ERR_FRIEND_OFFLINE_S and other friend error strings
- Added Enum.ItemMiscellaneousSubclass
- Added C_QuestLog.GetTitleForQuestID, GetQuestTagInfo
- Added Menu.ModifyMenu
- Added PartyMemberFramePool
- ClickableRaidBuffs: 1 warning → 0 warnings (+1 Lua file)
- OribosExchange: 4 warnings → 0 warnings (+4 Lua files)
- GlobalIgnoreList: 4 warnings → 0 warnings (+4 Lua files)
- Total: 46 → 36 warnings (-10 warnings, -22%)
- Addons with 0 warnings: 105 → 108 (+3)
- Lua files: 5110 → 5130 (+20 files)
- XML files: 299 → 300 (+1 file)

Previous session (v17):
- Added OBJECTIVE_TRACKER_BLOCK_HEADER_COLOR and QUEST_OBJECTIVE_FONT_COLOR constants
- Fixed XML path resolution with fallback to addon root directory (fixes Angleur translations.xml)
- Added PlumberSettingsPanelLayoutTemplate handler (creates FrameContainer with child frames)
- Added registeredAddons table to AddonCompartmentFrame
- Added ObjectiveTrackerManager stub (used by !KalielsTracker)
- Added LIGHTGRAY_FONT_COLOR constant
- Added Enum.ContentTrackingTargetType, Enum.QuestRewardContextFlags, Enum.HousingPlotOwnerType
- Added C_QuestLog.GetMaxNumQuestsCanAccept
- Added StaticPopup1-4 frames with EditBox children
- Angleur: 6 warnings → 0 warnings (path resolution fallback)
- Plumber: 2 warnings → 1 warning (template handler + registeredAddons)
- !KalielsTracker: 3 warnings → 1 warning, 51 Lua → 84 Lua files (+33 files)
- Total: 57 → 48 warnings (-9 warnings, -16%)

Previous session (v16):
- Added Enum.QuestTag.Delve (288) - fixes !KalielsTracker Constants.lua
- No more simple API additions found - remaining issues are addon-internal or dependency-related

Previous session v15: 103 with 0 warnings, 24 with warnings, 57 warnings
Previous session v14: 100 with 0 warnings, 27 with warnings, 62 warnings
Previous session v13: 99 with 0 warnings, 28 with warnings, 64 warnings
Previous session v12: 80 with 0 warnings, 47 with warnings, 93 warnings
Previous session v11: 79 with 0 warnings, 48 with warnings, 99 warnings
Previous session v10: 74 with 0 warnings, 44 with warnings
Previous session v9: 73 with 0 warnings, 45 with warnings
Previous session v8: 72 with 0 warnings, 46 with warnings
Previous session v7: 70 with 0 warnings, 48 with warnings
Previous session v6: ~110 warnings
Previous session v5: 4466 Lua, 271 XML, 130 warnings
Previous session v4: 4460 Lua, 270 XML, 136 warnings
Previous session v3: 4459 Lua, 270 XML, 137 warnings
Previous session v2: 4377 Lua, 268 XML, 147 warnings
Previous session v1: 4271 Lua, 264 XML, 179 warnings

Key addon improvements this session (v12):
- Auctionator: 341 Lua, 14 XML, 0 warnings ✅ (was 2 warnings - Enum.ItemQuality.Good, Enum.ItemClass.Questitem)
- AllTheThings: 93 Lua, 0 XML, 2 warnings (was 4 warnings - fixed TooltipDataType enum sync)
- Angleur: 26 Lua, 6 XML, 6 warnings (was 8 warnings - fixed $parent_Sibling anchor references)
- DialogueUI: 97 Lua, 13 XML, 0 warnings ✅ (was showing warnings before)

Bug fixes:
- Fixed $parent_ sibling references in anchors (was generating invalid Lua with literal $)
- Synced TooltipDataType enum between two definitions (second was overwriting first with different values)
- Added Enum.ItemClass.Questitem alias for Quest (value 12)
- Added Enum.ItemQuality.Good alias for Uncommon (value 2) in both definitions

Previous session (v11):
- WorldQuestTracker: 123 Lua, 3 XML, 0 warnings ✅ (was 3 warnings - QuestFrame panels/buttons, overlayFrames, LFGListFrame.SearchPanel)

Frame/API additions (v11):
- ObjectiveTrackerFrame.Header.MinimizeButton structure
- LFGListFrame.SearchPanel.SearchBox structure
- Background texture to ObjectiveTrackerContainerHeaderTemplate
- WorldMapFrame.overlayFrames table
- QuestFrameRewardPanel, QuestFrameDetailPanel, QuestFrameProgressPanel
- QuestFrameAcceptButton, QuestFrameCompleteButton, QuestFrameCompleteQuestButton

Previous session (v10):
- Chattynator: 36 Lua, 2 XML, 0 warnings ✅ (was 2 warnings - ChatTypeGroup, ChatFrameUtil, loot/chat message constants)
- ExtraQuestButton: 28 Lua, 2 XML, 1 warning (just directory listing) (was 2 warnings - EditModeSystemSelectionTemplate with Label/Selection, SetOnClickHandler)
- DeModal: 6 Lua, 1 XML, 1 warning (just directory listing) (was 2 warnings - SettingsListTemplate, SettingsCheckBoxControlTemplate)

Templates added (v10):
- EditModeSystemSelectionTemplate - creates Label FontString and sets parent.Selection
- SettingsListTemplate - creates Header/ScrollBox hierarchy
- SettingsCheckBoxControlTemplate - creates Text/Checkbox children

Previous session (v9):
- HousingItemTracker: 3 Lua, 0 XML, 0 warnings ✅ (was 1 warning - SetDesaturated method on FrameHandle)

Previous session (v8):
- DynamicCam: 62 Lua, 1 XML, 0 warnings ✅ (was 2 warnings - CompactRaidFrameContainer, SettingsPanel.Container)
- DialogueUI: 97 Lua, 13 XML, 0 warnings ✅ (was 5 warnings - Enum.QuestTag, Enum.QuestCompleteSpellType, Font parsing, C_VoiceChat)

Previous session (v7):
- NameplateSCT: 52 Lua, 2 XML, 0 warnings ✅ (was failing - Enum.Damageclass)
- idTip: 1 Lua, 0 XML, 0 warnings ✅ (was failing - HasScript method)
- EditModeExpanded: 89 Lua, 7 XML, 0 warnings ✅ (was 4 warnings - Enum.EditModeSystem, MicroButtons, action bars)

Previous session (v6):
- Auctionator: 242 Lua, 12 XML, 2 warnings (was failing earlier - Enum.ItemBind, Enum.AuctionHouseFilter)
- Leatrix_Plus: 5 Lua, 1 XML, 0 warnings ✅ (was failing - SetFormattedText)
- Chattynator: 34 Lua, 2 XML, 2 warnings (was 3 warnings - CreateFontFamily, GetFontObjectForAlphabet)
- CooldownToGo_Options: 40 Lua, 2 XML, 1 warning (was failing - Settings.GetCategory)
- idTip: Loading now (was failing - TooltipDataProcessor.AddTooltipPostCall with nil)

Key addon improvements this session (v5):
- BetterWardrobe: 79 Lua, 19 XML, 0 warnings ✅ (was 4 warnings - Enum.TransmogSource, SetAutoDress)
- DeathNote: 60 Lua, 12 XML, 0 warnings ✅ (was 2 warnings - COMBATLOG_OBJECT_RAIDTARGET*, DUEL_WINNER_*)
- TomTom: 57 Lua, 8 XML, 1 warning (was 2 warnings - AddDataProvider method)
- Baganator: 108 Lua, 29 XML, 3 warnings (was 4 warnings - Enum.PlayerInteractionType.Merchant)

Previous session (v4):
- DragonRaceTimes: 10 Lua, 0 XML, 0 warnings ✅ (was failing - GossipFrame)
- AutoPotion: 24 Lua, 2 XML, 0 warnings ✅ (was failing - C_Spell.GetSpellCooldown)
- BigWigs_Plugins: 23 Lua, 0 XML, 0 warnings ✅ (was 22 Lua, 1 warning - RaidWarningFrame)
- BugSack: 13 Lua, 1 XML, 0 warnings ✅ (was failing - Settings.RegisterVerticalLayoutCategory)
- Dejunk: 73 Lua, 1 XML, 0 warnings ✅ (was failing - SetCountInvisibleLetters, EditBox methods)
- AdvancedInterfaceOptions: 43 Lua, 2 XML, 0 warnings ✅ (was failing - COMBAT_TEXT_SHOW_ENERGIZE_TEXT)
- WorldQuestTracker: 120 Lua, 3 XML, 3 warnings (was 4 warnings - WorldMapFrame.BorderFrame hierarchy)

Previous session (v2):
- RaiderIO: 31 Lua, 1 XML, 0 warnings (was 12 Lua, 1 warning - C_CreatureInfo.GetFactionInfo, Enum.LootSlotType, NEWS_* constants, WHO_LIST_* formats, GetTextWidth)
- RaiderIO_DB_*: All 12 database addons now load (was 2 warnings each)
- SimpleItemLevel: 5 Lua, 1 XML, 0 warnings (was 4 Lua, 1 warning - ContainerFrameContainer, LootFrame.ScrollBox, TooltipDataProcessor, Enum.TooltipDataType)
- !BugGrabber: 2 Lua, 0 warnings (was 1 Lua, 1 warning - seterrorhandler)
- BlizzMove_Debug: 2 Lua, 0 warnings (was 1 Lua, 1 warning - C_Console namespace)
- AngryKeystones: 9 Lua, 0 XML, 1 warning (was 7 Lua, 3 warnings - TOOLTIP_DEFAULT_COLOR, ScenarioObjectiveTracker)

Previous session (Jan 16):
- SavedInstances: 56 Lua, 0 warnings (was 53 Lua, 3 warnings - BOSS, GARRISON_*, ERR_RAID_DIFFICULTY_*, EJ_GetCreatureInfo, C_CurrencyInfo)
- TalentLoadoutManager: 49 Lua, 0 warnings (was 48 Lua, 1 warning - C_Traits.GetLoadoutSerializationVersion)
- Baganator: 105 Lua, 29 XML, 6 warnings (was 105 Lua, 28 XML, 7 warnings - ItemButton XML type, LINK_FONT_COLOR)
- Rarity: 170 Lua, 22 XML, 2 warnings (was 170 Lua, 2 warnings - SetTextCopyable, SetInsertMode, SetFading, AlertFrame)

Core addons (0 warnings):
- Ace3: 43 Lua, 15 XML ✅
- SharedXMLBase: 34 Lua, 2 XML ✅
- SharedXML: 155 Lua, 72 XML ✅
- GameMenu: 3 Lua, 2 XML ✅
- UIWidgets: 39 Lua, 38 XML ✅
- Details (+ plugins): 259 Lua, 5 XML ✅
- Plater: 151 Lua, 2 XML ✅
- BlizzMove: 49 Lua, 2 XML ✅
- EnhancedRaidFrames: 81 Lua, 3 XML ✅
- XIV_Databar_Continued: 132 Lua, 7 XML ✅
- SavedInstances: 56 Lua, 1 XML ✅ (NEW)
- TalentLoadoutManager: 49 Lua, 1 XML ✅ (NEW)

Key addons improvements:
- Cell: 93 Lua, 8 XML, 4 warnings
- Angleur: 26 Lua, 4 XML, 8 warnings
- Plumber: 128 Lua, 12 XML, 3 warnings
- BigWigs_Plugins: 22 Lua, 0 XML, 1 warning
- DynamicCam: 56 Lua, 1 XML, 6 warnings

APIs added this session (v10):
- **Frame methods**: SetOnClickHandler (WoW 10.0+ Edit Mode method), __len metamethod (for array-like child iteration)
- **Templates**: EditModeSystemSelectionTemplate (Label + parent.Selection), SettingsListTemplate (Header/ScrollBox), SettingsCheckBoxControlTemplate (Text/Checkbox)
- **Global strings**: LOOT_ITEM_* (12 loot message formats), CURRENCY_GAINED_*, COMBATLOG_XPGAIN_*, CHAT_*_GET (message formats), ACHIEVEMENT_BROADCAST
- **Global frames**: FriendsListFrame.ScrollBox child

APIs added previous session (v9):
- **Frame methods**: SetDesaturated/IsDesaturated on FrameHandle (for textures), RegisterUnitEvent accepts Value args (not just strings)
- **Addon table**: Default `unpack` method (returns values at indices 1-4, used by OmniCD pattern)

APIs added previous session (v8):
- **Global frames**: CompactRaidFrameContainer, StatusTrackingBarManager.bars table, SettingsPanel.Container hierarchy
- **Enums**: Enum.QuestTag (Dungeon/Raid/Raid10/Raid25/Scenario/Group/Heroic/PvP/Account/Legendary), Enum.QuestCompleteSpellType (Follower/Companion/Tradeskill/Ability/Aura/Spell)
- **C_ APIs**: C_VoiceChat (SpeakText/StopSpeakingText/IsSpeakingText/GetTtsVoices), C_TTSSettings (GetSpeechRate/SetSpeechRate/GetSpeechVolume/SetSpeechVolume)
- **XML loader**: Font element parsing with SetTextColor/GetFont/SetFont/CopyFontObject methods

APIs added previous session (v7):
- **Frame methods**: HasScript (check if frame supports a script handler)
- **Enums**: Enum.Damageclass (MaskPhysical/Holy/Fire/Nature/Frost/Shadow/Arcane), Enum.EditModeSystem (action bar/menu indices)
- **Global strings**: BUFFOPTIONS_LABEL, DEBUFFOPTIONS_LABEL, BUFFFRAME_LABEL, DEBUFFFRAME_LABEL
- **Global frames**: All MicroButtons (Character/Profession/PlayerSpells/Achievement/QuestLog/Guild/LFD/Collections/EJ/Store/Help/Housing), all action bars (MainActionBar, MultiBarBottomLeft/Right, MultiBarRight/Left, MultiBar5-8, StanceBar, PetActionBar, PossessBar, OverrideActionBar)
- **XML loader fix**: $parent placeholder replacement in frame/texture/fontstring names (fixes "$parent_Texture" syntax errors)

APIs added previous session (v6):
- **Font methods**: SetFormattedText (FontString), GetFontObjectForAlphabet (font objects), CopyFontObject (string support)
- **Global functions**: CreateFontFamily (font family creation with alphabet support), GetFontInfo (table input support)
- **Enums**: Enum.ItemBind (None/OnEquip/OnAcquire/OnUse/Quest/ToAccount), Enum.AuctionHouseFilter (quality/uncollected/upgrades)
- **Global strings**: FOCUS, TARGET, PLAYER, PET, PARTY, RAID, BOSS, ARENA, SHOW_TARGET_OF_TARGET_TEXT, HEALTH, MANA, etc.
- **Global frames**: QuestFrame, MainMenuMicroButton
- **Settings API**: Settings.GetCategory
- **Frame methods**: GetUnboundedStringWidth (FontString stub), SetCenterColor (NineSlice)
- **Tables**: ITEM_QUALITY_COLORS (quality index to r/g/b/hex), QuestDifficultyColors, QuestDifficultyHighlightColors
- **Tooltip methods**: TooltipDataProcessor.AddTooltipPostCall (optional data_type parameter)

APIs added this session (v5):
- **Enums**: Enum.TransmogSource (JournalEncounter/Quest/Vendor/etc), Enum.PlayerInteractionType.Merchant
- **Frame methods**: AddDataProvider, RemoveDataProvider (WorldMapFrame), SetAutoDress (Model)
- **Global strings**: DUEL_WINNER_KNOCKOUT, DUEL_WINNER_RETREAT, COMBATLOG_OBJECT_RAIDTARGET1-8

Previous session (v4):
- **Frame hierarchies**: WorldMapFrame.BorderFrame.MaximizeMinimizeFrame with MaximizeButton/MinimizeButton, WorldMapFrame.ScrollContainer, WorldMapFrame.pinPools table
- **Frame methods**: WrapScript, UnwrapScript, SetCountInvisibleLetters, GetCursorPosition, SetCursorPosition, HighlightText (EditBox)
- **Global frames**: GossipFrame, RaidWarningFrame
- **Global strings**: RETRIEVING_ITEM_INFO, RETRIEVING_DATA, COMBAT_TEXT_SHOW_ENERGIZE_TEXT
- **Color objects**: BLUE_FONT_COLOR, MIXED_TEXT_COLOR
- **API functions**: C_Spell.GetSpellCooldown, Settings.RegisterVerticalLayoutCategory, Settings.RegisterVerticalLayoutSubcategory
- **Global functions**: SecureHandlerSetFrameRef, SecureHandlerExecute, SecureHandlerWrapScript
- **Enums**: Enum.TransmogType (Appearance/Illusion), Enum.TransmogModification (Main/Secondary/None)
- **Utility globals**: TransmogUtil (GetTransmogLocation, CreateTransmogLocation, GetBestItemModifiedAppearanceID)

Previous session APIs (v2):
- **Frame methods**: GetTextWidth (EditBox), RegisterCallback, ForEachFrame, UnregisterCallback (ScrollBox mixin)
- **Global frames**: ContainerFrameContainer (with ContainerFrames table), ContainerFrameCombinedBags, LootFrame (with ScrollBox child), ScenarioObjectiveTracker
- **Enums**: Enum.TooltipDataType (Item/Spell/Unit/etc), Enum.LootSlotType (None/Item/Money/Currency)
- **Global strings**: WHO_LIST_FORMAT, WHO_LIST_GUILD_FORMAT, NEWS_* constants, DAY/HOUR/MINUTE/SECOND_ONELETTER_ABBR, SHOW_COMBAT_HEALING_*, COMBAT_TEXT_SHOW_*
- **Color objects**: TOOLTIP_DEFAULT_COLOR, TOOLTIP_DEFAULT_BACKGROUND_COLOR
- **API functions**: C_CreatureInfo.GetFactionInfo, C_Item.RequestLoadItemDataByID, seterrorhandler, ConsoleGetAllCommands
- **API namespaces**: C_Console (GetAllCommands, GetColorFromType), TooltipDataProcessor (AddTooltipPostCall)
- **Constants**: MAX_BOSS_FRAMES, MAX_PARTY_MEMBERS, MAX_RAID_MEMBERS

Previous session (Jan 16):
- **XML widget types**: ItemButton (for Baganator bag buttons)
- **Frame methods**: GetZoom/SetZoom (Minimap), GetCanvas (WorldMapFrame), SetTextCopyable, SetInsertMode, SetFading, SetFadeDuration, SetTimeVisible, AddQueuedAlertFrameSubSystem
- **Global frames**: LFGListFrame (with SearchPanel.ScrollFrame hierarchy), AlertFrame
- **Enums**: Enum.WorldQuestQuality, Enum.QuestTagType
- **Global strings**: INSTANCE_RESET_SUCCESS, ERR_RAID_DIFFICULTY_CHANGED_S, BOSS, GARRISON_LOCATION_TOOLTIP/SHIPYARD/MISSION_COMPLETE/FOLLOWER, BIND_TRADE_TIME_REMAINING, BIND_ON_PICKUP/EQUIP/USE, LINK_FONT_COLOR_CODE
- **Color objects**: LINK_FONT_COLOR, EPIC_PURPLE_COLOR
- **Color table**: OBJECTIVE_TRACKER_COLOR (with Header/HeaderHighlight/Normal/NormalHighlight/Complete/Failed)
- **API functions**: C_CurrencyInfo.GetCurrencyInfo (returns table with name/quantity/etc), EJ_GetCreatureInfo (returns stub name), C_Traits.GetLoadoutSerializationVersion, C_ClassTalents.GetLoadoutSerializationVersion
- **CVar support**: rotateMinimap, minimapZoom

Previous session APIs (Jan 16 early):
- **Addon name fix**: Loader now passes folder name (not Title metadata) as addon vararg, fixing AceLocale locale lookups
- **Frame methods**: SetParent (was missing from FrameHandle userdata)
- **Enum.TransmogCollectionType**: All 29 transmog slot/weapon categories
- **C_TransmogCollection**: GetNumMaxOutfits, GetOutfitInfo, GetAppearanceCameraID, GetCategoryAppearances
- **C_Housing**: GetHomeInfo, IsHomeOwner, GetNumPlacedFurniture (Delves housing)
- **C_DelvesUI**: GetCurrentDelvesSeasonNumber, GetFactionForDelve, GetDelvesForSeason, HasActiveDelve, GetDelveInfo
- **C_ToyBox**: GetToyInfo, IsToyUsable, GetNumToys, GetToyFromIndex, GetNumFilteredToys
- **LOCALIZED_CLASS_NAMES_MALE/FEMALE**: All 13 class name lookup tables

Previous session APIs:
- C_Calendar, C_CovenantCallings, C_WeeklyRewards namespaces
- DifficultyUtil, WeeklyRewardsUtil, ItemLocation utilities
- C_ContributionCollector namespace
- Enum.WeeklyRewardChestThresholdType
- Font path globals (UNIT_NAME_FONT_CHINESE, etc.)
- Font color codes (YELLOW_FONT_COLOR_CODE, etc.)
- RegisterStateDriver/UnregisterStateDriver
- SetFrameRef/GetFrameRef, SetPushedTextOffset methods
- Extended GetFontString stub with SetWordWrap, SetNonSpaceWrap
- DUNGEON_DIFFICULTY, PLAYER_DIFFICULTY strings
- ERR_LOOT_GONE, INSTANCE_SAVED, CURRENCY_GAINED strings
- ICON_LIST table for raid markers

**Note**: Remaining warnings are addon-internal issues (missing translation files, custom library methods, WoW-specific template syntax), not missing WoW API coverage.

**WTF SavedVariables Loading**: Added support for loading real WoW SavedVariables from WTF directory.
Configured for character "Haky" on "Burning Blade" realm.

### AllTheThings Progress
Reduced from 28 warnings to 2 by adding:
- Localization constants: WORLD, ZONE, SPECIAL, DUNGEONS, RAIDS, ARMOR, INVTYPE_*, MONTH_*, DUNGEON_FLOOR_*, ITEM_QUALITY*_DESC, etc.
- C_* APIs: C_MapExplorationInfo, C_MountJournal, C_PetBattles, C_TradeSkillUI, C_Heirloom
- Functions: GetAchievementInfo, CreateVector2D, GetTimePreciseSec, InterfaceOptions_AddCategory
- Frame methods: Model methods, FontString methods, EnableMouseWheel
- Frames: ItemRefTooltip, ItemRefShoppingTooltip1/2, ShoppingTooltip1/2

Remaining 2 warnings (addon-internal issues, not WoW API):
1. Settings - SetATTTooltip mixin method not applied (addon's own Mixin system)
2. RetrievingData.lua - string.find receiving nil (data flow issue)

### Session Additions (2025-01-16)
- **Template child elements**: CreateFrame now creates child elements for templates (e.g., UICheckButtonTemplate creates Text FontString)
- **Frame children_keys**: Frames can now store keyed child references accessible via __index
- **Methods added**: GetUnboundedStringWidth, SetHitRectInsets, GetHitRectInsets, SetGradient, SetDrawLayer, GetDrawLayer, InCombatLockdown, GetSize, ApplyBackdrop, AddMessage, AddMsg, SetAtlas, GetAtlas, SetTexelSnappingBias, GetTexelSnappingBias, SetTextureSliceMargins, GetTextureSliceMargins, SetTextureSliceMode, GetTextureSliceMode, ClearTextureSlice
- **Console height**: Increased from 100-140px to 160-200px
- **System functions**: IsMacClient, IsWindowsClient, IsLinuxClient, IsTestBuild, IsBetaBuild, IsPTRClient, IsTrialAccount, IsVeteranTrialAccount
- **Difficulty functions**: GetRaidDifficultyID, GetDungeonDifficultyID, SetRaidDifficultyID, SetDungeonDifficultyID
- **C_* APIs**: C_LFGInfo, C_NamePlate
- **Enums**: Enum.SpellBookSpellBank, Enum.SpellBookItemType
- **Constants**: TAXIROUTE_LINEFACTOR, Constants.TraitConsts
- **Global frames**: WorldFrame, DEFAULT_CHAT_FRAME, ChatFrame1
- **WTF SavedVariables**: Added WtfConfig struct and load_wtf_for_addon() for loading real WoW saved variables
- **C_ClassTalents**: InitializeViewLoadout, ViewLoadout, GetHeroTalentSpecsForClassSpec
- **C_Traits**: InitializeViewLoadout, GetTreeInfo, GetTreeNodes, GetTreeCurrencyInfo, GetAllTreeIDs, GetTraitSystemFlags
- **C_AddOns.GetAddOnInfo**: Now accepts both integer index and addon name string
- **GetDifficultyInfo**: Added difficulty IDs 21-81 for newer dungeons
- **C_EncodingUtil**: CompressString, DecompressString, EncodeBase64, DecodeBase64 (stub implementations)
- **Enum.CompressionMethod**: Deflate, Huffman values
- **Battle.net functions**: BNFeaturesEnabled, BNFeaturesEnabledAndConnected, BNConnected, BNGetFriendInfo, BNGetNumFriends, BNGetInfo
- **GetAutoCompleteRealms**: Fixed to return empty table instead of nil
- **Frame dragging**: SetMovable, IsMovable, SetResizable, IsResizable, SetClampedToScreen, IsClampedToScreen now track state in Frame struct
- **StartMoving/StopMovingOrSizing**: Now actually update frame.is_moving state
- **Font rendering**: Added wow_font_to_family() to map WoW font paths (FRIZQT, ARIALN, SKURRI, MORPHEUS) to system fonts
- **draw_pango_text_with_font**: New function that accepts font path and word_wrap parameters
- **Focus management**: SetFocus, ClearFocus, HasFocus now track focused_frame_id in SimState
- **GetCurrentKeyBoardFocus()**: Global function that returns currently focused frame
- **Text wrapping**: SetWordWrap/GetWordWrap properly track word_wrap field on frames
- **Pango text wrapping**: draw_pango_text_with_font uses pango::WrapMode::Word when enabled
- **UIDropDownMenu system**: Full dropdown menu implementation with:
  - Global constants: UIDROPDOWNMENU_MAXBUTTONS, UIDROPDOWNMENU_MAXLEVELS, UIDROPDOWNMENU_OPEN_MENU, etc.
  - Global frames: DropDownList1, DropDownList2, DropDownList3 with button children (DropDownListNButtonM)
  - Functions: UIDropDownMenu_Initialize, UIDropDownMenu_CreateInfo, UIDropDownMenu_AddButton, UIDropDownMenu_SetWidth, UIDropDownMenu_SetText, UIDropDownMenu_GetText, UIDropDownMenu_SetSelectedID/Value/Name, UIDropDownMenu_GetSelectedID/Value, UIDropDownMenu_Enable/DisableDropDown, UIDropDownMenu_Refresh, UIDropDownMenu_SetAnchor, UIDropDownMenu_SetInitializeFunction, UIDropDownMenu_JustifyText, UIDropDownMenu_SetFrameStrata, UIDropDownMenu_AddSeparator, UIDropDownMenu_AddSpace, UIDropDownMenu_GetCurrentDropDown, UIDropDownMenu_IsOpen, ToggleDropDownMenu, CloseDropDownMenus, UIDropDownMenu_HandleGlobalMouseEvent
- **loadstring()**: Added Lua 5.1 loadstring function for dynamic code compilation (used by DetailsFramework for method wrapping)
- **Frame methods added**: SetBlendMode, GetBlendMode, AdjustPointsOffset, SetAllPoints (added to getmetatable method list)
- **Animation methods added**: SetScaleFrom, SetScaleTo for Scale animations
- **SetUnit**: Changed to accept Option<String> to handle nil argument

### Session Additions (2025-01-17)
- **xpcall varargs support**: Custom xpcall implementation supporting Lua 5.2+ varargs syntax (critical for AceAddon's safecall function)
- **Frame methods added**: GetTextColor, GetBackdropColor, GetBackdropBorderColor, SetMaxBytes, GetMaxBytes, SetThumbTexture, GetThumbTexture, SetStepsPerPage, GetStepsPerPage, Enable, Disable
- **SetAllPoints**: Fixed to accept boolean argument (true=parent, false=no-op) in addition to frame references
- **Fixed AceAddon initialization**: The xpcall fix resolves "self is nil" errors during PLAYER_LOGIN/PLAYER_ENTERING_WORLD events
- **Custom type() function**: Returns "table" for FrameHandle userdata, matching WoW behavior where `type(frame) == "table"` checks pass (fixes DetailsFramework validation)
- **$parent name substitution**: CreateFrame, CreateTexture, CreateMaskTexture, CreateFontString now support $parent/$Parent name patterns
- **ScrollFrame template support**: FauxScrollFrameTemplate creates ScrollBar with ThumbTexture, ScrollUpButton, ScrollDownButton (each with Normal/Pushed/Disabled textures)
- **Enum.PowerType**: Added all power types (Mana, Rage, Focus, Energy, etc.)
- **UNITNAME_TITLE_* constants**: Added pet ownership display strings (PET, COMPANION, GUARDIAN, MINION, etc.)
- **PET_TYPE_* constants**: Added pet type strings (PET, DEMON, GHOUL, GUARDIAN, TOTEM, TREANT)
- **C_Spell.GetSpellInfo**: Now returns proper SpellInfo table (name, spellID, iconID, castTime, minRange, maxRange)
- **C_Spell.GetSpellCharges**: Returns charges table (currentCharges, maxCharges, cooldownStartTime, etc.)
- **C_Item methods**: Added GetItemNameByID, GetDetailedItemLevelInfo, IsItemBindToAccountUntilEquip, GetItemLink, GetItemQualityByID
- **Details warnings**: Reduced from 9 to 0
- **Custom rawget() function**: Returns nil for userdata instead of erroring (fixes SharedXML Dump.lua rawget check on FrameHandle)
- **SharedXML warnings**: Reduced from 1 to 0
- **GetGuildInfo**: Added global function returning nil when not in guild
- **C_PvP namespace**: Added GetZonePVPInfo, GetScoreInfo, IsWarModeDesired, IsWarModeActive, IsPVPMap, IsRatedMap, IsInBrawl
- **C_FriendList namespace**: Added GetNumFriends, GetNumOnlineFriends, GetFriendInfoByIndex, GetFriendInfoByName, IsFriend
- **C_Timer handle improvements**: NewTicker and NewTimer now pass the handle object to callbacks (fixes Plater/LibOpenRaid timer errors)
- **IsCancelled method**: Timer/ticker handles now have IsCancelled() method
- **Global GetSpellInfo**: Now returns proper multi-value tuple (name, rank, icon, castTime, minRange, maxRange, spellId, originalIcon)
- **GetNumClasses**: Returns 13 (number of playable classes in retail)
- **GetClassInfo**: Returns className, classFile, classID for all 13 classes
- **GetWeaponEnchantInfo**: Returns weapon enchant info (stub, no enchants)
- **C_Traits.GetConfigInfo**: Now returns stub table with empty treeIDs (prevents LibOpenRaid nil errors)
- **C_SpecializationInfo.GetAllSelectedPvpTalentIDs**: Returns empty array (no PvP talents)
- **AuraUtil namespace**: Added ForEachAura, FindAura, UnpackAuraData, FindAuraByName
- **SetTitle/GetTitle**: Frame methods for DefaultPanelTemplate frames
- **DefaultPanelTemplate support**: CreateFrame creates Bg texture and TitleText FontString children
- **PanelTabButtonTemplate support**: CreateFrame creates Text FontString child
- **DBM library stubs**: LibLatency, LibDurability, LibChatAnims with Register/Unregister callbacks

### Session Additions (2025-01-16)
- **Total: 126/126 addons loaded, 4472 Lua files, 274 XML files, 121 warnings** (down from 130 warnings)
- **Custom ipairs for frame iteration**: Frames now support ipairs() to iterate over children
- **Numeric frame indexing**: frame[1], frame[2] etc. now returns n-th child
- **GetChildren/GetNumChildren**: Proper methods to get child frames
- **GetStatusBarTexture/GetThumbTexture/GetCheckedTexture**: Return proper texture userdata
- **FontFamily XML element**: Added support for FontFamily in XML parser
- **Camera functions**: GetFramerate, GetCameraZoom, CameraZoomIn, CameraZoomOut
- **GetFonts**: Returns list of registered fonts
- **GetCVarDefault**: Returns default CVar values
- **Global frames**: QuestFrame, FriendsTooltip
- **SetRaidTargetIconTexture/SetPortraitToTexture/CooldownFrame_Set**: Texture helper functions
- **TEXT_MODE_A_STRING_* constants**: Combat text formatting
- **ObjectiveTrackerContainerHeaderTemplate**: Creates Text and MinimizeButton children
- **EnableKeyboard/IsKeyboardEnabled**: Keyboard input frame methods
- **ITEM_COOLDOWN_TIME**: Item cooldown format string

### Session Additions (2025-01-18)
- **Total: 126/126 addons loaded, 2932 Lua files, 220 XML files, 484 warnings** (down from 531 warnings)
- **secureexecuterange**: Added function for CallbackRegistry secure iteration
- **Chat functions**: GetChannelList, GetChannelName, GetNumDisplayChannels
- **Global frames**: EventToastManagerFrame, EditModeManagerFrame
- **C_VignetteInfo namespace**: GetVignettes, GetVignetteInfo, GetVignettePosition, GetVignetteGUID
- **C_AreaPoiInfo namespace**: GetAreaPOIInfo, GetAreaPOISecondsLeft, IsAreaPOITimed, GetAreaPOIForMap
- **C_PlayerChoice namespace**: Player choice popup system APIs
- **C_MajorFactions namespace**: Renown/Major Faction system APIs (DF+)
- **C_UIWidgetManager namespace**: UI widget visualization APIs
- **C_GossipInfo namespace**: NPC gossip/dialog system APIs
- **C_Scenario namespace**: Dungeon/scenario tracker APIs
- **Frame methods**: SetIgnoreParentScale, GetIgnoreParentScale, SetIgnoreParentAlpha, GetIgnoreParentAlpha
- **LFG error strings**: ERR_LFG_PROPOSAL_FAILED, ERR_LFG_PROPOSAL_DECLINED, ERR_LFG_ROLE_CHECK_FAILED, etc.
- **Plumber progress**: 7 Lua → 20 Lua files loading (13 more files)

## High Impact

### Fix Addon Errors
- [x] Details PlayerInfo.lua:852 - fixed with INVSLOT_* constants
- [x] Added C_GuildInfo, C_AlliedRaces, C_AuctionHouse, C_Bank, C_EncounterJournal, C_GMTicketInfo, C_GuildBank
- [x] Added STRING_SCHOOL_* spell school constants (35 values)
- [x] XML OnLoad script firing - frames from XML now fire OnLoad after full creation
- [x] UIWidget enums - added 40+ enum types for Blizzard_UIWidgets support
- [x] Custom getmetatable for frames - returns __index as table of methods for iteration
- [x] ColorSelect methods - SetColorRGB, GetColorRGB, SetColorHSV, GetColorHSV
- [x] CreateColor Lua implementation - proper mutable color objects with SetRGB/SetColorRGB
- [x] DBM external libraries - Downloaded/stubbed LibStub, CallbackHandler, LibDataBroker, LibDeflate, LibSerialize, LibCustomGlow, LibSpecialization, LibKeystone, LibDropDownMenu, ChatThrottleLib, LibSharedMedia, LibDBIcon, LibLatency, LibDurability, LibChatAnims
- [x] Copy addons from Windows server - 131 addons copied via claude-remote (276MB zip split into 28x10MB chunks)
- [x] Real WoW textures - Using ~/Repos/wow-ui-textures (110k+ pre-converted PNG files)

### More Frame Types
- [x] ScrollFrame - scrollable content areas
- [x] EditBox - text input fields
- [x] Slider - value sliders
- [x] CheckButton - checkboxes
- [x] StatusBar - progress/health bars
- [x] Cooldown - spell cooldown overlays
- [x] Model/PlayerModel - 3D model placeholders
- [x] ColorSelect - color picker placeholder
- [x] MessageFrame/SimpleHTML - text display areas
- [x] DropDownMenu - dropdown menus (UIDropDownMenu_* functions, DropDownList1-3 frames)

### Keyboard Input ✅
- [x] EditBox text entry
- [x] Keybinding system (SetBinding, GetBindingKey)
- [x] Focus management (SetFocus, ClearFocus, HasFocus, GetCurrentKeyBoardFocus)

## Visual Improvements

### Font Rendering ✅
- [x] FontString:SetFont() implementation (stores font path and size)
- [x] Font path mapping to WoW fonts (FRIZQT, ARIALN, MORPHEUS → Trajan Pro)
- [x] Text alignment (justify_h, justify_v)
- [x] Text wrapping (SetWordWrap/GetWordWrap with Pango word wrap)
- [x] Vertical text centering using ink extents (pixel_extents vs pixel_size for true visual centering)
- [x] Load actual WoW TTF fonts from project fonts/ directory via fontconfig FFI

### Tooltip System ✅
- [x] GameTooltip frame
- [x] SetOwner, AddLine, Show/Hide
- [x] Anchor positioning

### Frame Dragging/Resizing ✅
- [x] StartMoving/StopMovingOrSizing (state tracking)
- [x] SetMovable/SetResizable (state tracking)
- [x] Clamp to screen (SetClampedToScreen/IsClampedToScreen)

## Functionality

### Slash Commands ✅
- [x] Register handlers via SlashCmdList
- [x] Parse and dispatch /command input
- [x] Console input for typing commands

### API Coverage
Priority APIs by addon usage:
- [x] C_Timer (After, NewTimer, NewTicker) ✅
- [x] C_ChatInfo (SendAddonMessage, SendChatMessage) ✅
- [x] C_Covenants, C_Soulbinds ✅
- [x] Combat log events (CombatLogGetCurrentEventInfo, etc.) ✅
- [x] Unit auras (UnitAura, UnitBuff, UnitDebuff) ✅
- [x] Action bar APIs (GetActionInfo, HasAction, etc.) ✅
- [x] Table utilities (tContains, CopyTable, MergeTable) ✅
- [x] Secure functions (SecureCmdOptionParse, issecure, etc.) ✅

## Reference Addons

Located at `~/Projects/wow/reference-addons/`:
- `Ace3/` - Popular addon framework
- `AllTheThings/` - Collection tracking (3613 Lua, 43 XML)
- `DeadlyBossMods/` - Raid encounter alerts
- `Details/` - Damage meter
- `Plater/` - Nameplate addon
- `WeakAuras2/` - Custom display framework
- `wow-ui-source/` - Blizzard's official UI code

## Session Summary (2026-01-16 15:10)

### Progress: 195 → 189 warnings (-6 warnings, -3%)

### Key fixes implemented:

1. **BuffFrame.AuraContainer** - Added child frame with iconScale property
2. **UseRaidStylePartyFrames** - Added method to FrameHandle for EditModeManagerFrame
3. **PlayerCastingBarFrame** - Added global frame reference
4. **PartyFrame** - Added global frame reference
5. **PetFrame** - Added with healthbar, manabar, and text children (LeftText, RightText, TextString)
6. **PlayerFrame hierarchy** - Added PlayerFrameContent.PlayerFrameContentMain with:
   - HealthBarsContainer.HealthBar (with text children)
   - ManaBarArea.ManaBar (with text children)
7. **TargetFrame hierarchy** - Similar to PlayerFrame, plus totFrame
8. **FocusFrame hierarchy** - Similar to TargetFrame (with TargetFrameContent naming for compatibility)
9. **AlternatePowerBar** - Added global frame reference
10. **MonkStaggerBar** - Added global frame reference
11. **PowerBarColor** - Added color table for power bar colors (MANA, RAGE, ENERGY, RUNIC_POWER)
12. **Enum.UITextureSliceMode** - Added (Stretched=0, Tiled=1)
13. **UiMapPoint** - Added with CreateFromVector2D and CreateFromCoordinates methods
14. **Button textures** - Added automatic creation of NormalTexture, PushedTexture, HighlightTexture, DisabledTexture, Icon, IconOverlay, Border for Button/CheckButton types
15. **SettingsPanel.FrameContainer** - Added child frame

### Notable addon improvements:
- **BetterBlizzFrames**: 6 warnings → 2 warnings (only IO errors remain)
- **Plumber**: 4 warnings → 3 warnings (FontFamily XML and FrameContainer template issues remain)
- More Lua files loaded: 4254 → 4262 (+8 files)

## Session Progress (2026-01-16 continued)

### Progress: 121 → 117 warnings (-4 warnings)

### Key fixes implemented:

1. **SetFormattedText** - Added frame method for string.format + SetText in one call (used by Leatrix_Plus)
2. **CopyFontObject** - Fixed to accept both font table and font name string arguments (OmniCD fix)
3. **QuestDifficultyColors/QuestDifficultyHighlightColors** - Added quest difficulty color tables (Krowi_ExtendedVendorUI)
4. **GetFontInfo** - Added global function for font metadata (returns name/height/outline)
5. **Tooltip NineSlice** - GameTooltip and tooltip templates now create NineSlice child frame
6. **TOOLTIP_DEFAULT_BACKGROUND_COLOR** - Added color object with GetRGB method
7. **SetCenterColor** - Frame method for NineSlice center fill color
8. **XML $text variant** - Handle inline text content in XML (malformed XML resilience)
9. **lowercase script/include** - XML variants for compatibility (<script> and <include>)
10. **HUD_EDIT_MODE_* strings** - Added Edit Mode HUD label strings
11. **Enum.AuctionHouseSortOrder** - Auction sorting options
12. **Enum.AuctionHouseTimeLeftBand** - Auction time remaining bands
13. **Enum.ItemRecipeSubclass** - Recipe/profession item subclasses
14. **ModelScene methods** - TransitionToModelSceneID, SetFromModelSceneID, GetModelSceneID

### Current state:
- **126/126 addons loaded**
- **4548 Lua files** (+75 from session start)
- **277 XML files** (+3 from session start)
- **117 warnings** (down from 121)
- **70 addons with 0 warnings** (up from 69)

### OmniCD progress:
- Was 6 warnings with 0 Lua files loading
- Now 5 warnings with 51 Lua files loading
- Fixed CopyFontObject, NineSlice, tooltip colors

## Session Progress (2026-01-17 v17)

### Progress: 57 → 48 warnings (-9 warnings, -16%)

### Key fixes implemented:

1. **OBJECTIVE_TRACKER_BLOCK_HEADER_COLOR** - Added color constant for objective tracker headers
2. **QUEST_OBJECTIVE_FONT_COLOR** - Added color constant for quest objectives
3. **XML path resolution fallback** - resolve_path_with_fallback() tries xml_dir first, then addon_root
4. **PlumberSettingsPanelLayoutTemplate** - Creates FrameContainer with LeftSection, RightSection, CentralSection, etc.
5. **AddonCompartmentFrame.registeredAddons** - Added table for addon compartment registration
6. **ObjectiveTrackerManager** - Full stub with AssignModulesOrder, AddContainer, UpdateAll, etc.
7. **LIGHTGRAY_FONT_COLOR** - Added light gray color constant (0.75, 0.75, 0.75)
8. **Enum.ContentTrackingTargetType** - JournalEncounter, Vendor, Achievement, Profession, Quest
9. **Enum.QuestRewardContextFlags** - None, FirstCompletionBonus, RepeatCompletionBonus
10. **Enum.HousingPlotOwnerType** - None, Stranger, Friend, Self (Delves housing)
11. **C_QuestLog.GetMaxNumQuestsCanAccept** - Returns 35 (max quests)
12. **StaticPopup1-4 frames** - With EditBox, text, button1, button2 children

### Addons improved this session:
- **Angleur**: 6 warnings → 0 warnings (path resolution for translations.xml)
- **Plumber**: 2 warnings → 1 warning (template handler + registeredAddons)
- **!KalielsTracker**: 3 warnings → 1 warning, 51 Lua → 84 Lua files (+33 files)

### Current state:
- **127/127 addons loaded**
- **5092 Lua files** (+67 from session start)
- **298 XML files** (+4 from session start)
- **48 warnings** (down from 57)
- **105 addons with 0 warnings** (up from 103)
- **22 addons with warnings**

### Remaining warnings analysis:
Most remaining warnings are NOT WoW API issues:
- **IO errors** (~12): Missing files referenced in TOC/XML
- **UTF-8 encoding** (~6): Invalid byte sequences in Korean/Chinese/Taiwanese locale files
- **Syntax errors** (2): Lua 5.1 constant limits (BetterWardrobe), escape sequences (Cell)
- **Runtime cascading** (~26): Dependency failures, addon-internal issues
- **setmetatable on userdata** (2): WaypointUI tries to setmetatable on FrameHandle (fundamental design issue)

## Session Progress (2026-01-17 v15)

### Progress: 62 → 57 warnings (-5 warnings)

### Key fixes implemented:

1. **STAT_ARMOR and STAT_* strings** - Added stat strings (Strength, Agility, Stamina, Intellect, Spirit)
2. **ITEM_MOD_* strings** - Added 40+ item mod strings for primary, secondary, and tertiary stats
3. **Clear method conflict fix** - Removed Clear() from all FrameHandles, now only on Cooldown frames via __index
4. **AlertFrame:AddQueuedAlertFrameSubSystem** - Returns proper subsystem object with SetCanShowMoreConditionFunc
5. **Slash command globals** - Added SLASH_CAST*, SLASH_CASTSEQUENCE, SLASH_CLICK, SLASH_TARGET, SLASH_FOCUS, etc.
6. **Duplicate XML scripts fix** - ScriptsXml now uses Vec to allow duplicate OnClick/OnLoad/etc. elements

### Addons fixed this session:
- **Syndicator**: 1 warning → 0 warnings (STAT_ARMOR fix for gem stat check)
- **Rarity**: 2 warnings → 0 warnings (Clear method conflict + AlertFrame subsystem)
- **MacroToolkit**: 1 warning → 0 warnings (SLASH_CAST1, SLASH_CASTSEQUENCE1)
- **Baganator**: 2 warnings → 1 warning (duplicate OnClick XML fix)

### Current state:
- **127/127 addons loaded**
- **5025 Lua files** (+17 from fixes)
- **294 XML files** (+1 from Baganator fix)
- **57 warnings** (down from 62)
- **103 addons with 0 warnings** (up from 100)
- **24 addons with warnings**

### Remaining warnings analysis:
Most remaining warnings are NOT WoW API issues:
- **IO errors** (15 addons): Missing files referenced in TOC
- **UTF-8 encoding** (5 addons): Invalid byte sequences in Lua files
- **Addon dependencies** (3 addons): ElvUI_OptionsUI needs ElvUI, etc.
- **Structural limitations** (2 addons): setmetatable on userdata, too many constants
- **Template issues** (1 addon): parentKey template children not created

## Session Progress (2026-01-17 v14)

### Progress: 64 → 62 warnings (-2 warnings)

### Key fixes implemented:

1. **TOY global string** - Added for Syndicator search (lowercase "toy")
2. **WORLD_QUEST_REWARD_FILTERS_* strings** - Anima, Equipment, Gold, Resources for world quest filtering
3. **ITEM_ACCOUNTBOUND / ITEM_ACCOUNTBOUND_UNTIL_EQUIP** - Warbound item binding strings
4. **Socket strings** - All EMPTY_SOCKET_* constants (Blue, Red, Yellow, Meta, Prismatic, etc.)
5. **Item binding strings** - ITEM_SOULBOUND, ITEM_BIND_ON_EQUIP, ITEM_BIND_ON_PICKUP, ITEM_BIND_ON_USE
6. **More item strings** - MOUNT, PET, EQUIPMENT, REAGENT, APPEARANCE, TRANSMOG_SOURCE_LABEL, TRANSMOGRIFY
7. **AddonCompartmentFrame** - Converted from plain table to proper FrameHandle userdata with HookScript support
8. **C_CurrencyInfo.GetAzeriteCurrencyID** - Added Azerite currency ID for BfA content
9. **C_CurrencyInfo.GetBasicCurrencyInfo** - Added basic currency info lookup

### Current state:
- **127/127 addons loaded**
- **5008 Lua files** (+22 from AddonCompartmentFrame/MinimapButtonButton fix)
- **292 XML files**
- **62 warnings** (down from 64)
- **100 addons with 0 warnings** (up from 99)
- **27 addons with warnings**

### Addons fixed this session:
- MinimapButtonButton: 0 Lua, 1 warning → 22 Lua, 0 warnings

### Analysis of remaining warnings:
Most remaining warnings are NOT WoW API issues but addon-internal problems:

**Addon file issues (cannot fix in simulator):**
- Missing files referenced in TOC (Clicked, CraftSim, Simulationcraft, etc.)
- UTF-8 encoding errors in Lua files (ClickableRaidBuffs, GlobalIgnoreList, TomCats)
- Addon dependencies not loaded (ElvUI_OptionsUI needs ElvUI)

**Template/structural limitations:**
- Plumber: Custom template FrameContainer children not created (needs XML template instantiation)
- WaypointUI: setmetatable on userdata (frames are userdata, not tables)
- Rarity: Field assignment conflicts with method names (frame.Clear vs Clear button)

**Addon-specific Lua logic issues:**
- AllTheThings: Mixin method SetATTTooltip not applied to userdata
- BetterWardrobe: 65536 constant limit in ColorFilter.lua
- Cell: Korean locale escape sequences
- Syndicator: statKey nil in gem stat check

### Previous session (v13): 93 → 64 warnings

Key fixes:
- FontFamily XML handling for AstralKeys
- Button font object storage methods
- HybridScrollBarTemplate globals
- TOC placeholders: [TextLocale], [AllowLoadTextLocale], [Game]
- Enum.BagIndex, PortraitFrameTemplate, ClearAttributes()
- Zone text functions, FontString text scale methods

## Future Work

### Resource Limits
- Add memory and CPU limits when running the app (e.g., `ulimit` or cgroup-based limits)
- Current loading of 127 addons can be resource-intensive

### Remaining Addon Issues (v8)
Common patterns in remaining warnings:
- File not found errors (optional addon files)
- Locale encoding issues (UTF-8 with invalid escapes)
- SecureGroupHeader template child indexing
- Complex nested structures (Settings canvas layouts)

