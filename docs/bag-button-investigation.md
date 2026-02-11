# Bag Button Investigation

## Problem

Bag buttons (CharacterBag0Slot through CharacterBag3Slot) appeared greyed-out in the simulator.

## Root Causes Found

### 1. `GetInventorySlotInfo` returned nil texture (FIXED, committed)

`GetInventorySlotInfo(slotName)` returned `(slotId, nil, false)` instead of `(slotId, fileDataID)`. The Blizzard code in `PaperDollItemSlotButton_OnLoad` unpacks this and calls `icon:SetTexture(textureName)` — with nil, no texture was set.

**Fix**: Return fileDataIDs matching WoW's modern API. Added `slot_texture_file_data_id()` mapping all 19 equipment slot names to their WoW fileDataIDs (e.g., `Bag0Slot → 136511`). Also converted 19 paperdoll BLP textures to WebP.

Files: `src/lua_api/globals/c_item_api.rs`, `textures/paperdoll/`

### 2. `ContainerFrame_GetContainerNumSlots` stub returned 0 (FIXED, committed)

The stub in `c_stubs_api.rs` always returned 0, telling `BaseBagSlotButtonMixin:UpdateTextures()` that bags had zero slots. This selected the `bag-border-empty` atlas (dark background with embedded bag icon silhouette) instead of `bag-border` (transparent-center golden ring).

**Fix**: Stub now delegates to `bag_slot_count()` which returns 16 for bags 0-4.

Files: `src/lua_api/globals/c_stubs_api.rs`, `src/lua_api/globals/c_container_api.rs`

### 3. `ItemContextOverlay` rendered black 80% overlay on bag icons (FIXED)

`ItemButton`'s `PostOnShow` calls `UpdateItemContextMatching()` during creation. This calls `self:GetItemContextMatchResult()` which for bag buttons calls `ItemButtonUtil.GetItemContextMatchResultForContainer()` → `ItemButtonUtil.GetItemContext()`. `GetItemContext()` calls unstubbed `C_Spell.TargetSpellReplacesBonusTree()` etc., causing errors. The pcalled `UpdateItemContextMatching` fails, leaving `self.itemContextMatchResult = nil`.

Later, `PLAYER_ENTERING_WORLD` fires `UpdateBagMatchesSearch` → `SetMatchesSearch(true)` → `UpdateItemContextOverlay` → `GetItemContextOverlayMode`. With `itemContextMatchResult = nil`: `nil ~= DoesNotApply(3)` → `contextApplies = true` → returns `Standard` → `SetColorTexture(0,0,0,0.8)` + `SetShown(true)`.

**Fix**: Post-event workaround directly sets `btn.itemContextMatchResult = DoesNotApply` and calls `UpdateItemContextOverlay()` to clear the overlay.

Files: `src/lua_api/workarounds_bags.rs`

## Investigation Notes

### dump-tree shows 0x0 for anchor-dependent frames

`dump.rs` prints `frame.width x frame.height` directly, not the computed layout rect. Frames that derive size from anchors (e.g., `SetAllPoints`) show `(0x0)` in the dump even though `compute_frame_rect_cached` correctly resolves them. The rendering pipeline uses `frame.layout_rect` (populated by `ensure_layout_rects()`), not `frame.width`/`frame.height`.

### Bag button texture flow

1. `PaperDollItemSlotButton_OnLoad` → `GetInventorySlotInfo(slotName)` → `icon:SetTexture(fileDataID)`
2. `BaseBagSlotButtonMixin:UpdateTextures()` → `ContainerFrame_GetContainerNumSlots(bagID)` → selects atlas:
   - `bag-border` (slots > 0): golden ring, transparent center
   - `bag-border-empty` (slots = 0): golden ring, dark embedded bag icon
3. Atlas lookup falls back from `bag-border` → `bag-border-2x` (only `-2x` variants exist in atlas data)

### Workaround timing

`Blizzard_MainMenuBarBagButtons` loads before `Blizzard_UIPanels_Game` (which defines the real `ContainerFrame_GetContainerNumSlots`). The workaround in `workarounds_bags.rs:update_bag_button_textures()` re-runs `UpdateTextures()` after all addons load to fix this ordering issue.

The `ItemContextOverlay` fix runs in `apply_post_event` (after startup events) because the overlay is set visible by `PLAYER_ENTERING_WORLD` → `SetMatchesSearch` which fires during startup events.
