# Lua API Implementation

## Overview

The WoW UI Simulator provides a Lua API that mirrors World of Warcraft's frame, event, and utility systems. Implementation spans: environment setup, frame userdata with metatables, global functions, widget methods, animations, and API stubs.

**Key Files:**
- `src/lua_api/env.rs` - Main Lua environment (WowLuaEnv)
- `src/lua_api/state.rs` - Shared simulator state (SimState)
- `src/lua_api/frame/handle.rs` - Frame userdata (FrameHandle)
- `src/lua_api/frame/methods/` - All frame methods (14+ submodules)
- `src/lua_api/globals_legacy.rs` - Main global function registration
- `src/lua_api/globals/` - API namespace implementations

---

## WowLuaEnv

**File:** `src/lua_api/env.rs`

```rust
pub struct WowLuaEnv {
    pub(crate) lua: Lua,
    pub(crate) state: Rc<RefCell<SimState>>,
    on_update_errors: RefCell<HashSet<u64>>,
}
```

### Initialization (lines 31-52)

Creates Lua with full stdlib, initializes SimState with UIParent/WorldFrame via `create_builtin_frames`, registers all global functions via `register_globals`.

### Execution

- `exec()` / `exec_named()` -- Direct code execution
- `exec_with_varargs()` -- Addon loading with (addonName, addonTable) varargs
- `eval()` -- Return computed values

Loader execution keeps public `_G` and secure `__secureenv` distinct. Same-addon nil-symbol reconciliation records non-`C_*` publications separately for each environment, including secure Lua assignments and Rust-created secure frame exports; a publication only resolves a nil lookup from the same environment. Precompiled OnLoad/OnShow dispatch uses raw `_G.self` snapshot/restore so its temporary receiver does not become a client global observation. Post-cleanup namespace restoration likewise reads raw `_G.C_StoreSecure` before merging simulator state, avoiding attribution of bootstrap lookups to Blizzard code.

### Versioned UI strings

