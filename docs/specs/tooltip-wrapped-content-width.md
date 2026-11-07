# Tooltip wrapped content width

Retail/PTR tooltip content width selected before outer padding: the intrinsic
cap, the unwrapped and explicit-minimum floors, shrink-to-fit, and the forced
wrapped size. Source: `src/lua_api/tooltip.rs` (`TooltipData::select_content_width`,
shared by both sizing paths), `src/iced_app/tooltip.rs` (glyph-measured layout)
and `src/lua_api/frame/methods/widgets/tooltip/sizing.rs` (estimated geometry for
builds without GUI). Native evidence: WM63-L23 section 2 and WM72, Retail
12.1.5.70688.

## What it must do

- [x] A wrapped row contributes its intrinsic left text width plus the row inset
      and its attachment width reservation, capped at the custom wrap value, or at
      230.4 when no custom value is stored.
- [x] The selected content width is the largest wrapped contribution together with
      `SetMinimumWidth`, so short wrapped-only content stays narrow.
- [x] A custom wrap value is a cap, not a floor: it never widens wrapped-only
      content and never imposes a universal minimum on the tooltip.
- [x] An explicit custom wrap value of zero is a present cap of zero, not a request
      to restore the default.
- [x] An unwrapped row contributes its measured visible left and right text, its
      row inset and attachment reservation, and 38.4 between two visible columns.
      The widest unwrapped contribution can exceed the wrapped cap and then
      determines the available wrapping width.
- [x] A nonempty right text on a double line disables that row's requested wrap
      flag; an empty or missing right text preserves the requested flag.
- [x] Shrink-to-fit starts enabled. A wrapped row is re-measured at its assigned
      width, and the widest fitted row can reduce the selected width.
- [x] The shrink cannot cross the unwrapped-row or explicit-minimum floor and
      cannot increase the selected width.
- [x] A right column on another row only raises that floor; it does not disable
      the shrink of the wrapped rows.
- [x] `SetMinimumWidth` with force enabled sends every row through wrapped sizing
      and disables the shrink path.
- [x] The selected content width is assigned to each wrapped left FontString after
      subtracting that row's inset and attachment reservation.
- [x] Estimated geometry for builds without GUI follows the same cap, floors and
      shrink floors instead of treating the custom value as a floor.

## How it works

- [WM63 L23 wrap and spacing](../../../../.local/state/wow-ui-sim/handoff/native-evidence-70688/tooltip/WM63-L23-wrap-spacing.md)
  section 2 is the native arithmetic source (outside the repository).
- [Tooltip layout timing](tooltip-layout-timing.md) covers when sizing runs and
  the row anchors, not the width selection.
- [Tooltip attachments](tooltip-attachments.md) covers the reservations this
  selection subtracts from the assigned wrap width.

## Implementation inventory

- `src/lua_api/tooltip.rs` — `TooltipWidthRow`, `TooltipData::row_wraps` and
  `TooltipData::select_content_width`: the single width-selection policy.
- `src/iced_app/tooltip.rs` — `measure_native_tooltip`: glyph measurement, fitted
  re-measurement and the wrap-width assignment to the left FontString.
- `src/lua_api/frame/methods/widgets/tooltip/sizing.rs` —
  `estimated_native_content_width`: the same policy with estimated text widths.

## Tests asserting this spec

- `tests/tooltip_wrap_width_lane.rs` — sibling right column does not disable
  shrink; the shrink stops at the unwrapped floor.
- `tests/tooltip_geometry_lane.rs` — custom value as cap; cap and
  `SetMinimumWidth(..., true)` assigned to the left FontString.
- `src/lua_api/frame/methods/widgets/tooltip/sizing.rs` (`mod tests`) —
  estimated geometry cap and cap-not-floor.
- `tests/tooltip_word_wrap_min_width.rs` — custom wrap width on wrapped-only
  content.

## Known gaps (current cycle)

- [ ] Native header/body font metrics, glyph advances, baselines and exact
      rendered line heights are UNKNOWN; the wrap-width arithmetic is exact only
      relative to the simulator's measured text.

## Out of scope

- FontString height measurement and per-row fonts (WM63 L23 section 3).
- Texture and atlas row attachment reservations (WM63 S8L).
- Owner anchors, cursor placement and owner clearing.
