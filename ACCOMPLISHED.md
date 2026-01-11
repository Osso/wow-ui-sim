# Accomplished

## 2026-01-11 (session 4)

### Phase 5: Real Addon Testing - WeakAuras Support

**New APIs Added:**
- `C_AddOns` namespace - `GetAddOnMetadata`, `EnableAddOn`, `DisableAddOn`, `GetNumAddOns`, `GetAddOnInfo`, `IsAddOnLoaded`, `IsAddOnLoadable`, `LoadAddOn`, `DoesAddOnExist`
- `AddonCompartmentFrame` - `RegisterAddon`, `UnregisterAddon` (retail addon button compartment)
- Legacy globals: `GetAddOnMetadata`, `GetNumAddOns`, `IsAddOnLoaded`, `LoadAddOn`

**New Frame Methods:**
- `SetFixedFrameStrata(fixed)` - Frame strata inheritance control
- `SetFixedFrameLevel(fixed)` - Frame level inheritance control
- `SetToplevel(toplevel)`, `IsToplevel()` - Toplevel frame handling
- `Raise()`, `Lower()` - Frame z-order manipulation

**Button Methods Added:**
- `SetNormalFontObject`, `SetHighlightFontObject`, `SetDisabledFontObject`
- `GetNormalTexture`, `GetHighlightTexture`, `GetPushedTexture`, `GetDisabledTexture`
- `SetNormalTexture`, `SetHighlightTexture`, `SetPushedTexture`, `SetDisabledTexture`
- `SetEnabled`, `IsEnabled`, `Click`, `RegisterForClicks`
- `SetButtonState`, `GetButtonState`

**WeakAuras Testing:**
- **WeakAuras Init.lua loads successfully**
- WeakAuras table created with all version detection functions (IsRetail, IsClassic, etc.)
- AddonCompartmentFrame registration works
- Missing libs correctly detected and reported (expected - full lib loading not tested)

**Plater Testing:**
- 7 Lua files, 1 XML file load
- AceConfigDialog partially loads (needs more API stubs)
- Main blocker: DetailsFramework library not available in git repo

**Test Count:** 68 passing (7 ignored)

## 2026-01-11 (session 3)

### Phase 5: Real Addon Testing - Extended API Coverage

**New APIs Added:**
- `GetBuildInfo()` - returns version "11.0.7", build, date, tocversion
- `GetPhysicalScreenSize()` - simulated 1920x1080 screen
- `UnitPlayerControlled(unit)` - check if unit is player controlled
- `UnitIsTapDenied(unit)` - check if unit is tapped (always false)
- `PixelUtil` namespace - `SetWidth`, `SetHeight`, `SetSize`, `SetPoint`, `GetPixelToUIUnitFactor`
- `Round()`, `Lerp()`, `Clamp()`, `Saturate()`, `ClampedPercentageBetween()` - math utilities
- `C_EventUtils.IsEventValid()` - event validation
- `C_CVar` namespace - `GetCVar`, `SetCVar`, `GetCVarBool`, `RegisterCVar`
- `C_Container` namespace - `GetContainerNumSlots`, `GetContainerItemID`, `GetContainerItemLink`, `GetContainerItemInfo`
- `C_Item` namespace - `GetItemInfo`, `GetItemInfoInstant`, `GetItemIconByID`
- `C_SpellBook` namespace - `GetSpellBookItemName`, `GetNumSpellBookSkillLines`, `GetSpellBookSkillLineInfo`, etc.
- `C_Spell` namespace - `GetSpellInfo`, `IsSpellPassive`, `GetOverrideSpell`
- Legacy globals: `GetCVar`, `SetCVar`, `GetItemInfo`, `GetSpellInfo`, `GetNumSpellTabs`, etc.

**New Frame Methods:**
- `SetBackdrop(backdropInfo)` - accept backdrop config
- `SetBackdropColor(r, g, b, a)` - accept backdrop color
- `SetBackdropBorderColor(r, g, b, a)` - accept border color
- `SetID(id)`, `GetID()` - frame ID for tab ordering
- `HookScript(handler, func)` - hook into existing script handlers

