# Supplied item cooldowns

Item cooldown queries must project independently supplied start/duration pairs unchanged. Input lives in `src/lua_api/state/`; C API queries live in `src/c_api/item_spell/`. See the [API inventory](../wiki/systems/lua-api.md) for architecture. Independent archived source `ffb6d7d7d8ab5a13be9bdd5c603b963380002656` proves the five literal requirements below in bounded Retail and Mists tests; corrected-fixture Mists GREEN is recorded at `bf1b61f10` below; current post-retirement HEAD, PTR, native and full-parent acceptance remain pending.

## What it must do

- [ ] Simulator-only `A_Admin.SetItemCooldown(itemID, duration)` sets the item's start to current `GetTime()` and supplied duration; duration `0` clears the input. [Admin API](../admin-api/combat.md#a_adminsetitemcooldownitemid-duration).
- [x] `C_Item.GetItemCooldown` resolves numeric ItemInfo and item hyperlinks to the same item ID and returns exactly `(number, number, boolean)`, with the supplied pair unchanged and enabled `true`.
- [x] `C_Container.GetItemCooldown(itemID)` selects a numeric item ID and returns exactly `(number, number, number)`, with the supplied pair unchanged and enabled `1`; it does not select an equipment slot.
- [x] `C_Container.GetContainerItemCooldown(bag, slot)` resolves the occupied item ID and returns its unchanged pair with numeric enabled `1`; this route is distinct from the single-item-ID query.
- [x] Explicit simulator assumption, not native parity: missing item records and empty bag slots return three ready values `(0, 0, true)` for C_Item or `(0, 0, 1)` for Container. The Container item-ID declaration permits no returns; this model instead requires the ready tuple.
- [x] Legacy `GetInventoryItemCooldown(unit, slot)` preserves independent equipment-slot pairs and three numeric returns, including `(0, 0, 1)` for a missing slot record. Item ID 13 must not alias equipment slot 13.

## How it works

- [API inventory](../wiki/systems/lua-api.md).

## Implementation inventory

- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: item-ID cooldown map and empty initializer, separate from equipment-slot input.
- `src/lua_api/globals/admin.rs`: simulator-only item cooldown input setter.
- `src/lua_api/globals/quest_surface/task_quest.rs`: legacy quest-special-item cooldown query over the item-ID map.
- `src/lua_api/game_data.rs`: existing `SpellCooldownState` start/duration pair.
- `src/c_api/item_spell/helpers.rs`: pure item-map pair projection; absent input yields `(0, 0)` by explicit model policy, not fallback compatibility or native parity.
- `src/c_api/item_spell/c_item.rs`: existing ItemInfo parser selects the passive item-map pair; boolean enabled `true`.
- `src/c_api/item_spell/c_container.rs`: separate numeric item-ID and bag/slot selectors project the same item map; numeric enabled `1`. Archived `ffb` execution proves these bounded routes, not current-HEAD coverage.
- `src/lua_api/globals/cooldown_probes.rs`: unchanged legacy equipment-slot getter.

## Tests asserting this spec

`tests/item_cooldowns.rs`: item input, four query-route tests, exact return counts/types/values, colliding item/slot IDs, and absent records. `tests/objective_tracker_dirty.rs` checks the original quest item button's cooldown frame and texture vertex color. Inserts overwrite fixture defaults; removals establish missing records and an empty slot. Genuine preproducer Retail execution at `e4caa4899`: four tests, legacy 1 PASS and query 3 FAIL, exit 101. All three queries fail their first `cooldown start value` assertion. C_Item numeric/hyperlink selector controls and Container bag controls were reached; subsequent hyperlink/collision/missing tuples were not reached. This historical RED is not superseded or relabeled by subsequent GREEN.

## Bounded archived GREEN — 2026-10-14

Independent concise reports bind execution to source `ffb6d7d7d8ab5a13be9bdd5c603b963380002656`: Retail item **4/4**, existing ready controls **2/2**, visibility **10/10** PASS; Mists item **4/4** PASS. All selected item tuple/link/collision/missing assertions were reached, with exactly three returns and concrete types/values:

- C_Item numeric/full hyperlink 6948: `(11,31,true)`; item 13: `(22,47,true)`; missing 999999: `(0,0,true)`.
- Container item-ID 6948/13/missing: `(11,31,1)` / `(22,47,1)` / `(0,0,1)`.
- Bag occupants `(0,8)=6948`, `(0,13)=13` yield the corresponding numeric tuples; empty `(0,15)` yields `(0,0,1)`.
- Legacy equipment slot 13 yields `(33,59,1)`, independent of item ID 13; missing equipment slot 15 yields `(0,0,1)`.

Core producer `ee1bc9038` remains unchanged; association with that producer does not extend `ffb` execution coverage to later source. Retail visibility includes a passing intentional precall-error handling case, not globally clean Lua execution. Zero `[store-dispatch]` output does not establish the inherited trace flag was unset: inherited environment was not captured.

[Store lifecycle status](store-product-group-shape.md) remains separate from item query acceptance. Temporary Store diagnostics were retired at `bc5cecffb`; their historical observations are not item query acceptance.

## Known gaps (current cycle)

- [x] Passive input/tests/spec committed as `e4caa4899`; independent full-stream gate confirms genuine Retail RED (legacy 1 PASS, query 3 FAIL, exit 101). First start-value assertions fail; exact actual starts were not printed. Later cooldown tuples remain unreached, not successful selection proof.
- [x] Five literal requirements have bounded archived `ffb` Retail/Mists GREEN above; this does not grant current-HEAD, PTR or full-parent proof.
- [x] Existing one-argument bag fixture at source `ffb` genuinely FAILs: argument2 is nil. `baec` fixes the source fixture to a valid bag/slot pair; its first PTR compile failed E0277 because `u32` results lack `FromRiluaResults`. `2d13` changes that result to `i32`. Independent Mists gate at compiled `bf1b61f10` passes corrected bag **1/1** and item **4/4**, all assertions reached. This is not PTR or post-retirement current-HEAD proof.
- [ ] Native behavior, expiry/elapsed-time rules, enable derivation, secret validation, name selection, profile preservation, and wider consumer/runtime proof remain pending. These are not exclusions from the parent goal.

## Current profile preservation — 2026-10-14

[Seven-profile revision/gate matrix](store-product-group-shape.md#seven-profile-saved-runtime-matrix--2026-10-14) owns current evidence: item4 runtime PASS on all seven profiles at their listed compiled sources. Mists overall/authentication remains PARTIAL. Historical checkpoints above retain their original revision scopes; current runtime does not authenticate source inputs or establish native/full-parent acceptance.

## Out of scope

No parent exclusions established by this passive-input stage.
