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
