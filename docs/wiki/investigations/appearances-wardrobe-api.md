# Appearances Wardrobe API

Collections Journal > Appearances opens and renders a `WardrobeCollectionFrame`, but most meaningful browsing behavior is still backed by fixed/default transmog API stubs instead of simulator state.

## Content

Baseline:

- `target/debug/wow-sim` builds in this worktree when `LD_LIBRARY_PATH=target/debug` is used for the local `libiced_dynamic.so`.
- `wow-sim --no-addons --no-saved-vars lua-errors` returns `[]`.
- Opening the Wardrobe tab with `ToggleCollectionsJournal(5)` plus `CollectionsJournal_SetTab(CollectionsJournal, 5)` produces a visible `WardrobeCollectionFrame` subtree.

Current API shape:

- `C_TransmogCollection.GetCategoryAppearances` returns seeded rows by category and active collection/source/search filters.
- Default appearance seeds include two Shirt rows and one Tabard row in addition to the five-row armor/weapon categories, so the Wardrobe shirt/tabard slots are no longer empty by default.
- Source/collection/search filter setters now mutate `WorldState`, and category rows/counts apply collected/uncollected, source-type, and search-text filters.
- `C_CreatureInfo.GetClassInfo()` must return localized `className` values (`Paladin`) plus uppercase `classFile` tokens (`PALADIN`). The Wardrobe class dropdown uses `className` for row text and `classFile` for class-color lookup.
- `C_TransmogSets.GetBaseSets()` must return a table, not nil. `WardrobeSetsDataProviderMixin:GetBaseSets()` assigns the result before calling `DetermineFavorites()`; nil leaves `self.baseSets` unset and recurses until stack overflow. Retail and Forever now project supplied base records; profiles outside that scope keep the empty default.
- Search completion is synchronous and deterministic: no DB loading, no in-progress state, and progress equals result size.
- Appearance rows include displayability/usability fields (`canDisplayOnPlayer`, `isUsable`, `isValidSourceForPlayer`, `isHideVisual`, `isFavorite`, name, and quality) so Blizzard's Wardrobe model buttons do not render every card as an invalid red slashed placeholder.
- Wardrobe filter dropdowns rely on normal mouse dispatch to the dropdown button. Decorative child regions must remain visual-only hit-test descent candidates, not final click targets; otherwise the filter button's `OnMouseDown` never opens its Blizzard menu even though the menu description and filter callbacks are valid.
- `C_TransmogOutfitInfo.GetAllSlotLocationInfo` reports weapon appearance slots with `Enum.TransmogCollectionType.None`; Blizzard derives weapon browsing categories through `IsEitherHand()` and `GetCategoryInfo()`. Reporting weapon collection categories there makes `TransmogLocationMixin:GetArmorCategoryID()` non-nil for weapons, and Wardrobe's armor-only model setup path then indexes missing `MAINHANDSLOT` / `SECONDARYHANDSLOT` entries in `WARDROBE_MODEL_SETUP`.
- Tooltip and visual-state helpers such as `GetAppearanceSourceInfo`, `GetAppearanceInfoBySource`, `GetIsAppearanceFavorite`, `SetIsAppearanceFavorite`, `IsNewAppearance`, and `ClearNewAppearance` are not backed by first-class visual/source state.
- Retail and Forever publish catalog-backed `GetBaseSets`, `GetUsableSets`, `GetSetPrimaryAppearances`, and `GetBaseSetID`; the loaded `WardrobeSetsDataProviderMixin` reads those relations for its sets tab. **INFERRED (R88cs pending):** usability, variant collection aggregation, unknown-ID results, slot-source projection, and complete appearance-item results follow the shared consumer and supplied catalog model. Other profiles retain the byte-identical Lua defaults.

Implementation direction:

- Treat the Wardrobe panel as a C API/state-model problem, not a Blizzard Lua patching problem.
- Add stateful transmog source, visual, filter, search, and favorite state before trying to polish UI output.
- The item cards are `DressUpModel`/model-scene based. The simulator can clear invalid overlays and drive selection/filter state, but real 3D item previews remain under the documented no-3D rendering gap.
- Keep in-world `Blizzard_Transmog` compatible through the shared `C_TransmogCollection` surface, but defer transmogrifier transaction/apply-cost behavior until browsing/filtering works.
- Keep 3D model preview gaps isolated to existing model stubs.

## Sources

- [PLAN.mog.md](../../../PLAN.mog.md) — baseline commands, target scope, and audit checklist
- [Blizzard_Wardrobe.lua](../../../vendor/wow-ui-source/Interface/AddOns/Blizzard_Collections/Mainline/Blizzard_Wardrobe.lua) — Collections Journal Wardrobe call sites
- [Blizzard_Transmog.lua](../../../vendor/wow-ui-source/Interface/AddOns/Blizzard_Transmog/Blizzard_Transmog.lua) — in-world transmog call sites
- [Blizzard_TransmogShared.lua](../../../vendor/wow-ui-source/Interface/AddOns/Blizzard_TransmogShared/Blizzard_TransmogShared.lua) — shared tooltip/favorite/source helpers
- [transmog_collection.rs](../../../src/lua_api/globals/missing_surface/transmog_collection.rs) — current Rust `C_TransmogCollection` surface
- [transmog.rs](../../../src/lua_api/globals/missing_surface/transmog.rs) — current Rust `C_Transmog` surface
- [transmog-set-relations](../../specs/transmog-set-relations.md) — Retail/Forever relation contract and pending native evidence.
- [transmog_sets_defaults.rs](../../../src/lua_api/workarounds/temporary/transmog_sets_defaults.rs) — unchanged empty/default `C_TransmogSets` surface for profiles outside the modeled scope
- `~/.cache/wow-ui-sim/blizzard-ui/{retail,wowforever}/AddOns/Blizzard_TransmogShared/Blizzard_TransmogShared.lua` — runtime-selected provider call sites at 970, 994, 1068 and 1094; `Blizzard_Transmog.toc` allows Game loading and depends on `Blizzard_TransmogShared`.

## See Also

- [[lua-api]] — C API namespace registration and Lua surface conventions
- [[frame-data-flow]] — Lua/Rust state synchronization model
- [[addon-loading]] — Blizzard addon loading and startup sequence
- [[transmog-inventory-slot-scope]] — retail `Blizzard_TransmogShared` loader-only inventory-slot compatibility