The retail 12.1 compatibility table publishes 70 strings under `profile-retail` + `retail-12-1-0`: 45 existing strings, 16 Social UI compatibility labels, seven ChatFrame combat-audio format strings, and two ChatFrame combat-audio command names. Commit `cc02aa287` adds 13 housing Settings labels/tooltips pinned from build `12.1.0.69497`; missing values had caused the canonical Interface Settings registrant to allocate category ID 7 and abort before registration. Commit `cfec8576b` adds the 16 retail-12.1-gated Social UI tag/presence labels required to keep `Blizzard_SocialUIShared` tables populated; their English values were not among the existing 49-candidate live probe. Commits `cfec8576b` and `8f070a410` publish `Enum.SocialUIPresenceType` (`Unknown=0` through `AppearOffline=5`) and `Enum.SocialUIBlockType` (`None=0`, `Ignore=1`, `BattleNetInviteBlock=2`) with metadata (`MinValue`, `MaxValue`, `NumValues`). Commit `9555a7649` publishes the generated `Enum.CooldownViewerSound` contract (`0..93`, 94 values) and metadata, satisfying the enum reads in `Blizzard_ChatFrame/Shared/TextToSpeechCommands.lua` during preload. Commit `43075580b` adds seven retail-12.1-gated English compatibility format strings needed by unconditional `:format(min, max)` calls in that preload. Commit `abcfb395b` adds `SLASH_CAA_WHEN_TARGET_DIES` and `SLASH_CAA_PLAY_SOUND`, passed to `AddCommand` and normalized with `string.lower` during the same preload. Commit `4d90beffd` publishes generated `Enum.RaidDispelOverlayType` (`Disabled=0`, `UseDebuffColor=1`, `UseBlack=2`) with metadata; `CompactUnitFrameOptions` reads `UseDebuffColor`, and the later `CompactUnitFrameUtil` `pairs(nil)` was downstream of that aborted options load. Overlay rendering behavior is unmodeled. Commit `ec0e21537` adds `Enum.RecentAlliesFriendTag` (`0..5`) with metadata and sparse `Enum.RolodexType.LegacyFriend=23`, preserving the `21`/`22` gaps with metadata (`MinValue=0`, `MaxValue=23`, `NumValues=22`) for independent `Blizzard_RecentAllies` preload tables. Recent Allies service/search behavior is unmodeled. Commit `6eae179ff` adds generated `Enum.CustomAuraButtonDispelTypeTextureStyle` (`Border=0` through `CustomAsset=4`) with metadata for `Blizzard_Deprecated/Deprecated_12_1_0.lua` to construct `AuraButtonBorderStyle` aliases when deprecation fallbacks load. Texture and rendering behavior remain unmodeled. Commit `e3806caba` adds generated retail-12.1 `Enum.HousingResult` (`0..111`) with metadata (`MinValue=0`, `MaxValue=111`, `NumValues=112`) before the stale missing-enums fallback, allowing `Blizzard_HousingTemplates` to build `HousingResultToErrorText`; housing operations and result strings remain unmodeled, and the 12.0.0 contract is unchanged. Commit `db678a399` adds the current wowt-global-dataset `Enum.HouseSettingFlags` bitmask (`None=0` through `BlueprintExportParty=16384`) with metadata (`MinValue=0`, `MaxValue=16384`, `NumValues=16`) before that fallback; current `Blizzard_HousingData` exercises the four non-`Anyone` blueprint-export flags in `HousingAccessTypeStrings`. Housing access/export behavior and strings remain unmodeled, and the pre-12.1 contract is unchanged. Commit `045c9fa19` adds generated `Enum.HousingBlueprintType` (`None=0` through `Exterior=4`) with metadata (`MinValue=0`, `MaxValue=4`, `NumValues=5`) for `Blizzard_HousingData` blueprint-type label tables. Commit `432d357a1` adds generated `Enum.HousingBlueprintUnmetRequirementFlags` (`InsufficientBudget=1` through `HouseSizeLocked=128`, eight members and no `None`) with metadata (`MinValue=1`, `MaxValue=128`, `NumValues=8`) for that file's blueprint-requirement text table. Commit `665f5a31b` adds generated `Enum.HousingBlueprintContentType` (`None=0` through `Other=6`) with metadata (`MinValue=0`, `MaxValue=6`, `NumValues=7`) for its final content label table. Housing blueprint validation, operations, and strings remain unmodeled. The compatibility-string values were not captured by the live probe. This is compatibility string/enum publication only; separately probed nil globals stay absent.

Retail 12.1 removes public `GetInventorySlotInfo`. Current `Blizzard_TransmogShared` calls `C_PaperDollInfo.GetInventorySlotInfo` directly, so no loader-scoped legacy-global exception remains; see [[transmog-inventory-slot-scope]] for the retired stale-source workaround.

### Settings panel opening

`C_SettingsUtil.OpenSettingsPanel(categoryID, scrollTarget)` forwards to `SettingsPanelMixin.OpenToCategory(SettingsPanel, categoryID, scrollTarget)`. An absent category ID becomes `0`; it returns no values and is a no-op if `SettingsPanel` or that mixin method is unavailable. The temporary Settings defaults add only missing category IDs and `Settings.OpenToCategory`, preserving Blizzard-provided members. `C_SettingsUtil.NotifySettingsLoaded` and category/element scrolling behavior beyond forwarding the supplied target remain unsupported.

### Temporary pet-battle runtime

`src/lua_api/workarounds/temporary/pet_battle_runtime_state.rs` seeds sample allied and enemy pets for compatibility. `C_PetBattles.GetBreedQuality(owner, petIndex)` returns seeded `Enum.BattlePetBreedQuality.Rare` (`3`) or numeric fallback `0` for an absent pet, allowing current `PetBattleFrame` OnLoad rarity rendering. It does not model pet ownership, breeding, capture, combat outcomes, or live battle data.

### Temporary housing dashboard service response

`housing_dashboard_preload.rs` supplies only absent `C_Housing.GetPlayerOwnedHouses`. It schedules `PLAYER_HOUSE_LIST_UPDATED` with an empty table through `C_Timer.After(0)` rather than firing synchronously, so dashboard consumers observe the empty-owned-house response on the next timer tick. This remains a temporary preload bridge, not a model of owned-house service state, request failure, or payload semantics.

