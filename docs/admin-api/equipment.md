# Equipment

## Equipped item durability

`A_Admin.SetEquippedItemDurability(slot, current, maximum)` supplies durability for an equipped item. `slot` must be an equipment slot from 1 through 28 that has an equipped item. `current` and `maximum` are unsigned integers; `maximum` must be greater than zero, and `current` must not exceed `maximum`. Invalid values raise an error. Replacing or removing the equipped item clears its supplied durability.

`GetInventoryItemDurability(slot)` returns the supplied pair. An equipped item without a durability input retains the existing `100, 100` scenario default; an empty slot returns nil. The setter fires `UPDATE_INVENTORY_ALERTS`, which updates Blizzard's DurabilityFrame.

INFERRED R88g9 (native evidence pending): `GetInventoryAlertStatus` uses status 1 for supplied durability below maximum and status 2 at zero. These match DurabilityFrame.lua's yellow and red alert entries. The native warning threshold is unknown.

# Equipment item lock observations

These simulator-only inputs are available on Retail, PTR, Forever and Mists. They feed the [equipment-set lock contract](../specs/equipment-set-locks.md); they do not change `C_ItemUpgrade` or infer native item-lock rules.

```lua
A_Admin.SetEquipmentItemLock(211995, true)  -- supply locked
A_Admin.SetEquipmentItemLock(211995, false) -- replace with unlocked
A_Admin.ClearEquipmentItemLock(211995) -- remove this observation
```

The lock argument must be an explicit boolean. Missing, nil and non-boolean values raise a simulator error without replacing the existing observation. Clearing an observation restores the query's missing-observation error for any saved set that references that item ID. Other observations remain unchanged.

Saved sets store slot→item ID. All copies of one item ID therefore share one observation; this is not a claim about native item-instance lock behavior. Inputs do not equip gear, infer lock criteria or synthesize native events. See the [model](../wiki/systems/equipment-set-locks.md).

## Item overlay flags

Retail and PTR scenarios can supply item classifications by ID:

```lua
A_Admin.SetItemOverlayFlag(900001, "corrupted", true)
A_Admin.SetItemOverlayFlag(900001, "corrupted", false)
```

Flag names are `corrupted`, `cosmetic`, `conduit`, `curio`, `relic`, and `decor`. The third argument must be a boolean. False removes the flag. Unset flags return false as a simulator scenario default; this is not native item data.

## Inventory flyout candidates

Retail and Forever tests can supply the candidates consumed by `GetInventoryItemsForSlot`:

```lua
A_Admin.SetInventorySlotCandidates(11, false, {
    { equipmentSlot = 12, link = otherRingLink },
    { carriedSlot = 1, link = carriedRingLink },
}, {
    slotNames = { Finger0Slot = 11 },
    carriedEnabled = true,
    carriedCount = 2,
    interactionGuidNonzero = false,
    transmogReady = false,
})
```

The call replaces the accepted candidate list for that one-based inventory slot and transmogrify mode. Each row uses one-based `equipmentSlot`, one-based `carriedSlot` for the backpack range, or `bagID` plus one-based `bagSlot` (`bagID` defaults to `0`). `carriedEnabled` and `carriedCount` control the carried-slot range. `link` is optional; omit it only when the supplied model should remove that location from the returned table. Options are also optional. `slotNames` adds or replaces name-to-one-based-slot mappings. The other fields supply the carried-slot range, bank-access predicate input, and readiness for this slot's transmog query. This setter does not derive candidates from equipped items or bags, and an empty list supplies no candidates; native eligibility filtering remains unknown.

The existing Retail model publishes these supplied rows through `GetInventoryItemsForSlot`. **INFERRED:** R88k.1 Retail evidence supports this model; Forever registration is inferred from the selected `PaperDollFrameItemFlyout_GetItems` consumer call, with native evidence pending.

## Equipped weapon predicates (Retail and WoW Forever)

`IsRangedWeapon()` and `IsDualWielding()` read the items supplied with
`A_Admin.EquipItem(slot, itemID)` (`real/combat_stats.rs`). WoW Forever reads its
ranged slot (18); Retail has no ranged slot, so a main-hand (16) bow, gun,
crossbow or wand is ranged. `IsDualWielding()` is true when slots 16 and 17 both
hold one-hand weapons. INFERRED (R-ag10-15).
