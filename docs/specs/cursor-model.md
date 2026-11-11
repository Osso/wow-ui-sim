# Cursor model (Retail and PTR ActionBar)

Retail uses one shared clear and placement model for cursor payloads (`src/lua_api/globals/real/cursor_model.rs`). PTR 12.1.7 uses the native cursor projection for modeled payloads and the shared transitions for `PickupAction` / `PlaceAction`; other profiles keep their earlier behavior. See the [cursor model system](../wiki/systems/cursor-model.md) for implementation details.

## What it must do

### Native modes

| Simulator payload | Producer | Mode |
|---|---|---|
| Owned bag or equipped item | `PickupContainerItem`, `SplitContainerItem`, `PickupInventoryItem`, `EquipCursorItem` | 1 Item |
| Money | `PickupPlayerMoney` | 2 Money |
| Spell, or an action slot holding a spell | `PickupSpell`, `C_Spell.PickupSpell`, `C_SpellBook.PickupSpellBookItem`, `PickupAction` | 3 Spell |
| Pet action | `PickupPetAction` | 4 PetAction |
| Merchant row | `PickupMerchantItem` | 5 Merchant |
| Item ID | `C_Item.PickupItem` | 6 ActionBar |
| Macro | `PickupMacro`, `PickupAction` | 7 Macro |
| Equipment set | `C_EquipmentSet.PickupEquipmentSet` | 12 EquipmentSet |
| Currency | `C_CurrencyInfo.PickupCurrency` | 13 Currency |
| Transmog outfit | `C_TransmogOutfitInfo.PickupOutfit`, `PickupAction` | 21 Outfit |

VERIFIED (WM46): the mode names and the PickupAction rule that the slot's content selects the setter.

### PTR 12.1.7 ActionBar contract (R29)

- [ ] After `PickupAction(slot)` from a spell slot, `GetCursorInfo()` returns five values: `"spell"`, book slot or 0, `"spell"`, spell ID, and related ID or nil. The spell ID is the fourth value; an empty cursor returns zero values.
- [ ] `PickupAction(slot)` removes the action before notifying its source slot. `PickupAction(slot, true)` copies it without removing the source.
- [ ] Pickup event order is `ACTIONBAR_SHOWGRID`, `CURSOR_CHANGED`, `ACTIONBAR_SLOT_CHANGED(sourceSlot)`, then one queued/coalesced `ACTIONBAR_UPDATE_COOLDOWN`; no `ACTIONBAR_UPDATE_STATE` post is sent.
- [ ] Placing into an empty slot hides the grid, clears the cursor, notifies the target slot, then queues `ACTIONBAR_UPDATE_COOLDOWN`.
- [ ] Placing onto another action swaps the displaced action onto the cursor. A spell-to-spell swap shows the grid before notifying the target and suppresses the final `CURSOR_CHANGED` because the cursor mode stays Spell.
- [ ] Placing the same resolved spell clears the cursor without changing or notifying the target slot.
- [ ] During combat lockdown, a tainted caller's pickup is blocked before any change. A blocked place reports `ADDON_ACTION_BLOCKED`, clears the cursor, and does not assign the target.
- [ ] Action-bar lock settings are checked by the Lua drag consumer, not by `PickupAction` or `PlaceAction`; the receive-drag consumer has no lock check.
- [ ] PTR action slots are 1–360; Retail action slots are 1–180.
- [ ] PTR PetAction categories 6 and 7 return a book index or 0 as the third value and nil as the fourth. Retail returns nil as the third value for these categories.

### Stored ID and grid

- The shared stored item ID S is written by the merchant (its item ID) and ActionBar (its item ID) setters. Clearing zeros S only when the old mode is 5, 6, 10, 18, 19 or 20; other modes keep it. VERIFIED (WM46).
- The grid flag is set by pickups of modes 3, 4, 6, 7, 12, 13, 14, 16, 17 and 21, after `ACTIONBAR_SHOWGRID`. VERIFIED (WM46).
- An owned item (mode 1) shows the grid unless its class is 17, and only if it has a nonzero use spell (the `GetItemSpell` effect route, trigger 0) or its base Item record has a nonzero InventoryType; a missing record gives zero. VERIFIED (WM54 Q4). The inputs come from `crates/sim-data/data/item_cursor_grid.rs`, generated from `Item.csv`. Cursor items carry no enchantments in this model, so the enchantment use-spell route does not apply.

### Clear

`ClearCursor`, `DeleteCursorItem`, `PlaceAction`, `DropCursorMoney`, `PutItemInBackpack`/`PutItemInBag` and dropping an owned item into an empty bag or equipment slot run the shared clear:

1. If the grid flag is set, `ACTIONBAR_HIDEGRID`, then the flag is reset.
2. For a nonempty cursor, `CURSOR_CHANGED(true, 0, oldMode, S before the clear)`. An empty cursor emits nothing.

VERIFIED (WM46, WM44-A).

### Placement

Every producer above first runs the shared clear, even from an empty cursor, then stores the payload. VERIFIED (WM54 Q2). It then emits `ACTIONBAR_SHOWGRID` when the pickup shows the grid, then `CURSOR_CHANGED(false, mode, previous, id)`:

| Setter | previous | id | Fires when |
|---|---|---|---|
| Typed: money, spell, pet action, macro, outfit | Mode saved before the clear | 0 | previous ≠ mode |
| Owned item (mode 1) | Mode saved after the clear | 0 | previous ≠ 1 |
| Item ID: merchant, ActionBar | Mode saved after the clear | S saved after the clear | previous ≠ mode |

