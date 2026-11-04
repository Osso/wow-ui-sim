# Retail tooltip path separator (U1)

**Superseded (2026-11-04):** native evidence WM63 S8L and WM84 replaces the U1 geometry policy below. The divider now follows the native All anchor in [tooltip attachments](tooltip-attachments.md): it spans the blank row's left text and reserves no space for these margins. Numeric `AddTexture` attaches to the last row instead of appending one. The record below is kept as history.

Integration checkpoint (2026-10-16): corrected user-selected geometry oracle reached Retail RED → GREEN; [bounded proof](../baselines/approved-batch-2026-10-16.md). No native-parity claim.

Bounded simulator contract for the original cached Retail `SocialUIUtil.AddSeparatorToTooltip` consumer. Source: `src/lua_api/frame/methods/widgets/tooltip/line_frames.rs` and `src/iced_app/tooltip.rs`. Implemented simulator contract; numeric behavior is preserved. Geometry below is an explicit simulator policy, not established native semantics.

## What it must do

- [ ] Execute the unchanged cached `retail/AddOns/Blizzard_SocialUIShared/SocialUIUtil.lua:138-152` and attach its path/options divider to the wrapped blank row it just added, without increasing NumLines again.
- [ ] Emit one texture request for that path whose bounds equal the blank row's text rect under the render scale, with tooltip alpha.
- [ ] ClearLines removes the attachment; rebuilding text-only content emits no stale divider.
- [ ] Numeric and numeric-string AddTexture inputs attach to the last row without appending rows.

## How it works

- [Rendering pipeline](../rendering-pipeline.md)
- [Widget system](../widget-system.md)

## Implementation inventory

- `src/lua_api/tooltip.rs`: attached path and height/margin data; one geometry function governs both row reservation and image offset.
- `src/lua_api/frame/methods/widgets/tooltip/line_frames.rs`: bounded path/options decoding and existing-row attachment.
- `src/lua_api/frame/methods/widgets/tooltip/sizing.rs`: immediate Lua-visible row sizing.
- `src/iced_app/tooltip.rs`: measured sizing, render collection and deferred path quad emission.

## Tests asserting this spec

- `tests/tooltip_separator_u1.rs`: original complete cached Lua file, helper invocation, row/text order, emitted quad bounds at scales 1 and 0.5, alpha, clear/rebuild and numeric preservation. No image decode or CASC access required for deferred quad evidence.
- `tests/tooltip_text.rs`: existing numeric and numeric-string behavior controls.

## Known gaps (current cycle)

- [x] User selected unchanged blank-row height and centered image on 2026-10-16; simulator policy only.
- [x] Corrected tests-only oracle reached missing-divider RED before producer; divider/numeric cases became GREEN, with existing tooltip preservation and startup observation.
- [ ] Texture decoding/GPU pixels, native and other-profile acceptance remain outside this evidence.

## Out of scope

Other texture anchor modes, path-only/legacy positional calls, arbitrary nonblank-row attachments, numeric rendering changes, atlas changes, native parity, texture decode/assets/GPU acceptance, other profiles and vendor Lua modifications.
