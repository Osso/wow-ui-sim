# Tooltip owner anchors

`GameTooltip:SetOwner(owner, anchor, xOffset, yOffset)` placement contract for
Retail/PTR, from the Retail 12.1.5 / 70688 native decode WM63 O17 §1. The
lifecycle half (clear, hide, reassignment) is a
[separate spec](tooltip-owner-lifecycle.md). Screen clamping is not modelled:
the native final rendered rectangle after clamping is UNKNOWN, so no anchor
equation here is a rendering claim.

## What it must do

### Anchor mode table

With a non-null owner, the tooltip point pins to the owner point. Displacements
are `xOffset` and `yOffset`.

- [x] `ANCHOR_LEFT` — tooltip `BOTTOMRIGHT` to owner `TOPLEFT`.
- [x] `ANCHOR_RIGHT` — tooltip `BOTTOMLEFT` to owner `TOPRIGHT`.
- [x] `ANCHOR_BOTTOMLEFT` — tooltip `TOPRIGHT` to owner `BOTTOMLEFT`.
- [x] `ANCHOR_BOTTOM` — tooltip `TOP` to owner `BOTTOM`.
- [x] `ANCHOR_BOTTOMRIGHT` — tooltip `TOPLEFT` to owner `BOTTOMRIGHT`.
- [x] `ANCHOR_TOPLEFT` — tooltip `BOTTOMLEFT` to owner `TOPLEFT`.
- [x] `ANCHOR_TOP` — tooltip `BOTTOM` to owner `TOP`.
- [x] `ANCHOR_TOPRIGHT` — tooltip `BOTTOMRIGHT` to owner `TOPRIGHT`.

An original consumer path asserts the same pair: `TotemButtonMixin:OnEnter`
passes `ANCHOR_BOTTOMRIGHT`.

### Displacements

- [x] Both default displacements are `0`; there is no added 20-unit
  displacement in any anchor mode.
- [x] Explicit zero displacements are stored as `0`, not as a sentinel for
  another displacement.

### Name parsing

- [x] An absent, empty or unrecognized anchor name selects `ANCHOR_LEFT`, and
  the request reports no invalid-anchor error.
- [x] The stored anchor type for such a request is `ANCHOR_LEFT`.

### `ANCHOR_NONE` and `ANCHOR_PRESERVE`

- [x] `ANCHOR_NONE` with a non-null owner installs no owner anchor; a
  re-anchor clears the existing points first.
- [x] `ANCHOR_PRESERVE` returns before clearing or installing points, so the
  existing points and stored displacements are unchanged.

### Cursor modes

Cursor modes anchor relative to the coordinate root's `BOTTOMLEFT`, not to the
owner.

- [x] `ANCHOR_CURSOR` uses tooltip `BOTTOM` and ignores the stored
  displacements.
- [x] `ANCHOR_CURSOR_LEFT` uses tooltip `BOTTOMRIGHT` and applies the stored
  displacements.
- [x] `ANCHOR_CURSOR_RIGHT` uses tooltip `BOTTOMLEFT` and applies the stored
  displacements.
- [x] The update path recomputes cursor placement for all three modes while the
  tooltip is visible.
- [x] With no known cursor position, a cursor mode installs no anchor.

### Profile scope

- [x] Forever keeps its legacy anchor practice and is out of this contract.
  Forever and the other non-Retail profiles are not asserted here.

## How it works

- [Tooltip owner lifecycle investigation](../wiki/investigations/tooltip-owner-lifecycle.md)
- [Tooltip layout timing](tooltip-layout-timing.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/tooltip/owner.rs` — `SetOwner` /
  `SetAnchorType` argument parsing, mode defaulting, owner point pairs, cursor
  dispatch and `ANCHOR_NONE`/`ANCHOR_PRESERVE` handling.
- `src/lua_api/tooltip.rs` — `build_cursor_anchor`, the cursor-mode anchor
  relative to the coordinate root.
- `src/lua_api/state/cursor_tooltips.rs` — recomputes cursor anchors for
  visible cursor-mode tooltips when the mouse moves.

## Tests asserting this spec

- `tests/tooltip_owner_anchors.rs` — eight-mode point-pair table, zero
  displacements, absent/empty/unknown names, and the `TotemButtonMixin:OnEnter`
  original consumer.
- `tests/tooltip_set_owner.rs` — `ANCHOR_NONE`, `ANCHOR_PRESERVE`, `RIGHT`
  offsets, the absent-name default count, and cursor following with a known
  mouse position.
- `tests/tooltip_lifecycle_owner_lane.rs` — all three cursor modes with
  offsets, cursor movement, and the `GameTooltip_ShowSimpleTooltip`
  `ANCHOR_BOTTOMRIGHT` consumer.

## Known gaps (current cycle)

None for Retail/PTR anchor placement.

## Out of scope

- Screen clamping and final rendered-rectangle placement: WM63 O17 leaves the
  clamping movement UNKNOWN, so the simulator does not model it and no test
  asserts a clamped rectangle.
- Raw input-to-screen conversion, hidden-tooltip scheduling, owner-auto-hide,
  and callback re-entry.