### Timer System (lines 382-506)

- `schedule_timer()` -- Optional interval/iterations, returns unique timer ID
- `cancel_timer()` -- Marks timer as cancelled
- `process_timers()` -- Fires due callbacks, reschedules repeating timers
- `next_timer_delay()` -- For event loop timing

### Event Firing (lines 112-223)

- `fire_event()` -- Dispatches to registered listeners via `__scripts` table (`{frame_id}_OnEvent`)
- `fire_event_collecting_errors()` -- For test harnesses
- `fire_script_handler()` -- For arbitrary handlers (onClick, etc.)

---

## FrameHandle Userdata

**File:** `src/lua_api/frame/handle.rs`

```rust
pub struct FrameHandle {
    pub id: u64,
    pub state: Rc<RefCell<SimState>>,
}
```

### Dual System Architecture

**Lua Side:** FrameHandle userdata with metatables, parent-child via table properties, script handlers in `__scripts` table.

**Rust Side:** Frame struct in WidgetRegistry with `children: Vec<u64>` and `children_keys: HashMap<String, u64>`.

**Sync via `__newindex`:** `parent.Child = frame` triggers `parent_frame.children_keys.insert("Child", frame_id)`.

---

## Frame Methods (14+ Submodules)

### Core Methods (`methods_core.rs`)

- **Identity:** `GetName()`, `GetObjectType()`, `IsObjectType()`
- **Size:** `GetWidth()`, `GetHeight()`, `SetSize()`, `SetWidth()`, `SetHeight()`
- **Position:** `GetRect()`, `GetScaledRect()`, `GetLeft/Right/Top/Bottom/Center/Bounds()`
  - Coordinate conversion at line 144: `bottom = screen_height - rect.y - rect.height`
- **Visibility:** `Show()`, `Hide()`, `IsVisible()`, `SetAlpha()`, `GetAlpha()`
- **Strata/Level:** `GetFrameStrata()`, `SetFrameStrata()`, `GetFrameLevel()`, `SetFrameLevel()`
- **Mouse:** `EnableMouse()`, `IsMouseEnabled()`, `SetMouseClickEnabled()`, `SetMouseWheel()`
- **Scale:** `GetScale()`, `SetScale()`, `GetEffectiveScale()`

### Anchor Methods (`methods_anchor.rs`)

`SetPoint()`, `ClearAllPoints()`, `SetAllPoints()`, `GetNumAnchors()`, `GetAnchor()`

### Event Methods (`methods_event.rs`)

`RegisterEvent()`, `UnregisterEvent()`, `UnregisterAllEvents()`, `RegisterUnitEvent()`, `RegisterAllEvents()`, `IsEventRegistered()`

### Script Methods (`methods_script.rs`)

`SetScript()`, `GetScript()`, `HookScript()`, `WrapScript()`, `UnwrapScript()`, `ClearScripts()`, `HasScript()`

**OnUpdate:** `SetScript("OnUpdate")` adds frame to `on_update_frames` set for per-frame ticking.

### Child Creation (`methods_create.rs`)

`CreateTexture()`, `CreateFontString()`, `CreateFrame()`, `CreateAnimationGroup()`

### Texture Methods (`methods_texture.rs`)

`SetTexture()`, `SetAtlas()`, `GetTexture()`, `SetTexCoord()`, `GetTexCoord()`, `SetVertexColor()`, `SetBlendMode()`, `SetDrawLayer()`, `SetGradient()`

### Text/FontString Methods (`methods_text/mod.rs`)

`SetText()`, `GetText()`, `GetTextHeight()`, `GetStringWidth()`, `SetFont()`, `GetFont()`, `SetTextColor()`, `SetShadowColor()`, `SetShadowOffset()`, `SetJustifyH()`, `SetJustifyV()`, `SetFormattedText()`, `SetWordWrap()`

### Button Methods (`methods_button.rs`)

