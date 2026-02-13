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

## Known Issues

### ApplyButton (Activate) — no longer broken, low priority
The "solid red overlay" issue described previously is gone — NormalTexture/PushedTexture/DisabledTexture children exist (created unconditionally by `create_frame.rs:313-316` for all buttons) but have no texture set, so they render nothing. Three-slice children (Left/Right/Center) render correctly via atlas. Note: all buttons waste 4 empty texture widgets; could be optimized to create on demand only.

### Edge lines hide too early in live GUI — fixed
When purchasing a talent, `UpdateNodeInfo()` during OnUpdate dirtied button rects (via SetVisualState → SetFrameLevel). Arrow edge `UpdatePosition()` then called `IsRectValid()` which returned false (rect dirty), triggering `MarkEdgesDirty` which modified `buttonsWithDirtyEdges` mid-`pairs()` iteration → `"invalid key to next"` error, dropping edges from 158 to 57.

**Fix:** `IsRectValid()` now lazily resolves layout via `resolve_rect_if_dirty()` (like GetSize/GetWidth already do). Also added missing `issecretvalue`/`canaccessvalue`/`canaccesstable` stubs needed by `Pools.lua`/`SecureTypes.lua`.
