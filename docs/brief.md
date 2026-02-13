# WoW UI Sim Brief - 2026-02-12

## Completed: Talent Frame Performance + Edge Lines

### Two issues fixed:
1. **Performance: ensure_layout_rects O(47K) scan every frame** — FIXED (commit 0fee158)
2. **Multiple overlapping edge lines visible** — RESOLVED (commit 5c8f28d fixed atlas UVs + isActive)

### Performance Fix (ensure_layout_rects)

**Problem:** `ensure_layout_rects()` scanned ALL ~47K widgets every frame to find missing layout_rects and clear rect_dirty flags. Cost: **31ms/frame** — making the simulator crawl when the talent frame is open.

**Fix:** Added tracking sets to `WidgetRegistry`:
- `rect_dirty_ids: HashSet<u64>` — populated by `mark_rect_dirty_subtree`, drained by `ensure_layout_rects`
- `pending_layout_ids: HashSet<u64>` — populated by `register()` and `clear_all_layout_rects()`, drained by `ensure_layout_rects`

**Result:** First call processes pending set (199ms one-time at boot). Subsequent calls: **<1µs/frame** (from 31ms).

**Files modified:**
- `src/widget/registry.rs` — Added `rect_dirty_ids`, `pending_layout_ids` fields; `drain_rect_dirty()`, `drain_pending_layout()`, `mark_layout_resolved()` methods
- `src/lua_api/state.rs` — `ensure_layout_rects()` uses tracked sets instead of full scan; `recompute_layout_subtree` calls `mark_layout_resolved`; `clear_rect_dirty_subtree` uses `clear_rect_dirty`

Also moved one-time initial layout resolution to `boot()` so first render doesn't pay the cost.

### Edge Lines Issue (resolved)

**Symptoms:** Multiple colored edge lines visible simultaneously between talent nodes (gray + yellow, locked + active, etc.)

**Root cause:** Two issues in commit 5c8f28d:
- `build_line_quads` was hardcoding full texture UVs instead of using `f.tex_coords` from atlas
- `build_node_edges_dynamic` wasn't computing `isActive` from source node ranks

**Investigation confirmed:** Pool system (`SecureFramePoolCollectionMixin`) works correctly — 352 active edges, 0 duplicates, all GhostLines properly hidden. The 249 unique line positions vs 352 total is expected: multiple edges converge on the same talent node.

### Extra OnUpdate Ticks for Headless Modes

Added `run_extra_update_ticks()` helper that runs 3 cycles of `ensure_layout_rects` + `fire_one_on_update_tick` + `process_pending_timers` after exec-lua. Applied to both screenshot and dump-tree paths so deferred UI (talent frames, pool-created frames) can fully process.

### ClassTalentsFrame Visibility in Headless Mode

`PlayerSpellsFrame` uses `TabSystemOwnerMixin` (`SetTab`/`TrySetTab`) to show/hide content frames. The simulator doesn't implement TabSystem, so `TogglePlayerSpellsFrame()` alone doesn't show the talent tree content. Workaround: directly call `PlayerSpellsFrame.TalentsFrame:Show()` in exec-lua.

**File:** `src/main.rs`

## Prior Session Context

### Talent State Machine (commits c708903 through 5c8f28d)

Implemented interactive talent selection/removal:
- `TalentState` struct with `node_ranks`, `node_selections`, `group_currency_map`, `node_currency_map`
- State-aware `GetNodeInfo`, `PurchaseRank`, `RefundRank`, `SetSelection`, `ResetTree`, `GetTreeCurrencyInfo`, `GetNodeCost`
- Dynamic `create_condition_info` (gate conditions check spent amounts)
- Edge `isActive` computed from source node ranks
- Events: `TRAIT_NODE_CHANGED` (per-node), `TRAIT_TREE_CURRENCY_INFO_UPDATED`, `TRAIT_CONFIG_UPDATED`
- Cross-subtree edge filtering, SubTreeSelection node hiding

### Key Files
- `src/lua_api/globals/traits_api.rs` — C_Traits config/tree/node APIs, event firing
- `src/lua_api/globals/traits_api_node.rs` — GetNodeInfo, edge building, condition info
- `src/lua_api/state.rs` — TalentState, ensure_layout_rects, invalidate_layout
- `src/widget/registry.rs` — WidgetRegistry with dirty tracking sets
- `src/iced_app/quad_builders.rs` — Line widget rendering with atlas UVs
- `src/iced_app/update.rs` — Timer handler, ensure_layout_rects before OnUpdate
- `src/iced_app/app.rs` — boot() with eager layout resolution

### Blizzard UI Files (read, not modified)
- `Blizzard_SharedTalentFrame.lua` — TalentFrameBaseMixin: OnUpdate, LoadTalentTree, UpdateEdgesForButton, MarkEdgesDirty
- `Blizzard_SharedTalentEdgeTemplates.lua` — TalentEdgeArrowMixin: UpdateState (sets atlas based on edge state), UpdatePosition (checks IsRectValid)
- `Blizzard_ClassTalentEdgeTemplates.lua` — ClassTalentEdgeArrowMixin: parent matching, alpha visibility
- `Blizzard_TalentButtonBase.lua` — FullUpdate, UpdateNodeInfo, MarkEdgesDirty
- `Blizzard_TalentButtonArt.lua` — ApplyVisualState, UpdateStateBorder (SetAtlas with UseAtlasSize)

### Event Flow on Talent Change
1. `PurchaseRank`/`RefundRank` → updates `node_ranks`
2. `fire_node_changed_events` → `TRAIT_NODE_CHANGED` × 237 nodes
3. `fire_currency_updated_event` → `TRAIT_TREE_CURRENCY_INFO_UPDATED`
4. `fire_trait_config_updated` → `TRAIT_CONFIG_UPDATED`
5. Blizzard handlers mark nodes/edges dirty → RegisterOnUpdate
6. Next OnUpdate: processes dirty nodes → FullUpdate → UpdateStateBorder → edges

### IsRectValid Infinite Loop Risk
`TalentEdgeArrowMixin:UpdatePosition` (line 161-163): if `startButton:IsRectValid()` or `endButton:IsRectValid()` returns false → marks edges dirty → RegisterOnUpdate → next tick repeats. Our `IsRectValid` = `!anchors.is_empty() && !rect_dirty`. The `ensure_layout_rects` clears `rect_dirty` before OnUpdate, breaking the cycle. Without it → infinite dirty loop.

## Known Issues

- Buff duration text missing in live GUI mode
- Script inheritance bug: `src/loader/helpers.rs` lines 297-299 prepend/append boolean swapped
- TabSystem not implemented — headless talent screenshots require explicit `PlayerSpellsFrame.TalentsFrame:Show()`
