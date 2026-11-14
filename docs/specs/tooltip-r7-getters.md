# Round 7 tooltip getters

The Round 7 `C_TooltipInfo` getters use native argument readers and tooltip builders. Their contract and proof limits are recorded in the [tooltip-data wiki](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation).

## What it must do

- [x] Publish all eleven getters on Retail 12.1.5 (70688) and Forever 1.60.1 (70707); publish only `GetAzeriteEssence` and `GetAzeriteEssenceSlot` on PTR 12.1.7 (70668), as established in Round 7.
- [x] Use each getter's exact usage string, signed/unsigned conversion, optional argument, byte-slot, GUID, table and secret-argument rules from Round 7.
- [x] On Retail 12.1.5 (70688), missing-source calls to `GetQuestLogSpecialItem` and `GetQuestPartyProgress` return exactly zero Lua values, not `nil` or a tooltip table (R88n.1).
- [x] On Retail 12.1.5 (70688), `GetItemInteractionItem` returns zero values for a missing object or a failed kind-2/4 producer. A supplied object of any other kind returns one table with exactly one field, `lines`, containing an empty table. This path bypasses the builder and does not add `dataInstanceID` (VERIFIED, R88n.1).
- [x] The simulator's INFERRED interface for this object is a separate empty-by-default host input beside the interaction dialog state. Kind-2/4 producers are not modeled, and their supplied objects take the verified failure path. Registration and argument handling are unchanged; PTR remains unpublished and Forever retains its no-value result.
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
- [x] Tests exercise original selected consumers, the VERIFIED recipe-ID selector and qualitative rank-threshold behavior through the original accessor, the INFERRED concrete rank-1/rank-2 boundary and presentation, inferred simulator routes, the PRESERVATION miss case, argument errors, and backed runeforge instances.

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
- `src/c_api/c_item_interaction.rs` and `src/lua_api/state/sim_state.rs` — explicit selected tooltip object kind, separate from dialog content and cost.
- `src/c_api/tooltip_getter_args.rs` — shared native-style argument checks.
- `src/c_api/item_spell/c_item.rs` — `GetItemNumSockets` secret argument gate.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs` — modern tooltip registration.
- `src/lua_api/state_types/lfg_rewards.rs` and `src/lua_api/state/sim_state.rs` — empty-by-default host input for Retail LFG reward groups.

## Tests asserting this spec

- `tests/tooltip_round7.rs` — registration, argument conversion, usage errors, original accessor miss paths, secret rejection and populated owned runeforge item instances.
- `tests/tooltip_item_interaction_item.rs` — Retail-selected ItemInteraction slot through the original GameTooltip and TooltipDataHandler, and a separate exact Lua return-count and shape check.
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

- [ ] Azerite state supplies current essence ranks and active slots, but the rank-to-spell tooltip catalogue is absent; the getters return no values after resolving only the modeled selection. Possession spell/context and slotted-keystone GUID state are also not modeled; these getters return no values.
- [ ] Native item-interaction object selection actions, lifetime, events, and kind-2/4 success payload producers remain UNKNOWN; the simulator invents no writer, event or payload.
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

- Inventing state, catalogs, candidate predicates, party-progress rows, recipe labels, specialization text, source lookups, or tooltip lines for absent producer inputs.
- Synthesizing LFG reward snapshots or guessing the incoming packet/cache lifecycle.
- Changing the native `GetItemNumSockets` count calculation or claiming a result-tag rule without its enable/mode semantics.
