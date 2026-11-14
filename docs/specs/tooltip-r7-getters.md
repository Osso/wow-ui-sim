# Round 7 tooltip getters

The Round 7 `C_TooltipInfo` getters use native argument readers and tooltip builders. Their contract and proof limits are recorded in the [tooltip-data wiki](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation).

## What it must do

- [x] Publish all eleven getters on Retail 12.1.5 (70688) and Forever 1.60.1 (70707); publish only `GetAzeriteEssence` and `GetAzeriteEssenceSlot` on PTR 12.1.7 (70668), as established in Round 7.
- [x] Use each getter's exact usage string, signed/unsigned conversion, optional argument, byte-slot, GUID, table and secret-argument rules from Round 7.
- [x] On Retail 12.1.5 (70688), missing-source calls to `GetQuestLogSpecialItem` and `GetQuestPartyProgress` return exactly zero Lua values, not `nil` or a tooltip table (R88n.1).
- [x] For `GetRuneforgeResultItem`, resolve the supplied GUID to an owned modeled item instance, preserve its identity, apply the supplied displayed level, and use the shared item tooltip builder when no unmodeled power/modifiers are requested.
- [x] `C_Item.GetItemNumSockets` must reject a secret `itemInfo` argument from a tainted caller on Retail/PTR/Forever with the modeled native `AllowedWhenUntainted` error; an untainted caller may read the argument.
- [x] On Retail, `GetQuestLogSpecialItem` resolves the one-based index through host-supplied quest-ID request context, reads primary/candidate item IDs from a separate host-supplied cache record, and uses only a host-supplied selected inventory object whose ID matches that record. It uses the shared item builder and publishes the selected instance GUID.
- [x] On Retail, `GetQuestPartyProgress` publishes only an explicit host-supplied snapshot with type 24 and quest ID, the optional type-17 title row, supplied type-18 participant headings, and type-8 status/objective rows. The host supplies row text, colors, and participant-filter classification; no party/objective values are seeded.
- [x] `omitTitle` omits the supplied title row, and `ignoreActivePlayer` filters only participant groups marked by the host.
- [x] Tests exercise original selected consumers, the original `TooltipDataHandler` accessors, argument errors, exact zero-value misses, and backed runeforge instances.

## How it works

- [Tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation) — argument readers, registration, producers and missing state.
- [Item socket count](item-num-sockets.md) — count and secrecy boundary.

## Implementation inventory

- `src/c_api/c_tooltip_info_round7.rs` — Round 7 publication, arguments, missing-state decisions and owned-item runeforge path.
- `src/c_api/quest_party_progress.rs` — host-supplied party-progress snapshot input.
- `src/c_api/quest_special_item.rs` — separate host inputs for index-to-quest context, cache item IDs and the selected inventory object.
- `src/lua_api/sim_substates/quest_log.rs` — empty-by-default party snapshots and quest special-item inputs.
- `src/lua_api/globals/missing_surface/tooltip_info/builders.rs` — shared tooltip-row and D56 publication path for supplied Retail party-progress data.
- `src/c_api/tooltip_getter_args.rs` — shared native-style argument checks.
- `src/c_api/item_spell/c_item.rs` — `GetItemNumSockets` secret argument gate.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs` — modern tooltip registration.

## Tests asserting this spec

- `tests/tooltip_round7.rs` — registration, argument conversion, usage errors, original accessor miss paths, secret rejection and populated owned runeforge item instances.
- `tests/tooltip_quest_party_progress.rs` — selected ObjectiveTracker consumers, original tooltip accessor selectors, exact missing-source counts, and supplied owned-item selection.
- `src/c_api/c_tooltip_info_round7.rs::tests::populated_azerite_selection_misses_without_rank_spell_data` — the existing current-rank/active-slot state is populated, while the missing rank-to-spell producer retains the no-values result.
- `tests/c_item_probes.rs` — existing socket counts and integer miss result.

## Known gaps (current cycle)

- [ ] Azerite state supplies current essence ranks and active slots, but the rank-to-spell tooltip catalogue is absent; the getters return no values after resolving only the modeled selection. LFG reward stores, possession spell/context, recipe rank chain, selected item-interaction descriptors and slotted-keystone GUID state are also not modeled.
- [ ] `GetQuestLogSpecialItem` keeps quest-ID context, quest-cache fields, and the selected inventory object as separate empty-by-default host inputs. The upstream cache, inventory/object writers, candidate predicates, and invalidation routes remain UNKNOWN. The native owner-GUID request context is not consumed by the shared simulator item builder.
- [ ] `GetQuestPartyProgress` has no native progress-tree writer or party-progress query service. Its host-supplied snapshots start empty; member/objective values are not derived. R88n.3 confirms the writer routes remain UNKNOWN. **UNKNOWN:** Native return shape when a successful query has no output rows (for example, `omitTitle=true` with every participant filtered); this simulator returns zero values when its supplied snapshot emits no rows.
- [ ] Native getter-specific RGB/data-instance publication beyond the shared builder is not independently established; the Retail implementation reuses the existing shared serializer.
- [ ] Runeforge power/modifier lookup and non-bag GUID object resolution are not modeled; those inputs do not produce a guessed tooltip.
- [ ] The native secret-result enable byte and mode are UNKNOWN. Do not claim that the socket count is always public or always secret.
- [ ] The VM does not expose Round 7's full secret-access mode selector. It supports the current untainted-access check; exact behavior for the other native mode messages is not modeled.

## Out of scope

- Inventing state, catalogs, candidate predicates, party-progress rows, or tooltip lines for absent producer inputs.
- Changing the native `GetItemNumSockets` count calculation or claiming a result-tag rule without its enable/mode semantics.
