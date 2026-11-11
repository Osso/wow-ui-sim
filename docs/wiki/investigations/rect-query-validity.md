# Lua rectangle-query validity

`verified: 2026-11-11`

VERIFIED (R48): native geometry resolution follows anchor targets and checks the queried object's validity, not containment ancestors. INFERRED (R48): a child can have a valid rectangle under an unanchored parent when its own anchor target resolves.

## Symptoms and cause

VERIFIED: the PTR `CharacterBag0Slot` test failed with `expected numeric result, got nil`; this does not establish the Lua return count. VERIFIED (PTR source): the button is contained by unanchored `BagSlotCluster` but anchors directly to sibling `MainMenuBarBackpackButton`. VERIFIED (simulator source): Lua rect queries reused `frame_has_render_layout`, which checked containment ancestors and rejected the child.

## R48 findings

- VERIFIED: position getters resolve dirty geometry, follow anchor targets, and check the queried object's rectangle validity; containment ancestors do not need rectangles unless an anchor uses them.
- INFERRED: a child with a resolvable sibling target returns numeric position and size values under an unanchored parent.
- INFERRED: invalid position queries return zero Lua values; invalid rectangles do not make width and height getters return no values. Size getters use stored dimensions when no resolved rectangle is available.
- VERIFIED: `IsRectValid` reads cached validity and does not run layout first.
- UNKNOWN: exact `CharacterBag0Slot` coordinates and every protected return branch.

## Simulator change

VERIFIED (simulator commit `f255ed881`): the Lua-query-only validity check follows explicit anchor targets and implicit parent targets, and rect getters use it after resolving dirty geometry. VERIFIED: size getters use resolved dimensions for valid rectangles and stored dimensions otherwise. VERIFIED (diff): `frame_has_render_layout` remains unchanged, so this change does not alter rendering or hit-test eligibility.

VERIFIED (simulator source): `IsRectValid` remains unchanged; it resolves dirty layout when a frame has anchors, then returns `has_anchors && !is_rect_dirty`. The inspected source search found no separate `rect_valid` field. Follow-up: reconcile this eager resolution with R48's cached read after identifying an appropriate layout-maintained validity state.

## Verification status

VERIFIED (test inventory): the new regression tests cover an invalid unanchored frame and a child anchored to a sibling under an unanchored parent. VERIFIED (work status, 2026-11-11): these tests have not yet run on this commit. PTR bag and ActionBar selectors, `layout` and `frame_positions` on Retail/PTR/Forever/Mists, and the post-fix PTR integration/lib suite remain pending. VERIFIED (full-suite status): the pre-R48 PTR attempt for `18c174268` ended with integration and lib exit 101; the master baseline is still pending. See the [PTR cleanup report](../../../handoff/ptr-int-clean/REPORT.md) for the suite and source evidence.

## Sources

- R48 native findings in `~/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md`
- [rect_geometry.rs](../../../src/lua_api/rect_geometry.rs) — Lua rectangle getters and query validity
- [core_state/helpers.rs](../../../src/lua_api/frame/methods/core_state/helpers.rs) — size query behavior
- [region.rs](../../../src/lua_api/frame/methods/core_state/region.rs) — existing `IsRectValid` behavior
- [layout_render_eligibility.rs](../../../src/layout_render_eligibility.rs) — unchanged rendering and hit-test predicate
- [rect_geometry.rs tests](../../../tests/rect_geometry.rs) — regression cases
- [PTR cleanup report](../../../handoff/ptr-int-clean/REPORT.md) — full triage and verification receipts

## See Also

- [[layout-system]] — anchor resolution and frame geometry
- [Frame-position profile baselines](../../specs/frame-position-profile-baselines.md)
