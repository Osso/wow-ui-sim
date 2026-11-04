# Tooltip texture and atlas attachments (Retail/PTR)

`GameTooltip:AddTexture` and `AddAtlas` on Retail and PTR attach images to the last existing text row and reserve layout space for them. The contract follows native evidence WM63 S8L (anchors, settings, cap) and WM84 (reservation groups, overflow, row inset) for Retail 12.1.5 / 70688. Source: `src/lua_api/frame/methods/widgets/tooltip/attachments.rs`, with layout in `src/iced_app/tooltip.rs` and `src/lua_api/frame/methods/widgets/tooltip/sizing.rs`. Wrath template slots and the separate texture rows of the other profiles are unchanged. This supersedes the U1 policy in [tooltip-path-separator](tooltip-path-separator.md).

## What it must do

### Attachment

- [x] Attach a file ID, path or atlas to the last existing row. Do not create a row; an empty tooltip accepts nothing.
- [x] Accept at most 30 attachments for the whole tooltip. ClearLines and SetText release them.
- [ ] SetOwner and Hide also release attachments (implemented; no direct test).
- [x] Anchor each attachment independently to the selected left or right text of that row. Equal settings overlay; there is no horizontal or vertical sequence.
- [x] Apply the anchor ordinals: LeftTop moves left by the right margin; RightCenter moves right by the left margin; All spans the selected text with two points and no horizontal offset. `verticalOffset` raises the attachment.
- [ ] LeftCenter, LeftBottom, RightTop and RightBottom placement (implemented from the same table; reservation only is tested for LeftBottom and RightTop).
- [x] Read settings with defaults width 12, height 12, offset 0, margins left 0, right 8, top 0, bottom 0. Margins stay fractional and negative without clamping.
- [ ] Anchor and region truncate toward zero; out-of-range values keep 0. Tint and desaturation clamp to 0 through 1. A zero dimension uses the resolved texture or atlas extent (implemented; no direct test).

### Reservation

- [x] Sum four horizontal groups: outward left, inward from left, inward from right, outward right. Outward groups take the first candidate, then the maximum. Inward groups start at zero and share one first-eligible selection, so a negative first inward candidate is kept.
- [x] Width selection and wrapped width use the row inset plus that sum. The left text anchor adds the inset and the outward-left group. The right text anchor subtracts the inset and both outward groups.
- [x] Top and bottom overflow are the per-row maxima of the WM84 candidates, each at least zero. Row height adds both to the measured left text. A row's TOP anchor subtracts its top overflow and the preceding indexed row's bottom overflow.

### Row inset

- [x] Store each row's inset as its explicit inset plus the tooltip-wide extra inset: `AddLine` argument 7, `AddDoubleLine` argument 11, `SetText` zero. Attachments do not change it.
- [x] ClearLines sets the extra inset to zero; a later row has no inset from an earlier row.

## How it works

- [Rendering pipeline](../rendering-pipeline.md): attachments are pooled child Texture regions drawn by the generic texture path.
- [Tooltip layout timing](tooltip-layout-timing.md)

## Implementation inventory

- `src/lua_api/tooltip.rs`: attachment record, width groups, overflow, row inset and the extra-inset seam.
- `src/lua_api/frame/methods/widgets/tooltip/attachments.rs`: settings parsing, texture pool, anchors and release.
- `src/lua_api/frame/methods/widgets/tooltip/line_data.rs`: explicit inset arguments; ClearLines reset.
- `src/lua_api/frame/methods/widgets/tooltip/sizing.rs`: row anchors and estimated sizing without glyph measurement.
- `src/iced_app/tooltip.rs`: measured width, wrap width, overflow and row heights; assigned wrap width for drawing.
- `src/lua_api/frame/methods/text_attribute_event/text.rs`, `owner.rs`, `content.rs`: release attachments when rows are replaced.

## Tests asserting this spec

- `tests/tooltip_attachments_lane.rs`: original `TooltipDataRules` post-callbacks (horizontal groups, anchors, overflow), line offsets forwarded by `SharedTooltipTemplates`, the `ItemRefTooltip` close-button spacer, overflow, fractional margins, All, shared inward selection and the cap.
- `tests/tooltip_separator_u1.rs`: original `SocialUIUtil.AddSeparatorToTooltip` All divider and its rendered quad.
- `tests/tooltip_text.rs`, `tests/tooltip_texture_secrecy.rs`: no appended rows on Retail/PTR.

## Known gaps (current cycle)

- [x] RED (7/7 failing) and GREEN (7/7 passing) on Retail; GREEN 7/7 on PTR.

## Out of scope

- UNKNOWN natively, kept as named seams: any writer of a nonzero tooltip-wide extra inset; draw order between attachments and text (simulator default texture layer); renderer clipping (none applied); placement against a missing right text object (not drawn); the integer-error path for anchor or region values outside the signed 32-bit range; atlas and explicit texture-coordinate composition (texture `SetTexCoord` semantics); CopyTooltip attachment behavior (copies carry none).
- Money and status-bar rows, non-finite geometry and live-client verification.
