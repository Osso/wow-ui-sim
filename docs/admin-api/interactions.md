# Interactions

### A_Admin.SetItemLocationInputs(inventoryCount, inventoryModeActive, availableEquipmentSlots)

Supplies Retail ItemLocation bounds. `inventoryCount` is a signed integer or nil, `inventoryModeActive` is a Boolean or nil, and `availableEquipmentSlots` is an array of signed equipment-slot integers. Missing values remain unknown and reject locations that require them.

### A_Admin.SetItemInteractionScenario(interactionType, acceptedItemIDs, selectionID, recordID)

Supplies a Retail item-interaction scenario. `interactionType` and `selectionID` are signed integers; `recordID` and every `acceptedItemIDs` entry are unsigned integers. The setter replaces the current interaction record, explicit pending-item acceptance observations, and bookkeeping IDs. It does not model native acceptance predicates.

### A_Admin.SetPossessionObject(object)

Sets or clears the Retail possession object. The object requires unsigned `identityFirst` and `identitySecond` plus a `candidates` array. Each candidate supplies `spellID`, `tested`, `effects`, `identityTypePresent`, `comparedIdentityFirst`, `comparedIdentitySecond`, and optional `classificationField`. These are host observations, not native object or effect catalogs.

### A_Admin.SetPossessionSpellID(spellID)

Writes the shared Retail possession spell ID and requests `UPDATE_POSSESS_BAR` only when the value changes.

### A_Admin.UpdatePossessionPlayer(identityFirst, identitySecond)

Runs the Retail local possession rescan for the supplied unsigned identity pair. Exact object resolution and candidate ownership remain simulator projections.

These simulator-only helpers drive player-interaction state and its normal manager events. They do not simulate server responses.

## Guild rename (Retail and WoW Forever)

```lua
A_Admin.OpenGuildRename()
```

This activates `Enum.PlayerInteractionType.GuildRename` and fires `PLAYER_INTERACTION_MANAGER_FRAME_SHOW`. The Blizzard PlayerInteractionFrameManager then opens the registered GuildRename frame. Deliver server results separately with `GUILD_RENAME_STATUS_UPDATE` when a test needs them.

## Mailbox

- `A_Admin.OpenMailbox()` activates `MailInfo` and fires the profile's mailbox-open event.
- `A_Admin.CloseMailbox()` clears `MailInfo` and fires the matching close event when the interaction was active.
