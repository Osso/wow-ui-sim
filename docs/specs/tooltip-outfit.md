# C_TooltipInfo.GetOutfit

`C_TooltipInfo.GetOutfit(outfitID)` returns TooltipData for an outfit ID. Its shared tooltip registration lives in `src/lua_api/globals/missing_surface/tooltip_info/mod.rs`; its producer is in `src/c_api/c_tooltip_info_outfit.rs`. See the [tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation) for shared construction.

## What it must do

### Forever publication and inputs

- [x] **VERIFIED (Forever 1.60.1.70707):** `TooltipInfoDocumentation.lua:646-659` declares `GetOutfit` with one required number argument, `MayReturnNothing`, and a non-nilable TooltipData return.
- [x] **VERIFIED (R88n.8):** accept finite numbers and convertible numeric strings in 0..=4294967295, validate before truncating toward zero, and use `Usage: local data = C_TooltipInfo.GetOutfit(outfitID)` for argument errors.
- [x] **VERIFIED (R88n.8):** outfit ID zero returns zero Lua values.
- [x] **VERIFIED (R88n.8):** a nonzero ID without a saved-name record returns one TooltipData table built with an empty name.
- [x] **INFERRED:** a matching row in the shared `transmog_outfit_catalog` supplies the saved name used by this getter. The original Retail `GameTooltip:SetOutfit` consumer renders two distinct catalog-created names.
- [x] The optional Forever contract test checks registration, the zero-ID path, a nonzero missing-name result, and the original SpellBook outfit consumer. The default test also checks two distinct nonzero missing-name inputs and ID conversion.

### Retail compatibility

- [x] Preserve the existing Retail catalog lookup, positive-integer parser, shared title builder, and zero-value miss.

## How it works

- [Tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation)
- [Forever Camelot outfit inputs](forever-camelot-spellbook.md)

## Implementation inventory

- `src/c_api/c_tooltip_info_outfit.rs` — profile-specific ID conversion, name lookup and TooltipData publication.
- `src/c_api/c_tooltip_info_outfit.rs` — reads Forever names from the shared `transmog_outfit_catalog`.
- `src/c_api/mod.rs` — compiles the producer for Retail-family clients and Forever.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs` — registers the getter on Retail/PTR and Forever.
- `src/c_api/tooltip_getter_args.rs` — Forever unsigned numeric conversion.

## Tests asserting this spec

- `tests/tooltip_outfit_forever.rs` — default Forever API test for zero, range validation, numeric-string truncation, and two distinct missing-name IDs.
- `tests/tooltip_forever_four_getters.rs` — optional missing-name API case and original `SpellBookOutfitItemMixin:OnIconEnter` dispatch on the selected Forever PlayerSpells addon.
- `tests/tooltip_residual_prefork.rs::tooltip_residual_outfit_catalog_name_reaches_original_consumer` — two populated Retail catalog names through the original `GameTooltip:SetOutfit` accessor.
- `tests/p1200_rest_outfit_tooltip.rs` — existing Retail catalog and miss behavior.

## Known gaps (current cycle)

- [ ] **INFERRED:** the native accessor's saved-name record is the current player's outfit state. Writer, persistence and update-event parity remain unknown; the simulator test proves only its catalog-to-consumer mapping.
- [ ] **UNKNOWN:** native TooltipData line layout and other fields beyond the verified empty-name table path.

## Out of scope

- Outfit creation, mutation, persistence, icon selection, slot summaries, and native writer/event behavior.