`SetNormalTexture()`, `SetPushedTexture()`, `SetHighlightTexture()`, `SetDisabledTexture()` and corresponding getters. `GetFontString()`, `SetButtonState()`, `IsPressed()`.

### Hierarchy Methods (`methods_hierarchy.rs`)

`GetParent()`, `SetParent()`, `GetChildren()`, `GetNumChildren()`, `GetRegions()`

### Attribute/Backdrop Methods

`SetAttribute()`, `GetAttribute()`, `SetBackdrop()`, `GetBackdrop()`, `SetBackdropColor()`, `SetBackdropBorderColor()`

**Flatten Render Layers:** `SetFlattensRenderLayers()` persists per-frame local flatten state, `GetFlattensRenderLayers()` returns only that local flag, and `GetEffectivelyFlattensRenderLayers()` walks ancestors so descendants inherit effective flattening without inheriting the local flag itself.

### Widget-Type-Specific Methods

**EditBox:** `SetText()`, `GetText()`, `SetMaxLetters()`, `SetMultiLine()`, `SetAutoFocus()`, `SetFocus()`, `ClearFocus()`

**Slider:** `GetMinMaxValues()`, `SetMinMaxValues()`, `GetValue()`, `SetValue()`, `GetValueStep()`, `SetValueStep()`, `GetOrientation()`, `SetOrientation()`

**StatusBar:** `GetMinMaxValues()`, `SetMinMaxValues()`, `GetValue()`, `SetValue()`, `SetStatusBarColor()`

**Cooldown:** `SetCooldown()`, `GetCooldownDuration()`, `GetCooldownStartTime()`, `Clear()`

**Tooltip:** `SetOwner()`, `AnchorTo()`, `SetText()`, `AddLine()`, `AddDoubleLine()`, `Hide()`, `Show()`

**MessageFrame:** `AddMessage()`, `Clear()`

**ScrollBox:** `ScrollToBegin()`, `ScrollToEnd()`, `SetHorizontalScroll()`, `SetVerticalScroll()`

### Metamethods (`methods_meta.rs`)

**`__index`:** Numeric indexing -> child, named access -> children_keys, custom fields -> `__frame_fields` table, fallback methods.

**`__newindex`:** Syncs frame property assignment to Rust `children_keys`.

**`__len`:** Returns children count. **`__eq`:** Frame comparison by ID.

---

## QuestPOI Blob Data

Quest world-map blob polygons are already sourced in `data/quest_poi_blobs.rs`.
The current simulator uses a small hand-maintained dataset rather than a
generated import pipeline.

I also checked `~/Projects/world-of-osso/game-engine` for reusable world-map
code. That project does have a real `WorldMapPlugin` and a `WorldMapFrame`
screen, but they live in a separate Bevy/UI-toolkit stack with their own
screen model and FDID-driven texture pipeline. There is no drop-in WoW widget
or render helper there that cleanly ports into `wow-ui-sim`, so the simulator
continues to own its QuestPOI blob data and rendering path locally.

That data feeds the full QuestPOI blob path:
- `GetQuestPOIBlobCount(...)` reads the per-quest polygon table
- `UpdateMouseOverTooltip(x, y)` uses `hit_test_blobs(...)` for point-in-polygon matching
- `DrawBlob()` rendering triangulates the stored polygon vertices into the render batch

Each blob entry stores `quest_id`, `map_id`, and normalized `(x, y)` vertices
in the same 0.0-1.0 map space used by the QuestPOI APIs.

---

## Global Functions

### Registration Flow (`globals_legacy.rs:44-52`)

```rust
register_print, register_custom_ipairs, register_custom_getmetatable,
register_create_frame, register_submodule_apis, register_ui_strings_and_fonts,
patch_string_format
```

### Core Overrides

- **print** -- Appends to `SimState.console_output`, tab-separated
- **ipairs** -- Custom iterator for frames (returns children), falls back to original for tables
- **getmetatable** -- Returns fake metatable with widget-filtered `__index` methods; direct userdata calls still resolve through the shared `FrameRef` registration path
- **string.format** -- Converts `%F` -> `%f` (WoW LuaJIT vs standard Lua 5.1)

