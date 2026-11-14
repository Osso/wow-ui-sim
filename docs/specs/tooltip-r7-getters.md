# Round 7 tooltip getters

The Round 7 `C_TooltipInfo` getters use native argument readers and tooltip builders. Their contract and proof limits are recorded in the [tooltip-data wiki](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation).

## What it must do

- [x] Publish all eleven getters on Retail 12.1.5 (70688) and Forever 1.60.1 (70707); publish only `GetAzeriteEssence` and `GetAzeriteEssenceSlot` on PTR 12.1.7 (70668), as established in Round 7.
- [x] Use each getter's exact usage string, signed/unsigned conversion, optional argument, byte-slot, GUID, table and secret-argument rules from Round 7.
- [x] Retail `GetAzeriteEssence` requires the essence record and exact selected-rank and rank-one records. It does not use a lower rank as a fallback; an explicit rank bypasses current-player rank lookup, and rank zero selects rank one (R88n.1/R88n.2).
- [x] Without an explicit rank, `GetAzeriteEssence` and `GetAzeriteEssenceSlot` use current player rank data. A present rank-list miss supplies zero before rank-one selection. Missing player or player essence state returns zero Lua values (R88n.1/R88n.2).
- [x] A failed tooltip formatter or spell producer returns exactly zero Lua values and does not publish a partial TooltipData result (R88n.1 failure boundary).
- [x] Retail Azerite Essence results use top-level TooltipData type 8 and essence ID. Main and passive powers use the shared spell producer. An active major slot selects the rank-one main description; any active slot selects the rank-one passive description. Otherwise each block uses the selected-rank actual spell (R88n.2).
- [x] Retail `GetAzeriteEssenceSlot` uses zero-based slots 0–3. Slot zero includes main and passive power blocks; slots one through three omit the main block and keep the passive block. It omits the name/rank and next-rank source stages (R88n.2).
- [x] INFERRED: map the four native spell-selector roles to the similarly named `AzeriteEssencePower` CSV columns from the exact-build 70391 export. Keep this mapping separate from verified rank and slot selection.
- [x] Compute the source-stage next rank from the original rank, not the rank normalized for spell selection. Next rank four disables this stage; other next ranks require an exact record. Next rank one uses a source heading and requires a nonempty source description; higher next ranks use a next-rank heading and the source description. Wrap the description when emitted (R88n.2). The simulator's `SOURCE`/`TOOLTIP_TALENT_NEXT_RANK` key mapping, Alliance/Horde selection by modeled race faction, and source/heading row type are INFERRED.
- [ ] UNKNOWN: R88n.2 does not define missing/empty source behavior. The producer omits an unavailable source without claiming native parity.
- [ ] UNKNOWN: the inline major/minor description rows have no verified relative order, and the `powerState="gray"` selector is unknown. Requirement-row production is also unknown; do not emit guessed rows.
- [x] On Retail 12.1.5 (70688), missing-source calls to `GetQuestLogSpecialItem` and `GetQuestPartyProgress` return exactly zero Lua values, not `nil` or a tooltip table (R88n.1).
- [x] Retail LFG reward getters read only ready, host-supplied reward groups keyed by dungeon ID and type; ordinary entries and shortage tiers use their selected indexes, and empty or unready input does not fabricate rewards.
- [x] Use each LFG entry's currency discriminator to select the shared currency or item tooltip builder, not the numeric ID range.
- [x] On Retail 12.1.5 (70688), `GetRecipeRankInfo` follows supplied parent records to the base recipe, follows successor records whose spells exist, and publishes top-level TooltipData type 17 with the requested recipe ID (R88n.4).
- [x] The supplied rank qualitatively selects normal versus disabled description color (VERIFIED, R88n.4). The selector's concrete rank-1/rank-2 boundary is INFERRED coverage; exact color globals, RGB packing, rank numbering, and text-row wrapping are also INFERRED simulator policy.
- [x] Optional recipe labels and next-rank source text use only host-supplied strings; the simulator does not synthesize a recipe catalogue or source lookup.
- [x] R88n.1 verifies zero Lua values after a failed recipe-rank producer. The absent-recipe preservation case is tested; choosing missing parent/base spell or color globals as failure predicates is INFERRED simulator policy.
- [x] Host cache insertion keeps the existing recipe record when the same recipe ID is supplied again (R88n.4a); the simulator cache models only key retention and lookup.
- [x] INFERRED simulator-route behavior: `C_TradeSkillUI.OpenTradeSkill` and profession-spell redirection share an initialization helper that clears the cache; a changed recipe filter queues a pending update that clears on `begin_tick` before `TRADE_SKILL_LIST_UPDATE`. The tick path is notification only; it does not rebuild rows.
- [x] For `GetRuneforgeResultItem`, resolve the supplied GUID to an owned modeled item instance, preserve its identity, apply the supplied displayed level, and use the shared item tooltip builder when no unmodeled power/modifiers are requested.
- [x] `C_Item.GetItemNumSockets` must reject a secret `itemInfo` argument from a tainted caller on Retail/PTR/Forever with the modeled native `AllowedWhenUntainted` error; an untainted caller may read the argument.
- [x] On Retail, `GetQuestLogSpecialItem` resolves the one-based index through host-supplied quest-ID request context, reads primary/candidate item IDs from a separate host-supplied cache record, and uses only a host-supplied selected inventory object whose ID matches that record. It uses the shared item builder and publishes the selected instance GUID.
- [x] On Retail, `GetQuestPartyProgress` publishes only an explicit host-supplied snapshot with type 24 and quest ID, the optional type-17 title row, supplied type-18 participant headings, and type-8 status/objective rows. The host supplies row text, colors, and participant-filter classification; no party/objective values are seeded.
- [x] `omitTitle` omits the supplied title row, and `ignoreActivePlayer` filters only participant groups marked by the host.
- [x] Optional Retail Azerite tests exercise the original selected consumer for exact essence/rank, active-slot modes, nonempty source text under the INFERRED faction-column mapping, and next-rank edges; the slot getter retains its mode-specific blocks and the miss paths retain zero-value arity.
- [ ] The broader Round 7 tests exercise recipe-rank selectors and inferred routes, the cross-feature PRESERVATION miss cases, argument errors, and backed runeforge instances.

