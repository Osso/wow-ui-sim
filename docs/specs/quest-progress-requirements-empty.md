# Supplied quest progress requirements

Retail and WoW Forever model quest-giver progress text, required items and currencies, required money, and turn-in portrait input for an active NPC dialog. The model lives in `src/c_api/quest_objectives.rs`; the simulator-only producer is `A_Admin.SetQuestProgressRequirements`. The legacy temporary defaults remain byte-identical, and modeled globals register after them for Retail and WoW Forever. [Quest architecture](../wiki/systems/lua-api.md#quest-dialog-material) and the [execution receipt](../baselines/quest-lifecycle.md) own implementation details and proof.

## What it must do

- [x] Accept requirements for a quest ID and display its supplied values when that quest is selected in the active NPC dialog.
- [x] Show supplied progress text, required item and currency rows, and required money in Blizzard's original progress panel.
- [x] Render required item rows when the supplied item-row and quest-level visibility inputs are both false.
- [x] Return the supplied portrait display ID, text, and name in the order consumed by Blizzard's original turn-in panel. This order is inferred from the cached consumer; native evidence is pending (R88db).
- [x] Keep the no-input progress panel empty. Existing temporary defaults remain in the simulator for compatibility outside this modeled Retail/Forever behavior.
- [ ] Verify inferred defaults and field semantics against native evidence (R88db pending).

These are supplied-state and original-consumer contracts, not claims of native requirement discovery, inventory/currency ownership, or reward acquisition.

## How it works

- [Quest model and dialog architecture](../wiki/systems/lua-api.md#quest-dialog-material)
- [QL-2 readiness and reward-dialog transition](quest-readiness-dialog.md)

## Implementation inventory

- `src/c_api/quest_objectives.rs`: progress-requirement rows and active-dialog projection.
- `src/c_api/c_quest_offer.rs`: required-currency and hide-required-item queries.
- `src/lua_api/globals/admin_quest_objectives.rs`: validated simulator-only input.
- `src/lua_api/globals/real/quest_dialog.rs`: modeled progress globals, including per-row visibility.
- `src/lua_api/globals/quest_surface/register.rs` and `src/lua_api/env_init/mod.rs`: Retail/Forever registration after temporary defaults.
- `src/lua_api/workarounds/temporary/quest_progress_requirements.rs`: unchanged compatibility defaults.

## Tests asserting this spec

`tests/quest_progress_requirements.rs` loads the cached Retail or WoW Forever Blizzard UI closure, enters through `A_Admin.OpenMultiQuestNpc(); SelectActiveQuest(2)`, and checks two distinct supplied requirement sets through the original progress panel. It checks text, item names/counts, currency names/counts, required money, visibility, and portrait inputs. `tests/quest_dialog_requirements.rs` covers the no-input panel behavior.

## Known gaps (current cycle)

- [ ] Native behavior for portrait tuple order, item-hidden return values, and absent-input defaults remains unverified (R88db pending).

## Out of scope

Discovering requirements from native quest data, checking player inventory or currency, claiming native parity, granting rewards, and adding PTR behavior are outside this contract.