### Build options
**File:** `src/lua_api/globals/utility_system_spell/mod.rs`

Retail 12.1 `GetBuildOption("RestrictedAuraAPI")` returns `true`, selecting current forbidden aura templates. Unknown names return `nil`.

### Modeled targeting global
**File:** `src/lua_api/globals/targeting_verbs.rs`

`ClearTarget()` clears the current target and returns `true` iff a target existed; it returns `false` when no target was set and preserves the `PLAYER_TARGET_CHANGED` event.

### Profile-scoped guild panel toggle
**File:** `src/lua_api/globals/panel_toggle_verbs.rs`

`ToggleGuildFrame()` is profile-scoped: non-retail profiles retain the simulator fallback, while retail does not register one and relies on the canonical Blizzard implementation supplied by `Blizzard_Communities` after that addon loads. A bare retail `WowLuaEnv` therefore reports `type(ToggleGuildFrame) == "nil"`; reduced fixtures must load `Blizzard_Communities` before invoking the toggle.

### CreateFrame
**File:** `src/lua_api/globals/create_frame.rs`

```lua
CreateFrame(frameType, name, parent, template)
```

Lines 14-48: Main function. Lines 52-93: Argument parsing with `$parent`/`$Parent` substitution. Lines 116-156: Registration + parenting + strata inheritance. Lines 183-246: Widget type defaults.

### Font System
**File:** `src/lua_api/globals/font_api.rs`

`CreateFont()`, `CreateFontFamily()`, `GetFontInfo()`, `GetFonts()`

**Standard Fonts** (lines 335-414): GameFontNormal, GameFontHighlight, GameFontDisable, NumberFontNormal, SystemFont_Small/Med1-3/Large, ChatFontNormal, GameTooltipText, SubZoneTextFont, etc.

Font table structure: `__fontPath`, `__fontHeight`, `__fontFlags`, `__textColorR/G/B/A`, `__shadowColorR/G/B/A`, `__shadowOffsetX/Y`, `__justifyH/V`.

Methods: `SetFont()`, `GetFont()`, `SetTextColor()`, `SetShadowColor()`, `SetShadowOffset()`, `SetJustifyH/V()`, `CopyFontObject()`.

### Object Pools
**File:** `src/lua_api/globals/pool_api.rs`

- **CreateTexturePool** (lines 21-39): Stub textures with SetTexture, SetTexCoord
- **CreateFramePool** (lines 63-90): Maintains active/inactive tables, calls CreateFrame for new
- **CreateFrameFactory** (lines 172-245): Multi-template pooling for ScrollBoxListView
- **CreateObjectPool** (lines 292-325): Generic pool with creator/resetter functions

### Timer System
**File:** `src/lua_api/globals/timer_api.rs`

```lua
C_Timer.After(seconds, callback)
handle = C_Timer.NewTimer(seconds, callback)
handle = C_Timer.NewTicker(seconds, callback, iterations)
handle:Cancel()
handle:IsCancelled()
```

### Utility Functions
**File:** `src/lua_api/globals/utility_api.rs`

**Table:** `wipe()`, `tinsert()`, `tremove()`, `tInvert()`, `tContains()`, `tIndexOf()`, `tFilter()`, `CopyTable()`, `MergeTable()`

**String:** `strsplit()`

**Global:** `getglobal()`, `setglobal()`, `loadstring()`, `GetCurrentEnvironment()`

**Security:** `issecure()`, `issecurevariable()`, `securecall()`, `securecallfunction()`, `forceinsecure()`, `hooksecurefunc()`, `SecureCmdOptionParse()`. The read-only `debug.isglobalindex()` query exposes VM-scoped `_G.__index` provenance for syntactic global loads; explicit `_G.name`/`_G[name]` reads remain ordinary table probes.

**Mixin:** `Mixin(target, ...)`, `CreateFromMixins(...)`

### UI Mixins
**File:** `src/lua_api/globals/mixin_api.rs`