**Built-in UI Elements:**
- `Minimap` global frame - used by LibDBIcon for minimap button positioning

**Infrastructure Improvements:**
- `debugstack()` now returns real Lua stack traces with file paths
- `exec_named()` method for loading Lua with custom chunk names
- File paths transformed to WoW-style (`Interface/AddOns/...`) for library compatibility
- Enabled full debug library via `unsafe_new()` for debugstack support

**Details Addon Progress:**
- **67 Lua files loaded** (up from 65), 92 warnings (down from 94)
- All Ace3 libs load: LibStub, CallbackHandler, AceLocale, AceAddon, AceComm, AceSerializer, AceTimer
- LibSharedMedia, NickTag, LibDataBroker, LibDBIcon, LibGraph-2.0, LibWindow, PlayerInfo all load
- LibOpenRaid partially loads (Functions.lua, GetPlayerInformation.lua load)
- Current blocker: data error in ThingsToMantain_WarWithin.lua (not missing API)

**Test Count:** 66 passing (7 ignored)

## 2026-01-11 (session 2)

### Phase 4: Rendering Complete

- **iced integration** with canvas-based rendering
- **Z-ordering**: frames sorted by strata then level before drawing
- **Widget type differentiation**:
  - Frames: colored rectangles with name labels
  - Buttons: distinct blue color
  - Textures: brown/orange color
  - FontStrings: semi-transparent with text content rendered
- **Alpha transparency** applied to all widgets
- **Demo app**: main.rs creates sample frames and runs interactive UI

### Phase 5: Real Addon Testing Progress

- **Ace3 library suite**: 100% loaded (43 Lua, 15 XML files)
- **Details addon**: Partial load (65 Lua, 2 XML files, needs more game APIs)
- **DBM-Core**: Partial load (19 Lua, needs bundled Libs folder)

**New APIs Added:**
- `GetRealmName()`, `GetNormalizedRealmName()` - realm info
- `GetLocale()` - locale string
- `UnitName()`, `UnitGUID()`, `UnitLevel()`, `UnitExists()`, `UnitFactionGroup()` - unit info
- `GetCurrentRegion()`, `GetCurrentRegionName()` - region info
- `UnregisterAllEvents()` - frame method
- `C_ChatInfo.RegisterAddonMessagePrefix()`, `C_ChatInfo.SendAddonMessage()` - addon communication
- `RegisterAddonMessagePrefix()` - legacy global version

**Infrastructure:**
- Path normalization for Windows-style backslashes in XML includes
- 65 tests passing (7 ignored)

## 2026-01-11

### Blizzard UI Loading Results

- **Blizzard_SharedXMLBase**: 100% loaded (34 Lua, 2 XML)
- **Blizzard_SharedXML**: 100% loaded (155 Lua, 72 XML, 0 warnings)

### Addon Loading Infrastructure

- **TOC Parser** (`src/toc.rs`)
  - Parse .toc files for addon metadata and file lists
  - Interface versions, dependencies, optional deps, saved variables
  - Strip inline annotations like `[AllowLoadEnvironment Global]` from file paths

- **Addon Loader** (`src/loader.rs`)
  - Load addons from TOC files
  - Process Lua and XML files in order
  - Handle Script and Include elements in XML

### XML Parser (`src/xml/mod.rs`)

- **$value pattern for child elements** - handle duplicate Frame/Layers/Texture elements
  - `FrameXml` uses `Vec<FrameChildElement>` with helper methods (`.size()`, `.layers()`, `.scripts()`)
  - `FramesXml` uses `Vec<FrameElement>` enum for Frame/Button/CheckButton/etc.
  - `LayerXml` uses `Vec<LayerElement>` enum for Texture/FontString
- **Widget types**: Frame, Button, CheckButton, EditBox, ScrollFrame, Slider, StatusBar, GameTooltip, ColorSelect, Model, ModelScene, EventFrame, EventButton, EventEditBox, Cooldown, DropdownButton, AnimationGroup, Actor, Font, CinematicModel, PlayerModel, DressUpModel
- **Made `ScriptXml.file` optional** to support inline scripts

