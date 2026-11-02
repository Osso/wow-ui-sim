# Additional inventory and currency discrepancies

Static comparison at the [expanded-pass revisions and profile limits](additional-rendering-and-map.md). Legacy global names, tuple returns and item-backed old currencies are **not** universal modern contracts. Modern namespace successors count as implementations when their actual behavior is modeled. No bag/currency operation was executed.

## B1 — Stack splitting is a callable no-op

**External:** `WoW/Source/Ui/ContainerFrame_Scripts.cpp:779-817`, validates an accessible unlocked item and amount, clears prior cursor/targeting, installs an item cursor with a pending split amount and locks the source. `WoW/Source/Ui/GameUI.cpp:5794-5826`, retains item/container/slot identity and `m_stackSplit` in cursor state.

**Simulator:** `src/lua_api/workarounds/temporary/container_default_shapes.rs:44-47` installs an empty `C_Container.SplitContainerItem` function when no provider exists. The existing cursor represents item ID/count/origin, not a pending split transaction. The local shape test demonstrates callability only.

**Source-derived fixture:** item 2589, count 7, at bag `(0,5)`; request split amount 3. Reconstruction selects the item/split amount and locks it; the placeholder does nothing. **Do not assert the source immediately changes count to 4:** the inspected path records pending selection/lock, not completed placement/server reconciliation. The broad explorer's immediate-subtraction fixture was rejected.

**Rework/regression:** implement the target's pending cursor/split, lock and placement/completion transitions in the inventory backing model. Test selection and cancellation separately from committed source/destination counts, invalid amounts, locked source and placement/merge outcomes. Use simulator-supplied state/completion inputs; do not add real network traffic or fabricate an unobserved native commit boundary.

## B2 — Whole-item cursor transfer discards instance information and lacks item-state validation

**External:** `ContainerFrame_Scripts.cpp:416-459`, operates on item/container identity with control, bank-access and lock checks. `GameUI.cpp:5794-5826` retains item GUID rather than reconstructing an item from its template ID; the source's subsequent item lookup can still address the same instance.

**Simulator:** `src/c_api/container_inventory.rs:29-63` removes/inserts whole `BagItem` records and puts only ID/count/origin on the cursor. When dropping it recreates `BagItem { item_id, stack_count, hyperlink: None }`. Its module explicitly states that hyperlinks, enchants and gems do not survive cursor transfer. `src/lua_api/state_types/collections.rs:624-628` has no bag lock state; `src/c_api/item_spell/c_container.rs:185-198` hardcodes lock/readability/binding-related flags rather than projecting an actual instance. Existing ordinary whole-item swapping is implemented; this is not an absent cursor API.

**Counterexample:** supply a bag item with a concrete non-default hyperlink, record `GetContainerItemLink`, pick up and drop it into another valid slot, then query its link. Simulator drops the supplied instance hyperlink while preserving only base ID/count. Separately, no supplied locked-instance state can stop the transfer through this path.

**Rework/regression:** preserve the actual item instance through cursor movement and derive query/permission/lock behavior from that model. Assert exact instance link/state before and after a round trip, two distinct same-ID instances, swapping and locked/inaccessible-bank controls. Avoid “reconstruct a plausible default item” as transfer semantics. Legacy versus modern bag mapping and exact lock/ack timing still need target-profile evidence.

## B3 — Container item use has no stateful dispatch

**External:** `ContainerFrame_Scripts.cpp:266-411`, resolves bag/slot/target, checks lock and current context, then dispatches item targeting, repair, bank storage, sale, mail, auction, trade, equipment or item-use behavior. Not every branch is the same as casting a spell.

**Simulator:** `src/lua_api/workarounds/temporary/container_default_shapes.rs:39-42` supplies an empty `C_Container.UseContainerItem` when no provider exists. Legacy item-use names also appear among generic stubs (`src/lua_api/globals/stubs/global_stubs.rs:92-95`). The placeholder's `pcall` success does not demonstrate consumption, equipment or contextual use.

