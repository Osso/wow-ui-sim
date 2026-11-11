# Lua rectangle-query validity

`verified: 2026-11-11`

VERIFIED (R48): native geometry resolution follows anchor targets and checks the queried object's validity, not containment ancestors. INFERRED (R48): a child can have a valid rectangle under an unanchored parent when its own anchor target resolves.

## Symptoms and cause

VERIFIED: the PTR `CharacterBag0Slot` test failed with `expected numeric result, got nil`; this does not establish the Lua return count. VERIFIED (PTR source): the button is contained by unanchored `BagSlotCluster` but anchors directly to sibling `MainMenuBarBackpackButton`. VERIFIED (simulator source): Lua rect queries reused `frame_has_render_layout`, which checked containment ancestors and rejected the child.

## R48 findings

- VERIFIED: position getters resolve dirty geometry, follow anchor targets, and check the queried object's rectangle validity; containment ancestors do not need rectangles unless an anchor uses them.
- INFERRED: a child with a resolvable sibling target returns numeric position and size values under an unanchored parent.
- INFERRED: invalid position queries return no usable coordinates; invalid rectangles use stored dimensions for width and height queries. UNKNOWN: exact Lua return-list arity for these branches.
- VERIFIED: `IsRectValid` reads cached validity and does not run layout first.
- UNKNOWN: exact `CharacterBag0Slot` coordinates and every protected return branch.

## R59 findings

- VERIFIED (PTR 12.1.7.70668 and Retail 12.1.5.70688): the measured target-use gate has the same masks and test order in both profiles.
- VERIFIED: a dirty target is resolved; the anchor path rejects its rect when anchor-invalidated mask 4 is set, then requires rect-valid mask 1 before using the rect.
- VERIFIED: a clean target skips resolution and the immediate mask-4 check, but still requires mask 1.
- INFERRED (simulator mapping): the simulator treats a target as effectively dirty when its own entry, containment ancestor, or anchor dependency requires resolution; it uses ancestor and dependency walks to represent invalidation propagation, which does not establish native propagation.
- UNKNOWN: complete fallback with mixed valid and invalid anchors, callback effects, general cycle policy, exact Lua return-list arity, and constructor defaults remain unmodeled.

## Simulator change

VERIFIED (simulator commits `f255ed881` and `20cbd611a`): the Lua-query-only validity check follows explicit anchor targets and implicit parent targets, and size getters use stored dimensions when the resolved queried rect is invalid. VERIFIED (simulator commit `faf99f277`): each frame has cached rectangle-valid and anchor-invalidated state maintained during layout recomputation. VERIFIED (simulator commits `faf99f277`, `edf70955b` and `45fae8c13`): `IsRectValid` reads only cached validity; the query walker inspects anchor chains, resolves changed dependencies before dependents, checks anchor-invalidated only when a target or its dependency chain was resolved, and requires cached rect-valid for both effectively dirty and clean targets. VERIFIED (diff): render and hit-test eligibility predicates remain unchanged.

INFERRED (simulator mapping, not native evidence): effective dirty state includes dirty containment ancestors and anchor dependencies because the registry and query walker represent inherited invalidation through ancestor and dependency walks.

## Verification status

VERIFIED (test inventory): `tests/rect_geometry.rs` covers invalid-size fallback, cached validity, direct dirty-target resolution, clean cached-target use, resolved dimensions and setter equivalence through Lua methods; loader tests also cover cached `IsRectValid` behavior through Lua. No test asserts ancestor-only or dependency-only dirtiness. VERIFIED (build receipt): the Retail library/integration build at HEAD `0834e5438` includes code commit `45fae8c13` and completed with zero warnings. VERIFIED (Retail runtime): six public-Lua `rect_geometry` tests and two cached-validity loader tests pass; the original CVar file's stat and SHA-256 remained unchanged. UNVERIFIED: PTR coverage, remaining geometry selectors, the profile matrix, and post-fix PTR integration/lib suite remain pending. See the [PTR cleanup report](../../../handoff/ptr-int-clean/REPORT.md) for suite status and receipts.

## Sources

- R48 and R59 native findings in `~/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md`
- [Rectangle geometry query contract](../../specs/rectangle-geometry-queries.md)
- [rect_geometry.rs](../../../src/lua_api/rect_geometry.rs) — Lua rectangle getters and query validity
- [core_state/helpers.rs](../../../src/lua_api/frame/methods/core_state/helpers.rs) — size query behavior
- [region.rs](../../../src/lua_api/frame/methods/core_state/region.rs) — existing `IsRectValid` behavior
- [layout_render_eligibility.rs](../../../src/layout_render_eligibility.rs) — unchanged rendering and hit-test predicate
- [rect_geometry.rs tests](../../../tests/rect_geometry.rs) — regression cases
- [PTR cleanup report](../../../handoff/ptr-int-clean/REPORT.md) — full triage and verification receipts

## See Also

- [[layout-system]] — anchor resolution and frame geometry
- [Frame-position profile baselines](../../specs/frame-position-profile-baselines.md)