### Lua APIs Added (`src/lua_api/globals.rs`)

**Enums:**
- `Enum.LFGRole`, `Enum.UnitSex`, `Enum.GameMode`, `Enum.Profession`
- `Enum.VasTransactionPurchaseResult` (70+ values for VASErrorLookup.lua)
- `Enum.StoreError`, `Enum.GameRule`
- `Enum.ScriptedAnimationBehavior`, `Enum.ScriptedAnimationTrajectory`

**C_* Namespaces:**
- `C_UIColor.GetColors()`
- `C_ClassColor.GetClassColor()` - returns color object with methods
- `C_Timer.NewTicker()`, `C_Timer.NewTimer()`
- `C_GameRules.IsGameRuleActive()`, `C_GameRules.GetActiveGameMode()`, `C_GameRules.GetGameRuleAsFloat()`, `C_GameRules.IsStandard()`
- `C_Glue.IsOnGlueScreen()`
- `C_ScriptedAnimations.GetAllScriptedAnimationEffects()`

**Functions:**
- `CreateColor()` - creates color object with r/g/b/a fields and methods
- `CreateAndInitFromMixin()`, `nop()`
- `UnitRace()`, `UnitSex()`, `UnitClass()`
- `GetCurrentEnvironment()` - fixed to return `_G` table

**Frame methods:**
- `SetForbidden`, `IsForbidden`, `CanChangeProtectedState`
- `SetPassThroughButtons`, `SetFlattensRenderLayers`, `SetClipsChildren`
- `SetShown`, `GetEffectiveScale`, `GetScale`, `SetScale`
- `GetAttribute`, `SetAttribute` - with OnAttributeChanged script triggering

**Constants:**
- `Constants.LFG_ROLEConstants`, `Constants.AccountStoreConsts`
- `PLAYER_FACTION_COLOR_HORDE`, `PLAYER_FACTION_COLOR_ALLIANCE`
- `WOW_PROJECT_MAINLINE`, `WOW_PROJECT_ID`, `WOW_PROJECT_CLASSIC`

**Script Handlers:**
- `OnAttributeChanged` - triggered by SetAttribute, enables CallbackRegistry pattern

**Fixes:**
- Fixed `Mixin()` to handle nil arguments gracefully
- Fixed raw string delimiter (`r##"..."##`) for embedded `"#` in Lua code

### Tests

- 59 tests passing (7 ignored)
- Test files: `toc_parsing.rs`, `xml_parsing.rs`, `blizzard_shared.rs`

## 2026-01-10

- Implemented core Lua environment with mlua (Lua 5.1)
- Created widget system: Frame, Texture, FontString, Button types
- Implemented anchor system (SetPoint, GetPoint, ClearAllPoints, SetAllPoints)
- Added frame properties: alpha, strata, level, mouse_enabled, visibility
- Event system: RegisterEvent, UnregisterEvent, SetScript, GetScript
- Global WoW API functions: CreateFrame, wipe, strsplit, tinsert, tremove, hooksecurefunc
- Lua stdlib aliases: string (strlen, strsub, etc.), math (abs, floor, etc.), table (foreach, getn, etc.)
- Bit operations: pure Lua 5.1 implementation (band, bor, bxor, bnot, lshift, rshift, arshift)
- WoW intrinsics: Mixin, CreateFromMixins, issecure, issecurevariable, debugstack, GetTime
- Security functions stubbed as always-secure (simulation doesn't need taint tracking)
- Created TestAddon with 13 test patterns covering all implemented APIs
- LibStub and CallbackHandler-1.0 compatibility verified (4 tests)
- AutoRoll addon loads and runs (2 tests)
- Cloned reference addons to ~/Projects/wow/reference-addons/:
  - Ace3, DeadlyBossMods, Details, Plater, WeakAuras2, wow-ui-source
