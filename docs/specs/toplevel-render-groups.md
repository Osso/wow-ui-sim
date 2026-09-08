# Top-level render groups and raised levels

Top-level frame subtrees render in their top-level owner's strata group without changing Lua-visible raw strata or frame levels. An unraised owner still groups its descendants. Core ordering lives in `src/lua_api/state_render.rs` and `state_render_groups.rs`; see [rendering](../rendering-pipeline.md).

## What it must do

- [x] Keep a HIGH child of an unraised LOW top-level parent behind an independent MEDIUM panel, including after that panel is shown or raised.
- [x] Preserve independent HIGH, DIALOG, plain TOOLTIP and UIParent-parented GameTooltip controls above MEDIUM panels. Tooltip ownership is not parenting or raised-level inheritance.
- [ ] Retain local strata, frame-level and region order inside an owner group; emit its visible members once after child hide/show changes.
- [ ] Stop group and raised-level ancestry at UIParent and WorldFrame.
- [ ] Keep newly enabled top-level frames at raised level zero. Hide/Show and explicit Raise on a shown top-level frame advance the existing monotonic order; descendants share the active ancestor's value.
- [ ] Preserve raw strata/levels and ordinary non-top-level Raise/Lower behavior, including raised level zero.
- [ ] Keep the real SpellBook paper above the reproduced BetterBlizzFrames HIGH no-portrait overlay while preserving both frames' raw state.

## How it works

- [Rendering pipeline](../rendering-pipeline.md)
- [Native overlay investigation](../wiki/investigations/betterblizzframes-no-portrait-overlay.md)

## Implementation inventory

- `src/lua_api/state_render.rs` — raw buckets, top-level ownership, visibility and raised transitions.
- `src/lua_api/state_render_groups.rs` — assemble owner-strata groups while retaining their local ordering.
- `src/lua_api/frame/methods/misc/frame_level.rs` — parent-derived GetRaisedFrameLevel.

## Tests asserting this spec

- `tests/toplevel_render_groups.rs` — controlled native matrix, transitions, tooltip-owner independence and grouped visibility.
- `tests/spellbook.rs` — actual Blizzard panel and reproduced addon overlay render boundary.
- `src/lua_api/state_render_tests.rs` — existing active show-order, nested selection and ordinary Raise boundaries.
- `tests/frame_level.rs` — existing ordinary Raise/Lower and raw-level behavior.

## Native evidence

The 2026-09-08 retail `12.1.0.69587` controlled capture at physical 1440-pixel display height (`/tmp/pi-unit-controls-native.lua`, archived ignored at `docs/local/private/probes/UnitFrameLayerProbe-controls-2026-09-08.lua`) classifies case 1 BLUE and cases 2–5 RED in created, panel-hide-show, and panel-raise screenshots (`/tmp/pi-native-control-pixels.json`). The created MEDIUM controls report raised `0`; after hide/show they report `33..37`, and after explicit Raise `38..42`; LOW parent and red controls remain `0`. These values are observed transition ordinals, not constants or global ordering keys.

## Known gaps (current cycle)

- [ ] `f5d9ff91f` core grouping and `GetRaisedFrameLevel()` behavior require targeted GREEN after input wiring.
- [ ] Hit-grid integration, readability review, and full-scene GUI acceptance remain pending.

## Out of scope

No global promotion across independent strata, raw strata overrides, vendor/addon changes, or new counter state. Existing nested active-owner selection and Lower behavior are preserved; the probe does not establish new policies for those cases. Fixed-strata getter discrepancies are not changed by this slice.
