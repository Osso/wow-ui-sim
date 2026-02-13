# WoW UI Sim Brief - 2026-02-12

## Recently Fixed: Spec Node Filtering + Right-Click

### Spec filtering (FIXED)
Protection spec now correctly shows only ~40 spec nodes (was 119 from all three specs).

**Root cause:** `isVisible` in `GetNodeInfo` was always `true` for non-subtree nodes. Spec filtering happens via group-level conditions (condType=1) from `TraitNodeGroupXTraitCond` join table, not direct node conditions. The Blizzard UI relies on `isVisible` to decide whether to instantiate talent buttons.

**Fix:**
- Added `group_cond_ids` field to `TraitNodeInfo` (codegen: `gen_traits_load.rs`, `gen_traits_emit.rs`)
- `check_spec_conditions_met()` checks both node conditions and group conditions for condType=1
- `spec_set_contains_active_spec()` maps Paladin specSetIDs (27→Holy/65, 28→Protection/66, 29→Retribution/70) via SpecSetMember DB2 data from wago.tools
- Edges to hidden spec nodes are also filtered in `build_node_edges_dynamic`
- `evaluate_condition` also uses this for `create_condition_info` (isMet flag)

**Result:** visible=91 (51 class + 40 Protection), hidden=103, subtree=43

### Right-click (FIXED)
Right-click now fires `OnMouseDown`/`OnClick`/`PostClick`/`OnMouseUp` with `"RightButton"`, enabling talent refund via `RefundRank()`.

**Changes:**
- `CanvasMessage`: added `RightMouseDown`/`RightMouseUp` variants
- `render.rs`: handles `mouse::Button::Right` press/release
- `mouse.rs`: new module extracted from `update.rs` — all mouse event handlers
- `app.rs`: added `right_mouse_down_frame` field for tracking right-click target

## Prior Session Context

### Talent State Machine (commits c708903 through 207492e)

Implemented interactive talent selection/removal:
- `TalentState` struct with `node_ranks`, `node_selections`, `group_currency_map`, `node_currency_map`
- State-aware `GetNodeInfo`, `PurchaseRank`, `RefundRank`, `SetSelection`, `ResetTree`, `GetTreeCurrencyInfo`, `GetNodeCost`
- Dynamic `create_condition_info` (gate conditions check spent amounts)
- Edge `isActive` computed from source node ranks
- Events: `TRAIT_NODE_CHANGED` (per-node), `TRAIT_TREE_CURRENCY_INFO_UPDATED`, `TRAIT_CONFIG_UPDATED`
- Cross-subtree edge filtering, SubTreeSelection node hiding
- Performance fix: dirty tracking sets in WidgetRegistry (commit 0fee158)
- Line widget atlas UV fix + isActive computation (commit 5c8f28d)
- Extra OnUpdate ticks for headless modes (commit 207492e)

### Key Files
- `src/lua_api/globals/traits_api.rs` — C_Traits config/tree/node APIs, event firing
- `src/lua_api/globals/traits_api_node.rs` — GetNodeInfo, edge building, condition info, evaluate_condition, spec filtering
- `src/lua_api/state.rs` — TalentState, ensure_layout_rects, invalidate_layout
- `src/lua_api/talent_state.rs` — TalentState struct definition
- `src/widget/registry.rs` — WidgetRegistry with dirty tracking sets
- `src/iced_app/quad_builders.rs` — Line widget rendering with atlas UVs
- `src/iced_app/mouse.rs` — Mouse event handlers (left, right, middle, scroll, hover)
- `src/iced_app/render.rs` — Mouse event dispatch to CanvasMessage
- `src/iced_app/state.rs` — CanvasMessage enum
- `data/traits.rs` — Auto-generated trait data (includes group_cond_ids)

### OnUpdate error handling (FIXED)
OnUpdate handlers were permanently disabled after a single Lua error (`on_update_errors` HashSet in `env.rs`). This prevented the talent frame's deferred edge update from ever running again after any error, causing edge Line widgets to stop updating (and potentially disappear if they were released before the error).

**Root cause:** `fire_on_update_handlers` added errored frame IDs to a permanent HashSet. In real WoW, OnUpdate errors show an error popup but the handler keeps firing.

**Fix:** Changed `on_update_errors` from `HashSet<u64>` to `HashMap<u64, u32>` counting consecutive errors per frame. Handlers retry on next tick (count resets on success). Logging suppressed after 100 consecutive errors. Error messages include frame name for diagnostics.

