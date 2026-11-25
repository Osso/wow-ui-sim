# Forever Camelot spellbook observations

The [contract](../../specs/forever-camelot-spellbook.md) covers the two outfit reads used by the selected Forever Camelot spellbook. Availability reuses [the shared observation](transmog-availability.md); the outfit catalog is shared with outfit creation.

## Model

`SimState.transmog_outfit_catalog.entries` is the single outfit-record store used by the Retail writer and Forever outfit queries. The common OutfitEntry keeps the seven non-nil fields from pinned Forever 70532 `TransmogOutfitInfoDocumentation.lua:896–907`. Snapshot conversion copies rows and nested situationCategories. **INFERRED (R88ex; native evidence pending):** an empty catalog returns zero values from `GetOutfitsInfo`, matching its `MayReturnNothing` declaration and the selected Lua `or {}` consumers; a populated catalog returns one array.

Only Forever publishes the supplied GetOutfitsInfo and GetOutfitInfo handlers. A_Admin.SetUnlockedOutfitSlots supplies outfit capacity for tests; `C_TransmogOutfitInfo.AddNewOutfit` creates records through the user API. The other modern outfit queries remain unavailable on Forever.

## Selected consumer and ordering

Forever `Blizzard_PlayerSpells.toc` selects `[Game]` as Camelot and the shared SpellBook category. The selected Camelot CreateCategoryMixins:115 instantiates the transmog category at 67; its Init:422 calls original UpdateSpellGroups:483, which reads GetOutfitsInfo before later IsAvailable:506 reads availability at 507 and outfits at 508. Therefore constructor failure alone cannot prove availability publication. The independent original query in each availability test runs before explicit construction, then GREEN tests use the constructor-created category and original UpdateAllSpellData:487.

The original PopulateSpellGroupsIndiciesByRange:463 reads GetOutfitsInfo without consulting availability. It rebuilds membership and ordered indices as 100000+outfitID, including the clear-outfit index 100000. Sparse replacement fixtures distinguish a live supplied list from empty/fixed/no-op producers. The opening test calls ToggleClassTalentFrame with two records created through AddNewOutfit and checks the original projection.

## Proof status

Earlier exact producer-free RED selectors established the original availability and constructor boundaries. Current Forever proof includes four exact integration cases: enabled/disabled availability transitions, sparse outfit projection, and the real Player Spells toggle with an empty catalog followed by two records created through `C_TransmogOutfitInfo.AddNewOutfit`. **VERIFIED (simulator):** the empty query returns zero values; after supplying capacity and two distinct name/icon records, the original spellbook projects both outfit IDs and records no Lua errors from opening the panel. **INFERRED (R88ex; native evidence pending):** the zero-value empty result follows the pinned `MayReturnNothing` declaration and guarded consumers. The prior strict unset-list error is no longer part of the contract.

The actual frames load in existing MKB state before the host supplies Gamepad style. Preloading Gamepad requires missing C_GamePad.GetActiveDeviceID and aborts input-icon setup. No device-startup fix or queued input-transition dispatch is claimed. The toggle test clears startup diagnostics before opening the panel, then checks the recorded error list after both the empty and populated opens. Other addon startup diagnostics remain outside this proof. Successful Book construction/refresh is not clean bootstrap, complete UI browsing or native entitlement proof. Original GetElementDataForItem:453–460 calls GetOutfitInfo:457; this lane does not prove selecting/painting actual outfit items. Assertions on group membership do not establish rendered pixels.

Paired raw stock remains 26 keys/26 occurrences, with no additions, removals or count changes. All 34 process guards preserve the current original CVar bytes and exact stat metadata through fresh child-only copies. All seven full-feature all-target checks pass with zero Cargo JSON/plain compiler warnings. PTR/Retail cover e58baccde; the other five cover 2c72486e7, whose only later change repairs a client-mists-only test input field. Production code and the Forever tests remain unchanged; this does not establish other-profile runtime parity.

Retail impact check: the removed supplied-state field was Forever-only, and Retail's `c_transmog_outfit_info::catalog::get_outfits_info` implementation is unchanged. The current Retail `--lib --test integration --no-run` build passes with zero compiler warnings. The legacy direct Retail outfit-catalog test is gated by `retail-12-0-5`, which is not enabled in the current Retail profile; no current Retail runtime query result is claimed.

## Sources

- [Contract](../../specs/forever-camelot-spellbook.md)
- [Original-consumer tests](../../../tests/forever_camelot_spellbook.rs)
- Pinned Forever 1.60.1.70532 generated documentation, selected TOC, shared category/frame and Camelot frame override.

## See Also

- [Shared availability](transmog-availability.md)
- [Consumer selection and RED proof](../investigations/consumer-selection-and-red-proof.md)