**Source-derived fixture:** a supplied usable item in a valid slot and otherwise ordinary context should reach the modeled item action; the placeholder leaves state unchanged. Supported equip/merchant/bank contexts require their own observable transition, not a single success flag for every case. Those input/dispatch capabilities are missing; no currently runnable native-equivalent fixture is claimed.

**Rework/regression:** implement the required item-use/context dispatcher backed by real simulator inventory and existing context models, rather than replacing the empty function with another return shape. Cover ordinary use, a supported contextual operation, missing/locked item and target selection; assert concrete inventory/effect/event changes at the appropriate completion boundary. This audit does not authorize implementing every original-client service or live networking.

## B4 — Quest-item metadata and quest-log relationship are always absent

**External:** `ContainerFrame_Scripts.cpp:645-679`, derives quest-bound/start-quest/active-quest information from item metadata and current quest state; missing item/stats has its own result.

**Simulator:** `src/lua_api/workarounds/temporary/container_default_shapes.rs:17-25` always returns `{ isQuestItem=false, questID=nil, isActive=false }` regardless of actual bag contents, item binding/start-quest metadata or quest-log membership.

**Source-derived fixture:** occupied quest-start item with a supplied quest ID; compare before/after the quest is active, plus an ordinary item and empty slot. The placeholder cannot distinguish any of them.

**Rework/regression:** resolve the occupied instance, authoritative item/quest metadata and quest-log membership, returning the supported modern table or legacy shape at the appropriate API boundary. Test populated fields and the active-state transition, not merely table type/default values. Do not hardcode a particular item's quest relationship or transplant the old multi-return tuple into the modern namespace.

## B5 — Currency list expansion/watch state is a static catalogue, not a mutable display list

**External:** `WoW/Source/Ui/CurrencyTypes_Scripts.cpp:187-224`, mutates category expansion, unused and backpack selection then rebuilds the indexed display list. `:99-182` projects current display rows rather than a permanently expanded constant catalogue.

**Simulator:** `src/lua_api/globals/currency_data.rs:6-110` stores a static `CURRENCY_LIST`; headers are always expanded and watched entries are fixed constants. `src/c_api/item_spell/c_currency.rs:125-205` uses live supplied name/quantity/icon metadata for currency entries, **but** status fields still come from static row flags; backpack selection filters those same static entries. No mutable category/display-list or watch-selection mutation path was found. Successful quantity projection already exists and is not counted as missing.

**Source-derived fixture:** a header with two currencies, initially expanded; collapse it, expand it, and change the watched currency. Reconstruction changes visible indexed rows/watch state; the inspected simulator catalogue cannot represent those transitions. Do not infer API availability from namespace callability or from absence of an old global alone.

**Rework/regression:** own mutable list/category/watch state, project visible rows and relevant modern status fields from it, and connect supported mutation/input paths. Test header row counts/order, collapse/expand, watch/unwatch and changed quantities without losing item identity. Map the correct target's currency APIs and unused policy before implementing.

## Historical differences not promoted as new requirements

| Lead | Disposition |
|---|---|
| Old currency quantities derive from bag items | Old item-backed currencies are not proof that modern account currency quantities should come from bag stacks. Keep current supplied currency quantities; resolve profile-specific resource semantics separately. |
| Legacy `GetContainerItemInfo` tuple versus modern table/nil | API-version migration, not itself a bug. B2/B4 concern populated state/relationships, not reproducing retired return shapes. |
| Item cooldown enabled/charge derivation | Historical source differs, but [the current item-cooldown spec](../specs/item-cooldowns.md) explicitly requires enabled 1/true for supplied pairs and names native enable derivation as a pending gap. Record that contract conflict, not an automatic change to the tested model. |
| Refund, socketing, purchase restrictions and special bank mapping | Inventory found these surfaces, but their backing models/full lifecycle and supported-profile contracts were not sufficiently traced for new accepted findings here. |

Ordinary occupied-bag free-slot lists/counts and whole-stack swap/drop have intersecting tested simulator behavior; this does not establish banking, pending split, per-instance metadata or current native parity.
