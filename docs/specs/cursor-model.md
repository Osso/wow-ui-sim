# Cursor model (Retail)

Retail uses one shared clear and placement model for cursor payloads (`src/lua_api/globals/real/cursor_model.rs`). Other profiles keep their earlier behavior: payloads are assigned or dropped without native modes, stored IDs or grid events, and the sites that fired an argument-less `CURSOR_CHANGED` still do.

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
| Transmog outfit | `C_TransmogOutfitInfo.PickupOutfit`, `PickupAction` | 21 Outfit |

VERIFIED (WM46): the mode names and the PickupAction rule that the slot's content selects the setter.

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
- `GetCursorInfo()` for a merchant cursor returns `("merchant", index)`, the row index the pickup received. VERIFIED (WM46 projection; the stored zero-based row plus one).

## Original consumers

- `Blizzard_MainMenuBarBagButtons/Shared/MainMenuBarBagManager.lua` reads `CURSOR_CHANGED`'s new cursor type and expands the bag bar for Item, Merchant and GuildBank cursors.
- `Blizzard_ActionBar/Shared/ActionBar.lua` (`ActionBarMixin`) shows and hides all action buttons on `ACTIONBAR_SHOWGRID` and `ACTIONBAR_HIDEGRID`.

## Known gaps

- Talent and PvP-talent pickups keep their payload without a native mode. WM54 Q3 selects Flyout 14 or a gated Spell 3 from talent-list membership, the talent's spell attributes, spell relationships and flyout records; the build 70391 export has no talent, PvP-talent, spell-relationship or flyout tables. Clearing a talent cursor emits the argument-less `CURSOR_CHANGED`.
- Mode 1's item-action eligibility sub-branch (appearance selection, class/subclass and native flags) is not modelled; only the use-spell and InventoryType disjuncts are.
- `PickupPlayerMoney(0)` returns before the setter. The setter's zero-amount branch (clear only when the mode is already Money, install nothing) is therefore not reached; the public wrapper's guards are UNKNOWN.
- The Spell setter's supplied clear-notification control is not modelled; its clear always notifies.
- Native ordering between action-slot change events and cursor events in `PickupAction`/`PlaceAction` is not established.
- Trade, mail and auction drops still release the cursor without the shared clear or its events.
- Modes 8, 10–20 and their projections, guild-bank pickups, and the money storage read by `GetCursorMoney` are not modeled here.