**Edge update flow (Blizzard code):**
- `MarkEdgesDirty(button)` → just sets `buttonsWithDirtyEdges[button] = true` + RegisterOnUpdate
- `UpdateEdgesForButton(button)` → atomically releases old edges (Hide+ClearAllPoints) and acquires new ones (Init+Show) during OnUpdate
- Between click and OnUpdate, edges remain visible with stale state

### SetSelection nil deselect (FIXED)
Right-clicking a selection node to deselect crashed with `bad argument #3: error converting Lua nil to i32`.

**Root cause:** Blizzard code calls `C_Traits.SetSelection(configID, nodeID, nil, shouldClearEdges)` when deselecting. Rust binding required `(i32, i32, i32)`.

**Fix:** Changed `entry_id` parameter to `Option<i32>`. When `None`: removes ranks and selections (deselect/refund). When `Some`: sets selection and ensures rank >= 1.

### Line widget blend mode (FIXED)
`emit_line_vertices` in `quad_builders.rs` hardcoded `BlendMode::Alpha`, ignoring `f.blend_mode`. This caused FillScroll1/2 lines (alphaMode="ADD") to render with alpha blending instead of additive, making them too opaque.

**Fix:** Pass `f.blend_mode` through to `emit_line_vertices`.

## Debug: Showing Talent Panel on Load

The talent panel is a LoadOnDemand addon (`Blizzard_PlayerSpells`). It must be loaded explicitly.
Both `PlayerSpellsFrame:Show()` and `TalentsFrame:Show()` are needed (parent must be visible).

- **GUI mode:** Debug script `/tmp/debug-scrollbox-update.lua` works — runs before startup events but talent buttons are created when events fire during the GUI loop
- **Headless (screenshot/dump-tree):** Use `--exec-lua` — the debug script runs too early (before startup events), so talent buttons don't exist yet. `--exec-lua` runs after startup events + timers + OnUpdate.

```bash
# Screenshot with talent tree
wow-sim --no-addons --no-saved-vars --delay 100 --exec-lua \
  'C_AddOns.LoadAddOn("Blizzard_PlayerSpells"); PlayerSpellsFrame:Show(); PlayerSpellsFrame.TalentsFrame:Show()' \
  screenshot -o talents.webp --filter PlayerSpellsFrame
```

## Known Issues

### ApplyButton (Activate) — text/size FIXED, rendering still wrong
Text and size are now correct (164x22, "Apply Changes"). Two fixes were needed:

**Root cause 1 — Runtime template child size ordering (FIXED):**
`ClassTalentsFrameTemplate` is `virtual="true"`, so its children (including ApplyButton) are created by the runtime template loader (`template/mod.rs`). The child creation flow is:
1. `build_create_child_code` — creates frame, sets anchors
2. `apply_templates_from_registry` — applies inherited templates (sets template default sizes)
3. `apply_inline_frame_content` — applies instance-specific inline content

Previously, size was set in step 1 (`append_child_size_and_anchors`) and overwritten by step 2 (template defaults like UIButtonTemplate's 20x20). Step 3 never re-applied it.

**Fix:** Removed size from step 1 (now `append_child_anchors`). Added `elements::apply_inline_size()` in step 3 (`apply_inline_frame_content`). Templates set defaults first, then inline content overrides — matching WoW's property precedence.

**Root cause 2 — Missing text attribute in runtime template loader (FIXED):**
The `text=` XML attribute on Button elements wasn't applied during runtime template creation. Added `apply_button_text_attribute()` which resolves global strings and calls SetText.

**Remaining rendering issue:** NormalTexture/PushedTexture/DisabledTexture render as solid red overlays covering the three-slice children (Left/Right/Middle from `UI-Panel-Button-Up`). The `UIButtonTemplate` mixin sets `128-RedButton` atlas via `SetButtonArtKit()` in `InitButton()`, which overlaps the three-slice texture children.

### Edge lines hide too early in live GUI
When purchasing a talent in the live GUI, edge lines briefly or permanently disappear. The OnUpdate error fix helps (handlers no longer permanently disabled), but the root timing issue may still exist.

**Pool system:** Blizzard's `Pools.lua` (line 825) overwrites the Rust `CreateFramePool` shim. The real pools use `SecureObjectPoolMixin` with `Pool_HideAndClearAnchors` as the default resetter, which calls `Hide()` + `ClearAllPoints()` on release.

**Flow:** `ReleaseEdge` → `pool:Release()` → `Pool_HideAndClearAnchors` (Hide+ClearAllPoints) → then `AcquireEdge` → `pool:Acquire()` → `Init()` → `Show()`. This is atomic within one OnUpdate handler, so edges should be hidden then immediately replaced.

### Other
- Buff duration text missing in live GUI mode
- Script inheritance bug: `src/loader/helpers.rs` lines 297-299 prepend/append boolean swapped
