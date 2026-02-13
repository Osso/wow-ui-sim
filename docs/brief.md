# WoW UI Sim Brief - 2026-02-13

## Key Files
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

## In Progress: 2048 Atlas Tier (pixelated talent icons)

`atlas.rs`, `quad.wgsl`, `primitive.rs` updated to add tier 4 (2048px cells). `render.rs` and `quad_builders.rs` have incomplete atlas crop code — `load_texture_or_crop` and `remap_atlas_crop` referenced but not yet implemented. Needed so atlas sub-regions (40x40 icons from 2048x1024 textures) render at native resolution instead of being downscaled.

## Known Issues

### ApplyButton (Activate) — rendering wrong
NormalTexture/PushedTexture/DisabledTexture render as solid red overlays covering the three-slice children (Left/Right/Middle from `UI-Panel-Button-Up`). The `UIButtonTemplate` mixin sets `128-RedButton` atlas via `SetButtonArtKit()` in `InitButton()`, which overlaps the three-slice texture children.

### Edge lines hide too early in live GUI
When purchasing a talent in the live GUI, edge lines briefly or permanently disappear.

**Pool system:** Blizzard's `Pools.lua` (line 825) overwrites the Rust `CreateFramePool` shim. The real pools use `SecureObjectPoolMixin` with `Pool_HideAndClearAnchors` as the default resetter, which calls `Hide()` + `ClearAllPoints()` on release.

**Flow:** `ReleaseEdge` → `pool:Release()` → `Pool_HideAndClearAnchors` (Hide+ClearAllPoints) → then `AcquireEdge` → `pool:Acquire()` → `Init()` → `Show()`. This is atomic within one OnUpdate handler, so edges should be hidden then immediately replaced.
