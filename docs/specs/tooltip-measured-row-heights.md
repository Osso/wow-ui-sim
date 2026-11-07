# Tooltip measured row heights

Retail/PTR tooltip row heights come from the row's own left FontString, not from a
fixed font-size pair or a universal line-height multiplier. Source:
`src/iced_app/tooltip.rs` (`measure_native_tooltip`, `native_line_font`) and
`src/lua_api/frame/methods/widgets/tooltip/line_frames.rs` (row FontString
creation and declared row fonts). Native evidence: WM63-L23 section 3, Retail
12.1.5.70688.

## What it must do

- [x] A row's height is the measured height of that row's left FontString, using
      the FontString's font file, font size, text scale, line spacing and
      smooth-scaling flag.
- [x] A wrapped row is measured at its assigned wrap width minus the row inset
      and attachment reservation, so a wrapped description produces a row
      several text lines tall.
- [x] The tooltip height is the outer padding (20.48 plus explicit top and
      bottom), plus each visible row's measured height, plus the inter-row
      spacing before every visible row after the first, plus that row's
      attachment overflow.
- [x] Row heights are not a fixed constant times the row index: a header row and
      a body row measured in different fonts have different heights.
- [x] The measured row height is published on the row FontString, so the
      rendered rows and the tooltip frame height use the same number.
- [x] A row after the two declared font slots measures the tooltip's declared
      body font (`textLeft2Font`/`textRight2Font`) rather than the frame default.

## How it works

- Row fonts: a tooltip template declares two font slots, row 1
  (`textLeft1Font`/`textRight1Font`) and row 2 (`textLeft2Font`/`textRight2Font`),
  in `Blizzard_SharedXML/SharedTooltipTemplates.xml` lines 12-15, and
  `SharedTooltip_OnLoad` (`SharedTooltipTemplates.lua` lines 16-19) applies them
  to the two XML FontStrings. Later rows have no declared font of their own, so
  the simulator applies the declared body font to the FontString it synthesizes
  for them.
- Measurement: `measure_native_tooltip` reads each row FontString's font state,
  measures the stripped text with `WowFontSystem::measure_text_height`, stores
  the result on the row FontString and sums the visible rows.
- [WM63 L23 wrap and spacing](../../../../.local/state/wow-ui-sim/handoff/native-evidence-70688/tooltip/WM63-L23-wrap-spacing.md)
  section 3 is the native source (outside the repository).
- [Tooltip wrapped content width](tooltip-wrapped-content-width.md) selects the
  width that a wrapped row is measured at.
- [Tooltip layout timing](tooltip-layout-timing.md) covers when sizing runs and
  the row anchors.
- [Tooltip attachments](tooltip-attachments.md) covers the overflow and
  reservation a measured row carries.

## Implementation inventory

- `src/iced_app/tooltip.rs` — `measure_native_tooltip` (row measurement, frame
  height, FontString height publication) and `native_line_font` (per-row font
  state).
- `src/lua_api/frame/methods/widgets/tooltip/line_frames.rs` —
  `apply_body_font_to_synthesized_line`, which gives a row after the two declared
  font slots the tooltip's declared body font.
- `src/lua_api/frame/methods/widgets/tooltip/sizing.rs` — the estimator used only
  by builds without glyph measurement.

## Tests asserting this spec

- `tests/tooltip_measured_row_heights.rs` — an item tooltip's body rows measure
  one body font and the tooltip height is the sum of the measured rows; a wrapped
  description row is several text lines tall.
- `tests/tooltip_geometry_lane.rs` — the tooltip height follows a row's
  `SetFont` size.
- `tests/tooltip_texture_rows.rs` — a row height adds its attachment overflow.

## Known gaps (current cycle)

- [ ] Native font files, glyph advances, baselines and exact rendered line
      heights are UNKNOWN. The arithmetic is exact only relative to the
      simulator's own measured text.
- [ ] Builds without glyph measurement use the estimated glyph height in
      `sizing.rs`, not FontString measurement.
- [ ] WM63 L23 is Retail-only evidence; Wrath, Mists, Era, Anniversary and
      WowForever keep their legacy sizing path.

## Out of scope

- Wrap width selection (see [tooltip-wrapped-content-width](tooltip-wrapped-content-width.md)).
- Texture and atlas row attachment reservations (see [tooltip-attachments](tooltip-attachments.md)).
- Owner anchors, cursor placement and owner clearing.
