# Transmog outfits

Retail outfit editing stores catalog metadata, saved contents and a pending overlay in simulator state. The [publication closure contract](../../specs/patch-12-0-0-publication-sweep.md#transmog-outfit-closure-contract) owns required behavior and proof boundaries.

## State and API

`SimState.transmog_outfit_catalog.entries` holds `OutfitEntry` values. `SimState.transmog_outfits` holds saved slot/situation/secondary data keyed by outfit ID, pending slot and situation maps, viewed weapon options, host situation metadata, quotas and prices. Active and viewed IDs are separate optional integers; numeric API queries represent absence as zero.

Creation/rename and displayed selection emit synchronous documented events. Pending appearances overlay the viewed saved slots. Apply validates eligibility and funds before draining pending slots, debits money, saves slot contents and selects the applied outfit. Failed apply leaves pending state and money intact. Situations have their own pending commit/reset operations; no automatic world-trigger evaluation or persistence is claimed.

Rich appearance-source inputs and the existing compact world source catalog share one lookup. Set catalog entries and slot-source membership drive available-set filters and imports. Custom-set arrays use inventory-slot ordering, not outfit enum ordering. Pickup places outfit ID on the cursor; existing action-slot state receives placed outfits, including equipped-gear ID zero.

## Consumer boundary

Slot locations use `Enum.TransmogOutfitSlot`, which differs from zero-based inventory slots. Both shoulder locations exist for TransmogUtil caching; displayed slot groups hide the left shoulder until split is enabled. Equipped option queries read actual item inventory type. `C_Transmog.GetSlotVisualInfo` returns one structured value with numeric source/visual identities, not the retired tuple API. No Blizzard Lua or namespace retirement rules are patched.

Native prices/caps, artifact/spec restrictions, binding/race/form eligibility, automatic situation metadata, and native hyperlink interoperability are not verified. Simulator choices are marked `INFERRED` at their implementation sites.

## Sources

- [Outfit implementation](../../../src/c_api/c_transmog_outfit_info.rs)
- [Behavioral tests](../../../tests/transmog_outfit_lifecycle.rs)
- [Publication contract](../../specs/patch-12-0-0-publication-sweep.md)

## See Also

- [[transmog-inventory-slot-scope]] — previous cache/source mismatch, not a reason to monkey-patch vendor Lua
- [[lua-api]] — runtime namespace registration