VERIFIED (WM54 Q1; WM46 and WM44-A for the item-ID row). A same-mode typed replacement, such as spell to spell, emits only the clear's event. An owned item replacing an owned item still emits both, because the clear empties the cursor first.

The first two `CURSOR_CHANGED` arguments, for both the clear and the set-side events, are read from the cursor when the event fires, so a handler that changes the cursor changes them. The third and fourth are the saved values. VERIFIED (WM54 Q1).

Swapping an owned item with a bag or equipment slot places the displaced item through the same setter, which first clears the held one. INFERRED.

### Predicates and projections

- `CursorHasItem()` is true only for mode 1, so merchant and ActionBar cursors fail it. `CursorHasMoney()` tests mode 2, `CursorHasSpell()` mode 3 and `CursorHasMacro()` mode 7. VERIFIED (WM46).
- `GetCursorInfo()` returns these tuples. VERIFIED (WM46) unless marked.

  | Mode | Values |
  |---|---|
  | 1 Item | `"item"`, item ID, item link or nil. The cursor carries no owned modifiers, so the link is the one built from the item ID. |
  | 2 Money | `"money"`, amount |
  | 3 Spell | `"spell"`, main-book slot or 0, `"spell"`, spell ID, nil. VERIFIED (WM46, WM56): a spell in neither book projects slot 0 and `"spell"`. |
  | 4 PetAction | `"petaction"`, the packed word's low 24 bits, then 0 and nil when the category `(word >> 24) & 63` is 1, otherwise nil and nil. VERIFIED (WM46, WM56). PTR categories 6 and 7 return a book index or 0, then nil (R29); Retail returns nil for the third value. The word is the pet-bar slot's supplied `packed_action`, zero when unsupplied. |
  | 5 Merchant | `"merchant"`, the row index the pickup received (the stored zero-based row plus one) |
  | 6 ActionBar | `"item"`, item ID, link; zero values without item metadata |
  | 12 EquipmentSet | `"equipmentset"`, the set's name (INFERRED: WM46 verifies "the selected record's text"); zero values when the set no longer exists |
  | 13 Currency | `"currency"`, currency ID, icon file ID or nil when zero (INFERRED: WM46 verifies "a metadata integer"); zero values without a `currency_info` record |
  | 21 Outfit | `"outfit"`, outfit ID |

- A missing record projects zero values but does not change the mode or clear the cursor. VERIFIED (WM46).

## Original consumers

- `Blizzard_MainMenuBarBagButtons/Shared/MainMenuBarBagManager.lua` reads `CURSOR_CHANGED`'s new cursor type and expands the bag bar for Item, Merchant and GuildBank cursors.
- `Blizzard_ActionBar/Shared/ActionBar.lua` (`ActionBarMixin`) shows and hides all action buttons on `ACTIONBAR_SHOWGRID` and `ACTIONBAR_HIDEGRID`.

## Implementation inventory

- `src/lua_api/globals/real/cursor_model.rs` — native cursor transitions and supported projections.
- `src/lua_api/globals/inventory_verbs.rs` — action-slot pickup, placement, and slot notifications.
- `src/lua_api/state/sim_state.rs` — pending PTR cooldown-update state.
- `src/lua_api/env_runtime.rs` — coalesced cooldown event delivery after OnUpdate.

## Tests asserting this spec

- `tests/cursor_model_api.rs` — Retail cursor modes, projections, and clear/place events.
- `tests/action_bar_drag.rs` — PTR ActionButton drag, swap, security, locking, tuple, and slot-range behavior.

## Known gaps

- Talent and PvP-talent pickups keep their payload without a native mode. WM54 Q3 selects Flyout 14 or a gated Spell 3 from talent-list membership, the talent's spell attributes, spell relationships and flyout records; the build 70391 export has no talent, PvP-talent, spell-relationship or flyout tables. Clearing a talent cursor emits the argument-less `CURSOR_CHANGED`.
- Mode 1's item-action eligibility sub-branch (appearance selection, class/subclass and native flags) is not modelled; only the use-spell and InventoryType disjuncts are.
- `PickupPlayerMoney(0)` returns before the setter. The setter's zero-amount branch (clear only when the mode is already Money, install nothing) is therefore not reached; the public wrapper's guards are UNKNOWN.
- The Spell setter's supplied clear-notification control is not modelled; its clear always notifies.
- PTR R29 establishes the `PickupAction`/`PlaceAction` event order above; callback re-entry and queued-event delivery timing remain unverified.
- `CursorInfo` has no mount, companion, flyout, or battle-pet variants, so their R29 cursor tuples do not have modeled producers.
- Trade, mail and auction drops still release the cursor without the shared clear or its events.
- `C_EquipmentSet.PickupEquipmentSet` and `C_CurrencyInfo.PickupCurrency` accept any ID: the wrapper guards are UNKNOWN. `PlaceAction` does not place equipment-set or currency cursors on action slots.
- The simulator's pet book is an unordered set, so pet-book searches are misses: a spell only in the pet book projects slot 0 and `"spell"` instead of its pet slot and `"pet"`, and a category-1 pet action projects 0. Overrides are identity-only, so the override value is nil. Companion classification (the three-value `"companion"` tuple) is not modelled.
- `PickupPetAction` installs the slot's supplied word even when it is zero; native entry guards are UNKNOWN. The autocast writers that change the word's high flags (WM56) are not modelled.
- Macro 7 projects the picked-up index. Native matches a macro handle and follows the macro when slots move; the simulator has no handles.
- Modes 8, 10–20 and their projections, guild-bank pickups, and the money storage read by `GetCursorMoney` are not modeled here.
