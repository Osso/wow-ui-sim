# C_TooltipInfo.GetOutfit

`C_TooltipInfo.GetOutfit(outfitID)` returns TooltipData for an outfit ID. Its shared tooltip registration lives in `src/lua_api/globals/missing_surface/tooltip_info/mod.rs`; its producer is in `src/c_api/c_tooltip_info_outfit.rs`. See the [tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation) for shared construction.

## What it must do

### Forever publication and inputs

- [ ] **VERIFIED (Forever 1.60.1.70707):** `TooltipInfoDocumentation.lua:646-659` declares `GetOutfit` with one required number argument, `MayReturnNothing`, and a non-nilable TooltipData return.
- [ ] **VERIFIED (R88n.8):** accept finite numbers and convertible numeric strings in 0..=4294967295, validate before truncating toward zero, and use `Usage: local data = C_TooltipInfo.GetOutfit(outfitID)` for argument errors.
- [ ] **VERIFIED (R88n.8):** outfit ID zero returns zero Lua values.
- [ ] **VERIFIED (R88n.8):** a nonzero ID without a saved-name record returns one TooltipData table built with an empty name.
- [ ] **INFERRED:** a matching row in the simulator's `forever_outfits` snapshot supplies the saved name used by this getter.
- [ ] The optional Forever contract test checks registration, the zero-ID path, a nonzero missing-name result, and the original SpellBook outfit consumer.

### Retail compatibility

- [x] Preserve the existing Retail catalog lookup, positive-integer parser, shared title builder, and zero-value miss.

## How it works

- [Tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation)
- [Forever Camelot outfit inputs](forever-camelot-spellbook.md)

## Implementation inventory

- `src/c_api/c_tooltip_info_outfit.rs` — profile-specific ID conversion, name lookup and TooltipData publication.
- `src/c_api/forever_outfits.rs` and `src/lua_api/state/sim_state.rs` — existing supplied Forever outfit snapshot reused by the inferred name lookup.
- `src/c_api/mod.rs` — compiles the producer for Retail-family clients and Forever.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs` — registers the getter on Retail/PTR and Forever.
- `src/c_api/tooltip_getter_args.rs` — Forever unsigned numeric conversion.

## Tests asserting this spec

- `tests/tooltip_forever_four_getters.rs` — optional missing-name API case and original `SpellBookOutfitItemMixin:OnIconEnter` dispatch on the selected Forever PlayerSpells addon.
- `tests/p1200_rest_outfit_tooltip.rs` — existing Retail catalog and miss behavior.

## Known gaps (current cycle)

- [ ] **INFERRED:** the native accessor's saved-name record is the current player's outfit state, and `SimState.forever_outfits` is the matching simulator input. The populated-name path has no test because the lane asserts only verified native facts; writer, persistence and update-event parity remain unknown.
- [ ] **UNKNOWN:** native TooltipData line layout and other fields beyond the verified empty-name table path.

## Out of scope

- Outfit creation, mutation, persistence, icon selection, slot summaries, and native writer/event behavior.
