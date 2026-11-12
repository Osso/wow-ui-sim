# Forever Camelot spellbook observations

The [contract](../../specs/forever-camelot-spellbook.md) supplies only the two outfit reads used by the selected Forever Camelot spellbook. Availability reuses [the shared observation](transmog-availability.md); outfit-list observations are independent. Neither input infers native defaults.

## Model

`SimState.forever_outfits` is `Option<Option<Vec<OutfitEntry>>>`: outer None means no supplied observation; Some(None) is an explicit zero-result observation; Some(Some(entries)) yields one fresh array. The common OutfitEntry keeps the seven non-nil fields from pinned Forever 70532 `TransmogOutfitInfoDocumentation.lua:896–907`. It also serves the existing Retail/PTR catalog. Snapshot conversion copies rows and nested situationCategories; the modern catalog retains its existing behavior.

Only Forever publishes the supplied GetOutfitsInfo handler. The modern epoch-gated module, other outfit queries, mutations and catalog defaults remain unchanged.

## Selected consumer and ordering

Forever `Blizzard_PlayerSpells.toc` selects `[Game]` as Camelot and the shared SpellBook category. The selected Camelot CreateCategoryMixins:115 instantiates the transmog category at 67; its Init:422 calls original UpdateSpellGroups:483, which reads GetOutfitsInfo before later IsAvailable:506 reads availability at 507 and outfits at 508. Therefore constructor failure alone cannot prove availability publication. The independent original query in each availability test runs before explicit construction, then GREEN tests use the constructor-created category and original UpdateAllSpellData:487.

The original PopulateSpellGroupsIndiciesByRange:463 reads GetOutfitsInfo without consulting availability. It rebuilds membership and ordered indices as 100000+outfitID, including the clear-outfit index 100000. Sparse replacement fixtures distinguish a live supplied list from empty/fixed/no-op producers.

## Proof status

All three exact final producer-free RED selectors fail at original availability:507 (two cases) or constructor catalog:484 (one case). Byte-identical GREEN tests pass through the original selected constructor, original IsAvailable and original UpdateAllSpellData tab shown state. The independent sparse selector executes original PopulateSpellGroupsIndiciesByRange with 23/81→47→empty/no-result→81/23, exact stale-membership removal, independent environments and fresh nested snapshots. One-vs-zero result arity and both named unset errors/recovery are verified. These transition/replacement assertions reject constants and no-ops; no separately compiled constant-mutant run is claimed.

The actual frames load in existing MKB state before the host supplies Gamepad style. Preloading Gamepad requires missing C_GamePad.GetActiveDeviceID and aborts input-icon setup. No device-startup fix or queued input-transition dispatch is claimed. All six GREEN environments retain six unfiltered diagnostics: Communities GetPreferredDatacenterLocalities:154, TargetFrame AuraContainerUtil:242 and Camelot talents GetCombatConfigIDForSpecGroup:195, each with its reporter. Successful Book construction/refresh is not clean bootstrap, complete UI browsing or native entitlement proof. Original GetElementDataForItem:453–460 calls GetOutfitInfo:457; this lane does not publish that query or prove selecting/painting actual outfit items. Tab shown-state assertions do not establish rendered pixels.

Paired raw stock remains 26 keys/26 occurrences, with no additions, removals or count changes. All 34 process guards preserve the current original CVar bytes and exact stat metadata through fresh child-only copies. All seven full-feature all-target checks pass with zero Cargo JSON/plain compiler warnings. PTR/Retail cover e58baccde; the other five cover 2c72486e7, whose only later change repairs a client-mists-only test input field. Production code and the Forever tests remain unchanged; this does not establish other-profile runtime parity.

## Sources

- [Contract](../../specs/forever-camelot-spellbook.md)
- [Original-consumer tests](../../../tests/forever_camelot_spellbook.rs)
- Pinned Forever 1.60.1.70532 generated documentation, selected TOC, shared category/frame and Camelot frame override.

## See Also

- [Shared availability](transmog-availability.md)
- [Consumer selection and RED proof](../investigations/consumer-selection-and-red-proof.md)
