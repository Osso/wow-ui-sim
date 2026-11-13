# FontString Anchor-Derived Width

The simulator now uses the resolved horizontal span for a FontString whose left and right edges anchor to the same target. **INFERRED:** this matches native two-edge sizing; the available native evidence does not verify that rule. Issue #22’s full EllesmereUI layout remains unreproduced.

## Findings

- **VERIFIED (simulator):** the focused tests cover text set before anchors, left/right offsets, a zero-stored-width target with resolved geometry, parent resize, and the unchanged single-anchor auto-width case.
- **VERIFIED (simulator):** setting FontString anchors refreshes text measurements. With two opposite horizontal edges on one target, width comes from the resolved FontString rectangle and is no longer marked text-auto; the resolved rectangle also lets width follow parent resizing.
- **VERIFIED (code inspection):** `frame_size` returns the resolved FontString width but preserves its stored height when the stored width is positive. The new focused tests do not establish a separate native height contract.
- **INFERRED (native behavior):** two-edge anchors determine FontString width. L1 evidence for `GetStringWidth` cache behavior does not establish two-edge sizing or its invalidation rules.
- **VERIFIED (runtime probe):** the simulator probe at default and 3840×2160 sizes measured stale FontString widths after anchor or parent-size changes, and an action-button HotKey width that did not follow its measured anchor span.
- **VERIFIED (reproduction limit):** the addon-level run loaded 3 of 20 installed addons and EllesmereUI core reported a nil-call error. It does not establish all six issue symptoms or their causes. **UNVERIFIED:** the issue screenshots’ exact addon package, SavedVariables, and UI scale.

## Change and coverage

The width behavior is isolated in commits `eea53dc30` and `b96986e58`, both labeled `INFERRED`; `4f89b465e` preserves stored FontString height reads. The simulator change is in `src/lua_api/frame/methods/text_attribute_event/text.rs`, with the `SetPoint` refresh in `src/lua_api/frame/methods/button_anchor_hierarchy/anchors.rs`. Regression cases are in `tests/fontstring_anchor_pinned_width.rs`.

The tests prove the simulator’s stated behavior. They do not prove native WoW parity, the cause of the other EllesmereUI symptoms, or that the issue’s screenshots used the locally installed v9.8 package.

## Sources

- [text.rs](../../../src/lua_api/frame/methods/text_attribute_event/text.rs) — FontString width and height measurement refresh
- [anchors.rs](../../../src/lua_api/frame/methods/button_anchor_hierarchy/anchors.rs) — `SetPoint` refresh for FontStrings
- [fontstring_anchor_pinned_width.rs](../../../tests/fontstring_anchor_pinned_width.rs) — focused regression cases
- [anchor-resolution.md](../../anchor-resolution.md) — simulator anchor resolution
- [FontString Default Anchors](fontstring-default-anchors.md) — separate XML implicit-anchor behavior
- `/home/osso/.local/state/wow-ui-sim/handoff/issue22-layout/runtime-probe.lua` — read-only issue probe

## See Also

- [[fontstring-default-anchors]] — implicit XML anchor selection
- [[layout-system]] — resolved frame geometry