## How it works

- [Tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation) — argument readers, registration, producers and missing state.
- [Azerite Essence tooltip selection](../wiki/systems/lua-api.md#azerite-essence-tooltip-selection-r88n2) — catalog, selected consumers, and evidence limits.
- [Consumer-selection investigation](../wiki/investigations/consumer-selection-and-red-proof.md#retail-azerite-essence-tooltip-consumers) — source selection and RED/PRESERVATION proof.
- [Item socket count](item-num-sockets.md) — count and secrecy boundary.

## Implementation inventory

- `src/c_api/c_tooltip_info_round7.rs` — Round 7 publication, arguments, missing-state decisions and owned-item runeforge path.
- `src/c_api/c_tooltip_info_azerite_essence.rs` — Retail essence/slot rank selection, type-8 composition, and the bounded next-rank source stage.
- `src/c_api/azerite_essence_spell_selector.rs` — isolated INFERRED selector-to-column mapping.
- `src/c_api/tooltip_azerite_catalog_data.rs` — generated essence, rank and source-text catalog from the 12.1.5.70391 DB2 CSV inputs.
- `tools/gen_tooltip_azerite_catalog.py` and `data/db2/retail-12.1.5.70391/` — catalog generator, source tables and provenance.
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
- `tests/tooltip_azerite_essence.rs` — optional Retail original Azerite UI consumers, distinct essence/rank selectors, slot modes, next-rank source text, and zero-value misses. Spell tests compare all raw DB2 column candidates; source-column selection by modeled faction remains INFERRED.
- `tests/tooltip_recipe_rank_info.rs` — VERIFIED recipe-ID selection and qualitative threshold behavior through the original accessor; the concrete rank-1/rank-2 boundary and exact presentation are INFERRED coverage; simulator-route tests are INFERRED; the zero-value miss test is PRESERVATION.
- `tests/tooltip_quest_party_progress.rs` — selected ObjectiveTracker consumers, original tooltip accessor selectors, exact missing-source counts, and supplied owned-item selection.
- `tests/tooltip_lfg_rewards.rs` — selected GroupFinder reward consumers, distinct ordinary/shortage entries and API-level no-value misses.
- `src/c_api/c_tooltip_info_round7.rs::tests::populated_azerite_selection_misses_without_rank_spell_data` — the existing current-rank/active-slot state is populated, while the missing rank-to-spell producer retains the no-values result.
- `tests/c_item_probes.rs` — existing socket counts and integer miss result.

## Recipe-rank evidence boundary

| Classification | Evidence and model scope |
|---|---|
| VERIFIED (Retail 12.1.5.70688, R88n.4a) | The native rebuild inserts new cache keys, retains existing duplicates, appends/counts/grows the display vector, and projects favorite/crafting-order flags before row copy. The simulator cache models key retention and lookup only; it does not model the native vector or flags. |
| VERIFIED (Retail 12.1.5.70688, R88n.4b) | The native clear routine has two direct callers: trade-skill initialization and rebuild. A dirty-state check can reach rebuild. This does not identify every external trigger. |
| INFERRED (simulator routes) | `OpenTradeSkill` and profession-spell redirection share the simulator initialization helper. A changed name filter queues an update; `begin_tick` clears before the event. This is a deferred notification path, not row rebuild/population. The matching native public triggers and timing are not established. |
| INFERRED (simulator presentation) | Text-line type 0; color globals and packed RGB; one-based ranks and exact comparison boundary; `": "` label text; `TRADESKILL_RANK_HEADER` / `TRADESKILL_NEXT_RANK_HEADER`; blank separators; next-source placement; `wrapText`; cycle handling; and missing-parent/base-spell/color failure predicates. |
| PRESERVATION | R88n.1 verifies zero Lua values after failed producer completion. The miss-only test preserves that return boundary; it does not prove the producer selector. |
| UNKNOWN | Complete display-row member semantics, upstream data acquisition, all external clear triggers, learned-status semantics, and player/faction source lookup. |

## Known gaps (current cycle)

- [ ] Retail `GetAzeriteEssence` does not compose inline major/minor next-rank rows, the `powerState="gray"` alternate, or requirement-localized lines. Source text uses an INFERRED mapping from modeled faction to DB2 columns; empty-source behavior remains UNKNOWN and is not claimed as native parity. Possession spell/context, selected item-interaction descriptors and slotted-keystone GUID state are also not modeled; these getters return no values.
- [ ] Complete recipe display-row semantics, upstream acquisition, external clear triggers, learned-status meaning and player/faction source lookup remain UNKNOWN; see [Recipe-rank evidence boundary](#recipe-rank-evidence-boundary). Recipe source text and labels are host inputs; specialization wording is not derived.
- [ ] `GetQuestLogSpecialItem` keeps quest-ID context, quest-cache fields, and the selected inventory object as separate empty-by-default host inputs. The upstream cache, inventory/object writers, candidate predicates, and invalidation routes remain UNKNOWN. The native owner-GUID request context is not consumed by the shared simulator item builder.
- [ ] `GetQuestPartyProgress` has no native progress-tree writer or party-progress query service. Its host-supplied snapshots start empty; member/objective values are not derived. R88n.3 confirms the writer routes remain UNKNOWN. **UNKNOWN:** Native return shape when a successful query has no output rows (for example, `omitTitle=true` with every participant filtered); this simulator returns zero values when its supplied snapshot emits no rows.
- [ ] Native getter-specific RGB/data-instance publication beyond the shared builder is not independently established; the Retail implementation reuses the existing shared serializer.
- [ ] LFG reward entries retain their supplied quantity. Native currency quantity reaches the currency producer; the exact rendered quantity effects are not asserted. The shared simulator item builder has no quantity input, and native item-quantity effects are UNKNOWN; do not add a guessed quantity line.
- [ ] The LFG input seam does not model incoming reward updates or cache clearing. The public packet identity and complete clear routes are UNKNOWN.
- [ ] Runeforge power/modifier lookup and non-bag GUID object resolution are not modeled; those inputs do not produce a guessed tooltip.
- [ ] The native secret-result enable byte and mode are UNKNOWN. Do not claim that the socket count is always public or always secret.
- [ ] The VM does not expose Round 7's full secret-access mode selector. It supports the current untainted-access check; exact behavior for the other native mode messages is not modeled.

## Out of scope

- Inventing state, catalogs, candidate predicates, party-progress rows, recipe labels, specialization text, or source lookups for absent producer inputs.
- Synthesizing LFG reward snapshots or guessing the incoming packet/cache lifecycle.
- Changing the native `GetItemNumSockets` count calculation or claiming a result-tag rule without its enable/mode semantics.
