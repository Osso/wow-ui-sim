# Keybinding System

## Overview

The simulator implements WoW's keybinding system: actions are defined with Lua code to execute, and keys are mapped to those actions. When a key is pressed and no EditBox has focus, the binding fires.

**Key Files:**
- `src/lua_api/keybindings.rs` - Binding storage, defaults, and dispatch
- `src/lua_api/key_dispatch.rs` - Key press pipeline (integrates bindings)
- `src/lua_api/globals/strings/mod.rs` - Lua API functions (GetBindingKey, SetBinding, etc.)
- `src/iced_app/keybinds.rs` - iced key → WoW key name conversion

---

## Architecture

### Data Storage

Bindings are stored in two Lua registry tables:

| Table | Purpose | Example |
|-------|---------|---------|
| `__wow_binding_actions` | Action definitions (action → lua code) | `OPENALLBAGS → { action="OPENALLBAGS", lua_code="ToggleAllBags()" }` |
| `__wow_key_bindings` | Key assignments (key → action) | `B → "OPENALLBAGS"` |

The `__wow_binding_actions` table also stores action names by numeric index (1-based) for `GetBinding(index)` / `GetNumBindings()` enumeration.

### Key Press Pipeline

```
iced KeyPressed event
  → iced_key_to_wow() conversion (keybinds.rs)
  → Message::KeyPress(key_name)
  → WowLuaEnv::send_key_press(key)
    → ESCAPE: special dispatch (UISpecialFrames, GameMenuFrame)
    → Other keys:
      1. Focused EditBox special handlers (OnEnterPressed, OnTabPressed, OnSpacePressed)
      2. If NOT EditBox focused: check __wow_key_bindings → execute Lua code
      3. OnKeyDown dispatch with parent propagation
```

---

## Default Bindings

### Actions (from Bindings_Standard.xml)

| Action | Lua Code |
|--------|----------|
| `TOGGLEGAMEMENU` | `ToggleGameMenu()` |
| `TOGGLEBACKPACK` | `ToggleBackpack()` |
| `TOGGLEBAG1-4` | `ToggleBag(4)` through `ToggleBag(1)` |
| `OPENALLBAGS` | `ToggleAllBags()` |
| `TOGGLECHARACTER0` | `ToggleCharacter("PaperDollFrame")` |
| `TOGGLECHARACTER2` | `ToggleCharacter("ReputationFrame")` |
| `TOGGLESPELLBOOK` | `PlayerSpellsUtil.ToggleSpellBookFrame()` |
| `TOGGLETALENTS` | `PlayerSpellsUtil.ToggleClassTalentFrame()` |
| `TOGGLEACHIEVEMENT` | `ToggleAchievementFrame()` |
| `TOGGLEGROUPFINDER` | `PVEFrame_ToggleFrame()` |
| `TOGGLECOLLECTIONS` | `ToggleCollectionsJournal()` |
| `TOGGLEENCOUNTERJOURNAL` | `ToggleEncounterJournal()` |
| `TOGGLEWORLDMAP` | `ToggleWorldMap()` |
| `TOGGLESOCIAL` | `ToggleFriendsFrame()` |
| `TOGGLEGUILDTAB` | `ToggleGuildFrame()` |
| `TOGGLEQUESTLOG` | `ToggleQuestLog()` |

### Default Key Assignments

| Key | Action | Panel |
|-----|--------|-------|
| `BACKSPACE` | `TOGGLEBACKPACK` | Backpack |
| `F8`-`F11` | `TOGGLEBAG1`-`TOGGLEBAG4` | Individual bags |
| `B` | `OPENALLBAGS` | All bags |
| `C` | `TOGGLECHARACTER0` | Character sheet |
| `U` | `TOGGLECHARACTER2` | Reputation |
| `S` | `TOGGLESPELLBOOK` | Spellbook |
| `N` | `TOGGLETALENTS` | Talents |
| `A` | `TOGGLEACHIEVEMENT` | Achievements |
| `L` | `TOGGLEGROUPFINDER` | LFG / Group Finder |
| `O` | `TOGGLESOCIAL` | Friends list |
| `J` | `TOGGLEGUILDTAB` | Guild |
| `M` | `TOGGLEWORLDMAP` | World map |

Note: Some keys differ from WoW defaults (WoW uses P for spellbook, Y for achievements, I for LFG). These are simulator-specific overrides for convenience.

---

## Lua API Functions

### Query Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `GetBindingKey` | `(action) → key1 [, key2]` | Keys bound to an action (up to 2) |
| `GetBindingKeyForAction` | `(action, ...) → key` | First key bound to an action |
| `GetBindingAction` | `(key [, checkOverride]) → action` | Action bound to a key (empty string if none) |
| `GetBinding` | `(index) → action, header, key1 [, key2]` | Binding at 1-based index |
| `GetNumBindings` | `() → count` | Total number of defined actions |
| `GetCurrentBindingSet` | `() → 1` | Always returns 1 (character-specific) |
| `GetBindingText` | `(key [, prefix, abbrev]) → text` | Display-friendly key name (passthrough) |

### Mutation Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `SetBinding` | `(key, action) → true` | Bind a key to an action (nil action unbinds) |
| `SetBindingClick` | `(key, button [, mouseButton]) → true` | Stub (no-op) |
| `SetBindingSpell` | `(key, spell) → true` | Stub (no-op) |
| `SetBindingItem` | `(key, item) → true` | Stub (no-op) |
| `SetBindingMacro` | `(key, macro) → true` | Stub (no-op) |

### Persistence Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `SaveBindings` | `(which)` | No-op (bindings reset each session) |
| `LoadBindings` | `(which)` | No-op |

---

## Adding New Bindings

To add a new binding action, edit `src/lua_api/keybindings.rs`:

1. Add to `BINDING_ACTIONS` array:
   ```rust
   BindingAction { action: "MYACTION", lua_code: "MyFunction()" },
   ```

2. Optionally add a default key in `DEFAULT_KEYS`:
   ```rust
   DefaultKey { key: "K", action: "MYACTION" },
   ```

The Lua code in `lua_code` is executed directly via `lua.load(&code).exec()` — it must be valid Lua that's callable in the global environment after UI loading.

---

## Key Name Reference

Keys are converted from iced keyboard events in `src/iced_app/keybinds.rs`:

| iced Key | WoW Name |
|----------|----------|
| `Named::Escape` | `ESCAPE` |
| `Named::Enter` | `ENTER` |
| `Named::Tab` | `TAB` |
| `Named::Space` | `SPACE` |
| `Named::Backspace` | `BACKSPACE` |
| `Named::Delete` | `DELETE` |
| `Named::ArrowUp/Down/Left/Right` | `UP` / `DOWN` / `LEFT` / `RIGHT` |
| `Named::Home` / `Named::End` | `HOME` / `END` |
| `Named::PageUp` / `Named::PageDown` | `PAGEUP` / `PAGEDOWN` |
| `Named::Insert` | `INSERT` |
| `Named::F1` through `Named::F12` | `F1` through `F12` |
| `Character("a")` | `A` (uppercased) |

Modifier keys (Shift, Ctrl, Alt) are not currently combined into key names (no `SHIFT-B` style bindings yet).
