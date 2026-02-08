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
- **getmetatable** -- Returns fake metatable with `__index` for all frame methods
- **string.format** -- Converts `%F` -> `%f` (WoW LuaJIT vs standard Lua 5.1)

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

**Security:** `issecure()`, `issecurevariable()`, `securecall()`, `securecallfunction()`, `forceinsecure()`, `hooksecurefunc()`, `SecureCmdOptionParse()`

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
| C_Item | `c_item_api.rs` | GetItemInfo, GetItemCooldown (stubs) |
| C_System | `c_system_api.rs` | GetLocale -> "enUS" |
| C_EditMode | `c_editmode_api.rs` | GetLayouts |
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

**File:** `src/lua_api/globals/cvar_api.rs`

`GetCVar()`, `SetCVar()`, `GetCVarBool()`, `GetCVarNumber()` backed by `SimState.cvars: HashMap<String, String>`.

---

## Workarounds

**File:** `src/lua_api/workarounds.rs`

Applied after addon loading via `env.apply_post_load_workarounds()`:

1. **UpdateMicroButtons** -- Stub out micro button updates
2. **Map Canvas Scroll** -- Initialize targetScale/currentScale on WorldMapFrame.ScrollContainer
3. **Status Bar Animations** -- Provide LevelUpMaxAlphaAnimation stub
4. **Character Frame Subframes** -- Create missing subframes from CHARACTERFRAME_SUBFRAMES list

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
| `workarounds.rs` | Post-load workarounds |
