# Round 7 tooltip getters

The Round 7 `C_TooltipInfo` getters use native argument readers and tooltip builders. Their contract and proof limits are recorded in the [tooltip-data wiki](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation).

## What it must do

- [x] Publish all eleven getters on Retail 12.1.5 (70688) and Forever 1.60.1 (70707); publish only `GetAzeriteEssence` and `GetAzeriteEssenceSlot` on PTR 12.1.7 (70668), as established in Round 7.
- [x] Use each getter's exact usage string, signed/unsigned conversion, optional argument, byte-slot, GUID, table and secret-argument rules from Round 7.
- [x] On Retail 12.1.5 (70688), missing-source calls to `GetQuestLogSpecialItem` and `GetQuestPartyProgress` return exactly zero Lua values, not `nil` or a tooltip table (R88n.1).
- [x] Retail LFG reward getters read only ready, host-supplied reward groups keyed by dungeon ID and type; ordinary entries and shortage tiers use their selected indexes, and empty or unready input does not fabricate rewards.
- [x] Use each LFG entry's currency discriminator to select the shared currency or item tooltip builder, not the numeric ID range.
- [x] On Retail 12.1.5 (70688), `GetRecipeRankInfo` follows supplied parent records to the base recipe, follows successor records whose spells exist, and publishes type 17 with the requested recipe ID.
- [x] `GetRecipeRankInfo` renders available spell descriptions as wrapped rows and uses the supplied rank as a display threshold for normal versus disabled color.
- [x] Optional recipe labels and next-rank source text use only host-supplied strings; the simulator does not synthesize a recipe catalogue or source lookup.
- [x] Retail recipe-rank misses for an absent recipe, required parent, or base spell return exactly zero Lua values (R88n.1).
- [x] Host cache insertion keeps the existing recipe record when the same recipe ID is supplied again (R88n.4a).
- [x] On Retail, clear the host cache on the modeled trade-skill initialization and dirty rebuild routes (R88n.4b).
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
- `src/c_api/recipe_rank_tooltip.rs` and `src/lua_api/globals/missing_surface/tooltip_info/recipe_rank.rs` — empty-by-default recipe display-cache input and shared rank tooltip composition.
- `src/lua_api/state/sim_state.rs` — host-supplied recipe rank cache storage.
- `src/c_api/quest_party_progress.rs` — host-supplied party-progress snapshot input.
- `src/c_api/quest_special_item.rs` — separate host inputs for index-to-quest context, cache item IDs and the selected inventory object.
- `src/lua_api/sim_substates/quest_log.rs` — empty-by-default party snapshots and quest special-item inputs.
- `src/lua_api/globals/missing_surface/tooltip_info/builders.rs` — shared tooltip-row and D56 publication path for supplied Retail party-progress data.
- `src/c_api/tooltip_getter_args.rs` — shared native-style argument checks.
- `src/c_api/item_spell/c_item.rs` — `GetItemNumSockets` secret argument gate.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs` — modern tooltip registration.
- `src/lua_api/state_types/lfg_rewards.rs` and `src/lua_api/state/sim_state.rs` — empty-by-default host input for Retail LFG reward groups.

## Tests asserting this spec

- `tests/tooltip_round7.rs` — registration, argument conversion, usage errors, original accessor miss paths, secret rejection and populated owned runeforge item instances.
- `tests/tooltip_recipe_rank_info.rs` — original `TooltipDataHandler` accessor selectors, cache duplicate retention, wrapped threshold-colored descriptions, exact miss arity, and modeled initialization/rebuild cache clears.
- `tests/tooltip_quest_party_progress.rs` — selected ObjectiveTracker consumers, original tooltip accessor selectors, exact missing-source counts, and supplied owned-item selection.
- `tests/tooltip_lfg_rewards.rs` — selected GroupFinder reward consumers, distinct ordinary/shortage entries and API-level no-value misses.
- `src/c_api/c_tooltip_info_round7.rs::tests::populated_azerite_selection_misses_without_rank_spell_data` — the existing current-rank/active-slot state is populated, while the missing rank-to-spell producer retains the no-values result.
- `tests/c_item_probes.rs` — existing socket counts and integer miss result.

## Known gaps (current cycle)

- [ ] Azerite state supplies current essence ranks and active slots, but the rank-to-spell tooltip catalogue is absent; the getters return no values after resolving only the modeled selection. Possession spell/context, selected item-interaction descriptors and slotted-keystone GUID state are also not modeled; these getters return no values.
- [ ] The native recipe-cache population writer, complete external invalidation triggers, learned-status semantics and player/faction source lookup remain unknown. R88n.4b verifies two direct clear callers; the simulator clears only on its modeled initialization and dirty rebuild routes. Recipe source text and labels are explicit host inputs; specialization wording is not derived.
- [ ] `GetQuestLogSpecialItem` keeps quest-ID context, quest-cache fields, and the selected inventory object as separate empty-by-default host inputs. The upstream cache, inventory/object writers, candidate predicates, and invalidation routes remain UNKNOWN. The native owner-GUID request context is not consumed by the shared simulator item builder.
- [ ] `GetQuestPartyProgress` has no native progress-tree writer or party-progress query service. Its host-supplied snapshots start empty; member/objective values are not derived. R88n.3 confirms the writer routes remain UNKNOWN. **UNKNOWN:** Native return shape when a successful query has no output rows (for example, `omitTitle=true` with every participant filtered); this simulator returns zero values when its supplied snapshot emits no rows.
- [ ] Native getter-specific RGB/data-instance publication beyond the shared builder is not independently established; the Retail implementation reuses the existing shared serializer.
- [ ] LFG reward entries retain their supplied quantity. Native currency quantity reaches the currency producer; the exact rendered quantity effects are not asserted. The shared simulator item builder has no quantity input, and native item-quantity effects are UNKNOWN; do not add a guessed quantity line.
- [ ] The LFG input seam does not model incoming reward updates or cache clearing. The public packet identity and complete clear routes are UNKNOWN.
- [ ] Runeforge power/modifier lookup and non-bag GUID object resolution are not modeled; those inputs do not produce a guessed tooltip.
- [ ] The native secret-result enable byte and mode are UNKNOWN. Do not claim that the socket count is always public or always secret.
- [ ] The VM does not expose Round 7's full secret-access mode selector. It supports the current untainted-access check; exact behavior for the other native mode messages is not modeled.

## Out of scope

- Inventing state, catalogs, candidate predicates, party-progress rows, recipe labels, specialization text, source lookups, or tooltip lines for absent producer inputs.
- Synthesizing LFG reward snapshots or guessing the incoming packet/cache lifecycle.
- Changing the native `GetItemNumSockets` count calculation or claiming a result-tag rule without its enable/mode semantics.
