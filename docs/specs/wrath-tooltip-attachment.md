# Wrath tooltip attachment and sizing

`GameTooltip:AddTexture`, `Show` and `SetMinimumWidth` on Wrath 3.3.5a (12340) use the
template slots `<tooltipName>Texture1` .. `Texture10` and recalculate the tooltip size
when the tooltip is shown. Native evidence is the reconstructed 12340 source in
`handoff/laptop-labels-answers-20261018/tooltip-addtexture-12340.md` and
`batch8-tooltip-precision.md`. The original consumers are pinned
`FrameXML/GameTooltip.lua`, `FrameXML/LFRFrame.lua:619-662`,
`FrameXML/KnowledgeBaseFrame.lua:516-585` and `FrameXML/PaperDollFrame.lua:1056-1084`.
Retail/PTR keep the separate contract in [tooltip-attachments](tooltip-attachments.md);
Mists, Era, Anniversary and Wowforever keep their separate texture rows.

## What it must do

### Argument and row selection

- [x] Read the first argument with `lua_tolstring` semantics: a string or a number is a
  FILENAME, and a number is never resolved as a FileDataID. A number formats as its
  decimal filename (`136243` becomes `"136243"`).
- [x] A value that is not string-compatible leaves the filename null, consumes a slot and
  raises no type error.
- [x] Reject the call when the tooltip has fewer than two rows. Zero rows and one row both
  attach nothing and record no slot.
- [x] Attach to the LAST existing row and never add a row.
- [x] Use at most 10 slots; a call with all 10 used attaches nothing.
- [x] Record the filled slot in attachment order for `ClearLines` and the adjacency rule.

### Placement

- [x] Anchor the slot TOPRIGHT to the current row's left text TOPLEFT with x = -G, where
  G is the profile gap (`wrath_texture_gap`).
- [x] Anchor the current left text below the previous left text with x = 0 when the most
  recent slot is on the immediately preceding row, and x = slot width + G otherwise.
- [x] Pre-anchor the next existing left text below the current left text with
  x = -(slot width + G).

### Sizing

- [x] `Show` on a tooltip with an owner and at least one row runs `CalculateSize`
  synchronously, even when the tooltip is already shown.
- [x] A content setter that shows the tooltip (`SetText`, `SetSpell`, `SetUnit`,
  `SetInventoryItem`, and the other `Set*` population methods) sizes it synchronously,
  because the original consumer never calls `Show` after them.
- [x] `SetMinimumWidth` sets the CONTENT width floor and does not recalculate immediately.
  A wrapped row starts from that floor.

### Font metrics

- [ ] Native glyph metrics are unknown. Wrapped-row width uses the ordered left-string
  refinement: the candidate is the measured row width capped at 230.39999, the shared
  width grows only when it is below the candidate, and the last measured range runs to the
  end of the string. The resulting numbers are simulator measurements, not native values.

## How it works

- [Widget system](../widget-system.md), [rendering pipeline](../rendering-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/tooltip/wrath.rs` reads the filename, selects the
  slot and anchors it.
- `src/lua_api/frame/methods/widgets/tooltip/sizing.rs` exposes
  `size_wrath_tooltip_on_show`.
- `src/iced_app/tooltip/wrath.rs` measures Wrath tooltips.
- `src/lua_api/frame/methods/core_state/visibility.rs` sizes on `Show`.
- `src/lua_api/frame/methods/widgets/tooltip/content.rs` sizes when a setter shows.

## Tests asserting this spec

- `tests/tooltip_text.rs` covers the one-row rejection, the numeric-filename rule and the
  wrapped-row width.
- `tests/tooltip_word_wrap_min_width.rs` covers the `SetMinimumWidth` floor.
- `tests/tooltip_hover_render.rs` covers the synchronous size after a character-slot hover.
- `tests/widgets_b6_u1.rs` covers slot size, the ten-slot cap and the blank filename.
- `tests/common/wrath_spell_panels_f10.rs` covers the show-on-set path through `SetSpell`.

## Known gaps (current cycle)

- [ ] `Show` on a tooltip without an owner or without rows still shows it instead of
  hiding it, contrary to the 12340 `ShowThis` precondition.
- [ ] The `Usage:` error for a partial texcoord tuple (the first value numeric and a later
  one not) is not raised.
- [ ] A row that is not refined keeps a zero left-line width, so its wrap width is not the
  shared width. Height only, not width, is affected.
- [ ] `ClearLines` resets the minimum width even when the minimum was forced.

## Out of scope

- Backdrop definition, `BuffButtonTemplate` hover and `BuffFrame_OnUpdate` cadence.
