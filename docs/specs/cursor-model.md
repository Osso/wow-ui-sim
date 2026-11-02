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
- The grid flag is set by pickups of modes 3, 4, 6, 7, 12, 13, 14, 16, 17 and 21, after `ACTIONBAR_SHOWGRID`. Owned items (mode 1) never show the grid. VERIFIED (WM46), except mode 1 (see gaps).

### Clear

`ClearCursor`, `DeleteCursorItem`, `PlaceAction`, `DropCursorMoney`, `PutItemInBackpack`/`PutItemInBag` and dropping an owned item into an empty bag or equipment slot run the shared clear:

1. If the grid flag is set, `ACTIONBAR_HIDEGRID`, then the flag is reset.
2. For a nonempty cursor, `CURSOR_CHANGED(true, 0, oldMode, S before the clear)`. An empty cursor emits nothing.

VERIFIED (WM46, WM44-A).

### Placement

Every producer above first runs the shared clear, then stores the payload, then emits `ACTIONBAR_SHOWGRID` when its mode shows the grid, then `CURSOR_CHANGED(false, mode, 0, id)`:

- Item setters (merchant, ActionBar) report S after the clear. VERIFIED (WM44-A, WM46).
- Typed setters report 0 as the fourth value. VERIFIED (WM46). The third value 0 and the leading shared clear for typed setters are INFERRED from the WM44-B mode-1 payload `(false, 1, 0, 0)`; native confirmation is pending (WM54).
- Swapping an owned item with a bag or equipment slot clears the held item, then places the displaced one. INFERRED.

### Predicates and projections

- `CursorHasItem()` is true only for mode 1, so merchant and ActionBar cursors fail it. `CursorHasMoney()` tests mode 2, `CursorHasSpell()` mode 3 and `CursorHasMacro()` mode 7. VERIFIED (WM46).
- `GetCursorInfo()` for a merchant cursor returns `("merchant", index)`, the row index the pickup received. VERIFIED (WM46 projection; the stored zero-based row plus one).

## Original consumers

- `Blizzard_MainMenuBarBagButtons/Shared/MainMenuBarBagManager.lua` reads `CURSOR_CHANGED`'s new cursor type and expands the bag bar for Item, Merchant and GuildBank cursors.
- `Blizzard_ActionBar/Shared/ActionBar.lua` (`ActionBarMixin`) shows and hides all action buttons on `ACTIONBAR_SHOWGRID` and `ACTIONBAR_HIDEGRID`.

## Known gaps

- Talent and PvP-talent pickups keep their payload without a native mode; WM46 does not establish whether they select Flyout 14 or Spell 3. Clearing a talent cursor emits the argument-less `CURSOR_CHANGED`.
- Mode 1 grid display depends on item resolution, category 17 and two eligibility predicates with UNKNOWN meanings; owned items never show the grid.
- Native ordering between action-slot change events and cursor events in `PickupAction`/`PlaceAction` is not established.
- Trade, mail and auction drops still release the cursor without the shared clear or its events.
- Modes 8, 10–20 and their projections, guild-bank pickups, and the money storage read by `GetCursorMoney` are not modeled here.
