# Tooltip Layout Timing

Retail/PTR tooltip content sizing runs synchronously at native operation boundaries, not during rendering. The [contract](../../specs/tooltip-layout-timing.md) separates WM63/WM95 native evidence from emulated text limits.

## Current boundaries

- Line insertion, minimum width, wrapping and spacing setters store changes without resizing. `Show` and padding changes measure the current FontStrings before returning.
- `Show` without an owner, or with no rows unless allow-empty is enabled, runs tooltip hiding instead. The original `TooltipDataHandlerMixin:ProcessInfo` flow exposed a missing rejection: an empty DTO left the tooltip shown. The shared visibility path now clears and releases ownership without sizing rejected content.
- `AppendText` targets the first pooled left FontString's current text, not the last stored row. Its prior DTO-only update disagreed with glyph measurement, which reads pooled text. The producer now updates both representations before layout. A two-row original `ProcessInfo` fixture rejects the old implementation; the backpack hover is preservation coverage.

## Evidence limits

WM63 L4/L23 and WM95 establish bounded Retail 70688 static semantics, not full rendered-client parity. PTR uses the same modeled widget behavior. The UTF-8-safe 255-byte append cap is an explicit emulated seam. Native default font metrics, final cursor clamping, owner auto-hide and complete secret-aspect propagation remain unknown.

The historical one-frame render-order investigation predates this operation-boundary model. Moving sizing into each render would now violate the Retail/PTR timing contract.

## Sources

- [Timing and append contract](../../specs/tooltip-layout-timing.md)
- [Visibility transitions](../../../src/lua_api/frame/methods/core_state/visibility.rs)
- [Tooltip line methods](../../../src/lua_api/frame/methods/widgets/tooltip/line_data.rs)
- [Glyph measurement](../../../src/iced_app/tooltip.rs)
- [Original-consumer regression tests](../../../tests/tooltip_hover.rs)

## See Also

- [[tooltip-owner-lifecycle]] — historical Wrath lifecycle evidence, separate from this Retail/PTR slice
- [[lua-api]] — widget API boundary