POIButtonMixin, TaggableObjectMixin, MapCanvasPinMixin, Menu (GetOpenMenu, PopupMenu, OpenMenu, CloseAll, Response), MenuUtil (CreateRootMenuDescription, etc.)

---

## Animation System

**File:** `src/lua_api/animation/`

### Types

Alpha, Translation, Scale, Rotation, LineTranslation, LineScale, Path, FlipBook, VertexColor, TextureCoordTranslation, Animation. Smoothing: None, In, Out, InOut.

### API

```lua
group = frame:CreateAnimationGroup()
anim = group:CreateAnimation("Alpha", name)
group:Play() / Stop() / Pause()
group:SetLooping("NONE" | "REPEAT" | "BOUNCE")
group:SetScript("OnFinished", callback)
anim:SetFromAlpha() / SetToAlpha() / SetDuration()
```

### OnUpdate Integration (`env.rs:517-555`)

`fire_on_update()` fires OnUpdate handlers for visible frames, OnPostUpdate handlers, then ticks animation groups.

---

## C_* Namespace APIs

| Namespace | File | Key Functions |
|-----------|------|---------------|
| C_Timer | `timer_api.rs` | After, NewTimer, NewTicker |
| C_Map | `c_map_api.rs` | GetMapInfo (stub) |
| C_Item | `c_item.rs` | GetItemInfo, GetItemCooldown; state-backed IsConsumableItem, IsEquippableItem, IsItemInRange |
| C_Container | `item_spell/c_container.rs` | GetContainerItemID/Link/Info, GetBagSlotFlag/SetBagSlotFlag, and backpack flag queries backed by shared bag-slot state |
| C_GuildInfo | `guild_info.rs` / `guild_verbs.rs` | Invite, Uninvite, Promote, and Leave share state-backed guild management handlers with deprecated globals |
| C_ProfSpecs | `workarounds/temporary/profession_specs_defaults.rs` | Seeded profession fixture reports specialization only for Blacksmithing skill line 164; authoritative path/tree behavior is unsupported |
| C_Spell | `c_spell.rs` | GetSpellQueueWindow reads the current SpellQueueWindow CVar |
| C_VoiceChat | `missing_surface/voice_chat.rs` | GetActiveChannelType reads the active channel's ChatChannelType or returns nil |
| C_System | `c_system_api.rs` | GetLocale -> "enUS" |
| C_EditMode | `c_editmode_api.rs` | GetLayouts |
| C_CatalogShop | `c_catalog_shop.rs` | GetVCProductInfos -> fresh empty table |
| C_ChromieTime | `c_chromie_time.rs` | Empty expansion-option queries; CloseUI/SelectChromieTimeOption no-ops (retail/PTR) |
| C_Quest | `c_quest_api.rs` | IsQuestFlaggedCompleted -> false |
| C_AchievementInfo | `c_stubs_api.rs` | GetRewardItemID, GetAchievementInfo (nil) |
| C_ClassTalents | `c_stubs_api.rs` | GetActiveConfigID (nil) |
| C_Guild | `c_stubs_api.rs` | GetNumMembers (0), IsInGuild (false) |
| C_LFGList | `c_stubs_api.rs` | GetActiveEntryInfo (nil) |
| C_Mail, C_Stable, C_Tutorial, C_ActionBar | `c_stubs_api.rs` | All return stub values |

---

## Enums & Constants

**File:** `src/lua_api/globals/enum_api.rs` + `enum_data/`

```lua
Enum.FrameStrata = { BACKGROUND=1, LOW=2, MEDIUM=3, HIGH=4, DIALOG=5, ... }
Enum.AnchorPoint = { TOPLEFT=1, TOP=2, ..., BOTTOMRIGHT=9 }
Enum.TextJustifyHorizontal = { LEFT=1, CENTER=2, RIGHT=3 }
Enum.BlendMode = { BLEND=1, ADDITIVE=2, DISABLE=3 }
Enum.DrawLayer = { BACKGROUND=1, BORDER=2, ARTWORK=3, OVERLAY=4, HIGHLIGHT=5 }
```

---

## Slash Commands

