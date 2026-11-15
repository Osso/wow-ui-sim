# FontString Anchor-Derived Width

**VERIFIED (simulator code):** the simulator implements two-edge FontString sizing from resolved geometry. **INFERRED (Retail 12.1.5.70688):** native two-edge sizing is consistent with that model, but the public `GetWidth()` return path is not fully proved. **VERIFIED (reproduction record):** the six-symptom EllesmereUI reproduction remains incomplete.

## Native evidence: Retail 12.1.5.70688

### VERIFIED

- The rectangle calculator reads both resolved horizontal positions. The ordinary width-measurement path uses the valid rectangle span after scale conversion, before it considers stored width.
- A changed stored-size setter invalidates failed geometry state, invalidates the region and its dependents when needed, and queues layout work. The ordinary width-measurement path attempts to resolve dirty geometry before it reads a valid rectangle.
- The lower-level `SetPoint()` mutation path invalidates and queues layout when its change flag is set, unless the region is already dirty and queued. The full public `SetPoint()` path is not proved.
- `ClearAllPoints()` clears eligible anchor records, marks the rectangle dirty, and invalidates dependents when the rectangle needs a new calculation.
- The internal text-storage path clears the FontString text-width cache when stored text changes. The public `SetText()` connection to that path is inferred.
- When an edge is missing, stored width supplies the fallback edge. A nonzero stored width can also be returned when the FontString rectangle is not valid.

### INFERRED

- An ordinary completed `GetWidth()` call reports the resolved two-edge span.
- Two horizontal anchors can resolve against the same target and determine the width; the admitted evidence does not prove every point pairing.
- When the rectangle is valid, an explicit `SetWidth()` value does not replace the two resolved edges in the ordinary width-measurement result.
- With one horizontal anchor and zero stored width, the FontString width path uses its string-width measurement fallback.
- A public `SetText()` call that changes text reaches the internal text-storage path.

### UNKNOWN

- The renderer's cache-miss metric: full source-text width, wrapped visible width, or another metric.
- Whether the renderer uses the anchor span as its wrap width.
- Whether `SetPoint()`, `ClearAllPoints()`, or a parent resize clears the text-width cache. Geometry invalidation and text-width cache invalidation are separate operations.
- Whether every public `SetPoint()` call takes the lower-level change path described above.
- Native parity for Forever 1.60.1.70707 and other client builds; this evidence covers Retail 12.1.5.70688 only.

**VERIFIED (evidence provenance):** the source-backed labels above come from `~/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md`, sections L1, L1b, and L1c. This is static client-image analysis, not a live-client test.

## Simulator implementation and tests

- **VERIFIED (code history):** `eea53dc30` (`INFERRED: resolve FontString widths from anchors`) removes the stored-size shortcut from resolved width reads; `b96986e58` (`INFERRED: derive pinned width from FontString rect`) refined the temporary stored-width rewrite. The L1c follow-up removes that rewrite and the `SetPoint()` text-measurement refresh because they could impose UNKNOWN wrapping behavior. Commit `4f89b465e` (`INFERRED: preserve FontString height reads`) separately preserves stored FontString height reads. Each remaining inferred behavior stays in a separate commit with an `INFERRED` subject.
- **VERIFIED (simulator code):** two-edge `GetWidth()` reads use the resolved rectangle. `SetPoint()` no longer rewrites stored text width or refreshes wrapped-text measurements. The simulator does not clear a text-width cache on `SetPoint()` or parent resize.
- **VERIFIED (simulator tests):** default tests observe the resolved rectangle through `GetLeft()` and `GetRight()` when stored width conflicts with the anchor span, and after a parent size change. They do not assert the inferred public `GetWidth()` result or inspect internal width flags.
- **INFERRED (simulator tests):** tests behind `optional-tests` cover `GetWidth()` reporting the span and the single-anchor text-width fallback. Each test cites L1c.
- **UNKNOWN:** these simulator tests do not establish native renderer wrapping width, native text-cache invalidation, or native behavior outside Retail 12.1.5.70688.

**VERIFIED (simulator probe):** prior read-only runs measured stale FontString widths after anchor or parent-size changes and an action-button HotKey width that did not follow its measured anchor span. The addon-level run loaded 3 of 20 installed addons and EllesmereUI core reported a nil-call error. It did not reproduce all six symptoms or establish their causes. **UNVERIFIED:** the issue screenshots' exact addon package, SavedVariables, and UI scale.

## Sources

- `~/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md` — native L1/L1b/L1c evidence for Retail 12.1.5.70688
- [text.rs](../../../src/lua_api/frame/methods/text_attribute_event/text.rs) — simulator FontString width and height measurement
- [anchors.rs](../../../src/lua_api/frame/methods/button_anchor_hierarchy/anchors.rs) — simulator `SetPoint()` handling
- [helpers.rs](../../../src/lua_api/frame/methods/core_state/helpers.rs) — simulator `GetWidth()`/`GetHeight()` geometry reads
- [fontstring_anchor_pinned_width.rs](../../../tests/fontstring_anchor_pinned_width.rs) — verified rectangle tests and optional inferred `GetWidth()` tests
- [fontstring_smooth_scaling.rs](../../../tests/fontstring_smooth_scaling.rs) — separate wrapped line-count regression
- [anchor-resolution.md](../../anchor-resolution.md) — simulator anchor resolution
- [FontString Default Anchors](fontstring-default-anchors.md) — separate XML implicit-anchor behavior
- `/home/osso/.local/state/wow-ui-sim/handoff/issue22-layout/runtime-probe.lua` — read-only issue probe

## See Also

- [[fontstring-default-anchors]] — implicit XML anchor selection
- [[layout-system]] — resolved frame geometry
