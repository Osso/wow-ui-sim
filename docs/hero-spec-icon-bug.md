# Hero Spec Icon Position Bug

## Problem

The hero talent spec icon (large circular paladin fire icon with ring border) renders at the **bottom-right** of the talent panel instead of **top-center** where it belongs.

## Frame Hierarchy

```
PlayerSpellsFrame (scale=0.88)
  └─ ClassTalentsFrame (TalentsFrame)
      ├─ ButtonsParent (__tpl_25748, 1418x681, clipChildren=true)
      │   └─ [talent node buttons positioned via ApplyPosition]
      └─ HeroTalentsContainer (__tpl_25774, 200x800)
          │  anchor: TOP -> ButtonsParent:TOP offset(0,0)
          ├─ HeroSpecLabel (.HeroSpecLabel) "TEMPLAR"
          │    anchor: BOTTOM -> parent:TOP offset(0,23)
          ├─ HeroSpecButton (__tpl_25780, 108x108)
          │    anchor: TOP -> parent:TOP offset(0,-102)
          │    ├─ Icon1 (talentsheroclassicons) setAllPoints
          │    ├─ IconMask (common-mask-circle) setAllPoints
          │    └─ Border (talents-heroclass-ring-mainpane, 192x192)
          │         anchor: CENTER -> parent:CENTER offset(0,-2)
          └─ CurrencyFrame (30x30) "PV" / "0"
               anchor: CENTER -> HeroSpecButton:BOTTOM offset(0,-3)
```

Key: HeroTalentsContainer is a **sibling** of ButtonsParent (not a child), anchored to its TOP.

## Lua Positioning

`ClassTalentsFrameMixin:UpdateSpecBackground()` (Blizzard_ClassTalentsFrame.lua:204):
```lua
local heroContainerOffset = specVisuals and specVisuals.heroContainerOffset or 0;
self.HeroTalentsContainer:SetPoint("TOP", self.ButtonsParent, heroContainerOffset, 0);
```

4-arg SetPoint: `(point, relativeTo, offsetX, offsetY)` — relativePoint defaults to "TOP".

## Investigation Findings

### dump-tree shows correct position

```
__tpl_25774 [Frame] (176x704) x=711, y=61   anchor: TOP -> __tpl_25748:TOP -> (800,61)
  __tpl_25780 [Button] (95x95) x=752, y=151  anchor: TOP -> parent:TOP -> (800,163)
    .Icon1 [Texture] (95x95) x=752, y=151
    .Border [Texture] (168x168) x=715, y=115
```

### screenshot renders at bottom-right (~x=1000, y=610)

### No stale layout_rect

The `layout_rect` cached on every frame in this subtree **matches** the freshly computed rect (no `[layout_rect=...]` stale annotations in dump). This holds true even when dumping AFTER `ensure_layout_rects()` runs in the screenshot path.

### Identical loading sequence

Both dump-tree and screenshot follow the same startup:
1. `fire_startup_events` (OnShow -> UpdateSpecBackground -> SetPoint)
2. `apply_post_event_workarounds`
3. `rebuild_anchor_index`
4. `process_pending_timers`
5. `fire_one_on_update_tick`
6. `sleep(2s)` + `run_extra_update_ticks(3)`

### Rendering pipeline

`collect_sorted_frames` (frame_collect.rs:102) reads `f.layout_rect` directly — confirmed correct.
`emit_all_frames` (render.rs:190-192) converts to screen bounds with `UI_SCALE=1.0` — no transform.

## What's Ruled Out

- **Stale layout_rect**: Confirmed correct after ensure_layout_rects
- **SetPoint parsing**: 4-arg form correctly parsed (anchor shows TOP->TOP offset 0,0)
- **UI_SCALE**: Is 1.0, no transform
- **Duplicate frames**: Only 3 instances of talentsheroclassicons texture, all within HeroSpecButton
- **Pan offset**: Pan system moves individual talent buttons, not ButtonsParent position
- **clipChildren**: ButtonsParent clips, but HeroTalentsContainer is a sibling not a child

## Remaining Hypotheses

1. **Quad generation / GPU rendering**: The layout rect is correct but quads are generated or rendered at wrong screen coordinates. Need to inspect actual quad vertex positions.
2. **Masking hiding top icon**: Icon renders correctly at top but is masked/invisible, while a separate visual artifact appears at bottom. The `IconMask` and `HeroClassIconSheenMask` use `common-mask-circle` — if mask application is wrong, the icon could appear transparent at the correct position.
3. **Strata bucket ordering**: The frame might be in a wrong strata bucket, causing it to be rendered with a different effective position or behind other content.

## Debug Tools Added

`--dump-tree` flag on `screenshot` subcommand:
```bash
wow-sim screenshot --dump-tree __tpl_25774   # dump subtree after ensure_layout_rects
wow-sim screenshot --dump-tree               # dump all (no filter)
```

## Next Steps

- Add temporary logging in `emit_all_frames` to print quad vertices for the HeroSpecButton
- Check if the icon texture renders at (752,151) but is invisible (masked), and what's actually visible at (~1000,610)
- Inspect mask texture application for the circular mask