**File:** `src/lua_api/env.rs:239-288`

1. Parse `/command message`
2. Scan globals for `SLASH_NAME1`, `SLASH_NAME2`, etc.
3. Extract handler name
4. Call `SlashCmdList[name](message)`

---

## CVars

**File:** `src/lua_api/globals/set_cvar_verb.rs`

`GetCVar()`, `SetCVar()`, `GetCVarBool()`, `GetCVarNumber()` are backed by `SimState.cvars` (`CVarStorage`). A successful `SetCVar()` stores the new value before dispatching `CVAR_UPDATE` with the CVar name and value. For `uiScale` and `useUiScale`, the simulator applies the modeled UIParent scale when available, dispatches `DISPLAY_SIZE_CHANGED` then `UI_SCALE_CHANGED` while `GetCVar()` still exposes the old value, and then stores the requested value and dispatches `CVAR_UPDATE`.

---

## Workarounds

**File:** `src/lua_api/workarounds/mod.rs`

Applied after addon loading via `env.apply_post_load_workarounds()`:

1. **Settings surface** -- Reconcile a replacement `_G.SettingsPanel`/`Settings` pair before category registration and opening; repeated application is idempotent for the same panel
2. **UpdateMicroButtons** -- Stub out micro button updates
3. **Map Canvas Scroll** -- Initialize targetScale/currentScale on WorldMapFrame.ScrollContainer
4. **Status Bar Animations** -- Provide LevelUpMaxAlphaAnimation stub
5. **Character Frame Subframes** -- Create missing subframes from CHARACTERFRAME_SUBFRAMES list

---

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| Frame Creation | Complete | CreateFrame, templates, intrinsic children |
| Frame Anchoring | Complete | SetPoint, ClearAllPoints, GetAnchor |
| Events | Complete | RegisterEvent, fire_event |
| Script Handlers | Complete | SetScript, HookScript, OnClick, OnEvent, OnUpdate |
| Text Rendering | Partial | SetText, SetFont; width/height via cosmic-text |
| Textures | Partial | SetTexture, SetAtlas, SetTexCoord; atlas resolution |
| Animations | Partial | AnimationGroup creation; ticking implemented |
| Buttons | Complete | State-dependent texture switching |
| Sliders | Complete | Min/max, value, step |
| EditBox | Partial | SetText, GetText; no actual text input |
| Tooltips | Complete | SetOwner, AddLine, display |
| Cooldowns | Stub | SetCooldown stores duration; no swirl animation |
| Pools | Complete | CreateFramePool, CreateFrameFactory, Acquire/Release |
| Mixins | Complete | Mixin(), CreateFromMixins() |
| Timers | Complete | C_Timer.After, NewTicker, NewTimer |
| CVars | Complete | GetCVar, SetCVar |
| Slash Commands | Complete | Full dispatch |

---

## Key Files Map

| Path | Purpose |
|------|---------|
| `env.rs` | WowLuaEnv, timer loop, event dispatch |
| `state.rs` | SimState, PendingTimer, AddonInfo |
| `frame/handle.rs` | FrameHandle userdata |
| `frame/methods/mod.rs` | Method registration orchestrator |
| `frame/methods/methods_*.rs` | Categorized method implementations |
| `globals_legacy.rs` | Global function registration |
| `globals/create_frame.rs` | CreateFrame implementation |
| `globals/font_api.rs` | Font system + standard fonts |
| `globals/pool_api.rs` | Pool implementations |
| `globals/timer_api.rs` | C_Timer namespace |
| `globals/utility_api.rs` | Table, string, security functions |
| `globals/mixin_api.rs` | UI mixin tables |
| `globals/c_stubs_api.rs` | C_* namespace stubs |
| `animation/` | Animation types, ticking |
| `workarounds/mod.rs` | Post-load workaround ordering |
| `workarounds/temporary/settings_surface_defaults.rs` | Additive-only Settings defaults and replacement SettingsPanel/Settings reconciliation |
| `c_api/c_settings_util.rs` | `C_SettingsUtil.OpenSettingsPanel` forwarding surface |
