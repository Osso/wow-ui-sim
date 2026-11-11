# Rectangle geometry queries

Lua frame geometry getters report a frame's resolved rectangle or its stored dimensions when no valid rectangle is available. Native evidence and implementation details belong in [rectangle-query validity](../wiki/investigations/rect-query-validity.md).

## What it must do

- [ ] Resolve dirty anchor dependencies before the dependent frame's ordinary position or size query.
- [ ] For both Retail 12.1.5.70688 and PTR 12.1.7.70668, use a dirty target's rectangle only after resolution, when anchor-invalidated is clear and cached rectangle-valid is set. A clean target skips its own layout resolution and the invalidated-flag check, but still requires cached validity.
- [ ] `IsRectValid()` reads the cached rectangle-valid flag. It does not resolve layout.
- [ ] `GetWidth()`, `GetHeight()` and `GetSize()` use the queried frame's stored dimensions when its resolved rectangle is invalid.
- [ ] `SetSize()` writes the same stored width and height values as the corresponding `SetWidth()` and `SetHeight()` calls.
- [ ] Keep render and hit-test geometry eligibility unchanged.
- [ ] Treat sibling chains that resolve without a valid containment-parent rectangle as INFERRED, not as verified native behavior.

## How it works

- [Rectangle query validity](../wiki/investigations/rect-query-validity.md)
- [Layout system](../wiki/systems/layout-system.md)

## Implementation inventory

- `src/widget/frame.rs` and `src/widget/frame_defaults.rs`: cached rectangle-validity and anchor-invalidated state.
- `src/layout.rs` and `src/lua_api/state_render.rs`: compute and cache geometry validity without changing the resolved render rectangle.
- `src/lua_api/rect_geometry.rs`: resolve geometry targets and publish position and size queries.
- `src/lua_api/frame/methods/core_state/helpers.rs` and `region.rs`: stored-size fallback and cache-only `IsRectValid()`.
- `src/widget/registry/anchor_edits.rs` and `mod.rs`: track layout and anchor invalidation.

## Tests asserting this spec

- `tests/rect_geometry.rs`: invalid-size fallback, cached validity, dirty and clean target queries, resolved dimensions, and setter equivalence.
- `src/loader/tests/layout_anchoring.rs`: public-Lua cached validity before and after a geometry query.

## Known gaps (current cycle)

- [ ] INFERRED (simulator mapping): a target with a clean own dirty entry but a dirty containment ancestor or anchor dependency is treated as effectively dirty because the resolver represents inherited invalidation through ancestor and dependency walks; this does not establish native propagation. No test asserts either distinction.
- [ ] Exact return-list arity, constructor defaults, complete fallback for mixed valid/invalid anchors, callbacks, and cycle outcomes remain UNKNOWN.
- [ ] The verified target gate does not prove behavior for every frame type or every edge-availability combination.

## Out of scope

Render and hit-test eligibility, vendor UI changes, and claims of complete native layout or cycle parity.
