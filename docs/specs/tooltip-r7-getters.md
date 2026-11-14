# Round 7 tooltip getters

The Round 7 `C_TooltipInfo` getters use native argument readers and tooltip builders. Their contract and proof limits are recorded in the [tooltip-data wiki](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation).

## What it must do

- [x] Publish all eleven getters on Retail 12.1.5 (70688) and Forever 1.60.1 (70707); publish only `GetAzeriteEssence` and `GetAzeriteEssenceSlot` on PTR 12.1.7 (70668), as established in Round 7.
- [x] Use each getter's exact usage string, signed/unsigned conversion, optional argument, byte-slot, GUID, table and secret-argument rules from Round 7.
- [x] On a missing backing source, return no Lua values, not one `nil` value.
- [x] Retail LFG reward getters read only ready, host-supplied reward groups keyed by dungeon ID and type; ordinary entries and shortage tiers use their selected indexes, and empty or unready input does not fabricate rewards.
- [x] Use each LFG entry's currency discriminator to select the shared currency or item tooltip builder, not the numeric ID range.
- [x] For `GetRuneforgeResultItem`, resolve the supplied GUID to an owned modeled item instance, preserve its identity, apply the supplied displayed level, and use the shared item tooltip builder when no unmodeled power/modifiers are requested.
- [x] `C_Item.GetItemNumSockets` must reject a secret `itemInfo` argument from a tainted caller on Retail/PTR/Forever with the modeled native `AllowedWhenUntainted` error; an untainted caller may read the argument.
- [x] Tests must exercise original `TooltipDataHandler` accessors for selected native consumers, argument errors, no-value misses, and backed runeforge instances.

## How it works

- [Tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation) — argument readers, registration, producers and missing state.
- [Item socket count](item-num-sockets.md) — count and secrecy boundary.

## Implementation inventory

- `src/c_api/c_tooltip_info_round7.rs` — Round 7 publication, arguments, missing-state decisions and owned-item runeforge path.
- `src/c_api/tooltip_getter_args.rs` — shared native-style argument checks.
- `src/c_api/item_spell/c_item.rs` — `GetItemNumSockets` secret argument gate.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs` — modern tooltip registration.
- `src/lua_api/state_types/lfg_rewards.rs` and `src/lua_api/state/sim_state.rs` — empty-by-default host input for Retail LFG reward groups.

## Tests asserting this spec

- `tests/tooltip_round7.rs` — registration, argument conversion, usage errors, original accessor miss paths, secret rejection and populated owned runeforge item instances.
- `tests/tooltip_lfg_rewards.rs` — selected GroupFinder reward consumers, distinct ordinary/shortage entries and API-level no-value misses.
- `src/c_api/c_tooltip_info_round7.rs::tests::populated_azerite_selection_misses_without_rank_spell_data` — the existing current-rank/active-slot state is populated, while the missing rank-to-spell producer retains the no-values result.
- `tests/c_item_probes.rs` — existing socket counts and integer miss result.

## Known gaps (current cycle)

- [ ] Azerite state supplies current essence ranks and active slots, but the rank-to-spell tooltip catalogue is absent; the getters return no values after resolving only the modeled selection. Possession spell/context, quest special-item object relation, quest-party progress tree, recipe rank chain, selected item-interaction descriptors and slotted-keystone GUID state are also not modeled; these getters return no values.
- [ ] LFG reward entries retain their supplied quantity. Native currency quantity reaches the currency producer; the exact rendered quantity effects are not asserted. The shared simulator item builder has no quantity input, and native item-quantity effects are UNKNOWN; do not add a guessed quantity line.
- [ ] The LFG input seam does not model incoming reward updates or cache clearing. The public packet identity and complete clear routes are UNKNOWN.
- [ ] Runeforge power/modifier lookup and non-bag GUID object resolution are not modeled; those inputs do not produce a guessed tooltip.
- [ ] The native secret-result enable byte and mode are UNKNOWN. Do not claim that the socket count is always public or always secret.
- [ ] The VM does not expose Round 7's full secret-access mode selector. It supports the current untainted-access check; exact behavior for the other native mode messages is not modeled.

## Out of scope

- Inventing state, catalogs, or tooltip lines for absent producer inputs.
- Synthesizing LFG reward snapshots or guessing the incoming packet/cache lifecycle.
- Changing the native `GetItemNumSockets` count calculation or claiming a result-tag rule without its enable/mode semantics.
